# Prompt: TypeScript Parser (parsers/typescript)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-typescript`
**Criado em**: 2026-03-18 (ADR-0009)
**Arquivos gerados**:
  - 03_infra/ts_parser.rs + test

---

## Contexto

O núcleo L1 aguarda um `ParsedFile<'a>` completo e agnóstico.
Esta camada L3 faz o trabalho impuro para TypeScript: recebe
referência de `SourceFile`, aciona `tree-sitter-typescript`, e
traduz a AST nos campos que as regras V1–V12 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs` — a mesma trait do
`RustParser`. As regras de L1 não sabem que linguagem estão
a analisar.

Recebe três dependências injetadas via construtor:
- `PromptReader` — para V1 e V5
- `PromptSnapshotReader` — para V6
- `CrystallineConfig` — para resolução de camadas, subdirs,
  aliases TypeScript e configuração de wiring

**Diretiva Zero-Copy (ADR-0004):** `parse()` recebe `&'a SourceFile`
e retorna `ParsedFile<'a>` com referências ao buffer do fonte.

**Resolução física (ADR-0009):** A camada de um import é determinada
pelo caminho físico no disco após normalização algébrica, não pelo
texto do import. `../../src/../01_core/entities`, `../01_core/entities`
e `@core/entities` resolvem para o mesmo caminho canónico e para a
mesma camada. O texto do import é irrelevante.

---

## Header cristalino

Ficheiros `.ts` e `.tsx` usam comentários de linha `//` em bloco
contíguo no topo:

```typescript
// Crystalline Lineage
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
```

O bloco termina na primeira linha que não começa com `//` —
mesma semântica do `//!` em Rust.

**Justificativa:** `/** */` foi rejeitado porque ferramentas de
documentação (TypeDoc, ESLint jsdoc plugin) interpretam `@prompt`
e `@layer` como anotações JSDoc, gerando falsos positivos em
pipelines de documentação. `//` é universal e não tem semântica
especial em nenhuma ferramenta do ecossistema TypeScript.

**Extracção:** varrer linhas do buffer enquanto começam com `//`.
Parar na primeira que não começa. Field matching sobre `@prompt`,
`@prompt-hash`, `@layer`, `@updated` — fatias `&'a str` do buffer.

---

## Resolução de camadas — TsLayerResolver

A resolução é física, não léxica (invariante I2 do `_template.md`).

### Passo 1 — Detecção de package externo

Se o import não começa com `./`, `../` ou uma chave de
`[ts_aliases]` no `crystalline.toml`, é um package npm externo
→ `Layer::Unknown` directamente, sem álgebra de paths:

```
"express"           →  Layer::Unknown  (package npm)
"@angular/core"     →  Layer::Unknown  (package npm com escopo)
"node:fs"           →  Layer::Unknown  (built-in Node, tratado como externo)
"./utils"           →  continua para passo 2
"../01_core/layer"  →  continua para passo 2
"@core/entities"    →  continua para passo 2 (alias configurado)
```

### Passo 2 — Resolução de alias

Se o import começa com uma chave de `[ts_aliases]`, substituir
pelo valor correspondente:

```toml
# crystalline.toml
[ts_aliases]
"@core"  = "01_core"
"@shell" = "02_shell"
"@infra" = "03_infra"
```

```
"@core/entities/layer"  →  "01_core/entities/layer"
"@infra/walker"         →  "03_infra/walker"
"../01_core/layer"      →  sem alteração (não é alias)
```

### Passo 3 — Álgebra de paths com verificação de fuga

```rust
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // pop() em vec vazio = tentativa de sair da raiz
                // → caminho inválido → caller retorna Layer::Unknown
                if components.is_empty() {
                    return None;
                }
                components.pop();
            }
            Component::CurDir => {}
            c => { components.push(c); }
        }
    }
    let result: PathBuf = components.iter().collect();
    // Garantia adicional: resultado dentro da raiz do projecto
    if result.starts_with(project_root) {
        Some(result)
    } else {
        None
    }
}
```

`std::fs::canonicalize` é proibido — requer que o ficheiro exista
no disco, falha em CI com repos parciais. A normalização algébrica
é determinística e suficiente.

### Passo 4 — Reutilização de `resolve_file_layer`

```rust
let base = file.path.parent().unwrap_or(Path::new("."));
let joined = base.join(import_str_after_alias);
let target_layer = match normalize(&joined, &project_root) {
    Some(normalized) => resolve_file_layer(&normalized, &project_root, &config),
    None             => Layer::Unknown,
};
```

A mesma função do `FileWalker` é a fonte de verdade. Zero
duplicação de lógica de resolução.

### `target_subdir` para V9

Extraído do caminho normalizado via `strip_prefix` contra o valor
da camada em `[layers]`, depois tomando o primeiro segmento
restante:

```rust
fn resolve_subdir<'a>(
    normalized: &Path,
    target_layer: &Layer,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Option<&'a str> {
    if *target_layer != Layer::L1 { return None; }
    let layer_dir = config.layers.get("L1")?; // ex: "01_core"
    let base = project_root.join(layer_dir);
    let relative = normalized.strip_prefix(&base).ok()?;
    // Primeiro segmento é o subdir: "entities/layer.ts" → "entities"
    relative.components().next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| /* intern s no buffer */ s)
}
```

`target_subdir` é `Some(subdir)` para imports de L1 independentemente
de o subdir estar em `[l1_ports]`. V9 decide.

---

## Extracção de imports (V3, V9, V10)

Nós AST relevantes: `import_statement`, `import_require_clause`,
`export_statement` com `from`.

| Campo | Como extrair |
|-------|--------------|
| `path` | String literal do nó `string` filho do import — fatia `&'a str` do buffer, sem aspas |
| `line` | `node.start_position().row + 1` |
| `kind` | `ImportKind::EsImport` para todos os imports TypeScript |
| `target_layer` | TsLayerResolver — 4 passos descritos acima |
| `target_subdir` | `resolve_subdir` após resolução física — apenas para L1 |

**Imports dinâmicos:** `import('./foo')` como `call_expression`
são capturados como tokens (V4) mas não como `Import` — não têm
string literal estática analisável em tempo de parse.

**Re-exports:** `export { X } from './foo'` e `export * from './foo'`
geram `Import` com `target_layer` resolvido — V3 e V10 aplicam-se.

---

## Extracção de tokens — símbolos proibidos (V4)

V4 proíbe I/O em L1. Em TypeScript, I/O ocorre via imports de
módulos Node.js built-in ou via chamadas a `process`, `Date` e
`Math`. A detecção usa dois mecanismos:

**Mecanismo 1 — Import de módulo proibido:**
Se `target_layer == Layer::Unknown` e o path do import está na
lista de módulos proibidos, todos os identificadores importados
são registados como tokens proibidos com o FQN do módulo.

Lista de módulos proibidos em L1:
```
fs, node:fs, fs/promises, node:fs/promises,
child_process, node:child_process,
net, node:net,
http, node:http,
https, node:https,
dgram, node:dgram,
dns, node:dns,
readline, node:readline,
```

**Mecanismo 2 — Call expression proibida:**
Nós `call_expression` cujo identificador resolve para um símbolo
proibido:

```
process.env         →  Token { symbol: "process.env", .. }
Date.now()          →  Token { symbol: "Date.now", .. }
Math.random()       →  Token { symbol: "Math.random", .. }
```

Extraídos como `Cow::Borrowed` se o símbolo está literalmente
no buffer, `Cow::Owned` se requer concatenação.

**Sem Motor de Duas Fases:** TypeScript não tem o sistema de aliases
de módulo de Rust (`use X as Y`). Os imports são resolvidos
fisicamente no TsLayerResolver — não há necessidade de uma fase
separada de resolução de aliases para tokens. V4 opera directamente
sobre os `call_expression` do AST.

---

## Test coverage (V2)

`has_test_coverage = true` se qualquer das condições:

**1. Construto de teste no AST:**
Nós `call_expression` de nível superior (ou dentro de
`export_statement`) cujo identificador é `describe`, `it`,
`test` ou `suite`. Detecta Jest, Vitest, Mocha e equivalentes.

**2. Ficheiro de teste adjacente:**
`source_file.has_adjacent_test` — `true` se existe
`<stem>.test.ts`, `<stem>.spec.ts`, `<stem>.test.tsx` ou
`<stem>.spec.tsx` no mesmo directório.

**3. Declaration-only (isento de V2):**
Ficheiro que exporta apenas `interface`, `type` e `export type`
sem nenhuma implementação (`function`, `class`, `const` com corpo).
Equivalente ao ficheiro Rust que declara apenas `pub trait` e
`pub struct` sem `impl` com corpo.

---

## Interface pública (V6)

Nós AST com `export` de nível superior:

| Nó | `FunctionSignature` / `TypeSignature` | `TypeKind` |
|----|--------------------------------------|------------|
| `function_declaration` com `export` | `FunctionSignature` | — |
| `lexical_declaration` com `export` e tipo função | `FunctionSignature` | — |
| `class_declaration` com `export` | `TypeSignature` | `Class` |
| `interface_declaration` com `export` | `TypeSignature` | `Interface` |
| `type_alias_declaration` com `export` | `TypeSignature` | `TypeAlias` |
| `enum_declaration` com `export` | `TypeSignature` | `Enum` |
| `export_statement` com `from` | `reexports` | — |
| `export_statement` `{ X }` sem `from` | `reexports` | — |

**`FunctionSignature`:**
- `name`: identificador da função — `&'a str` do buffer
- `params`: tipos dos parâmetros normalizados (whitespace colapsado)
- `return_type`: tipo de retorno se explícito, `None` se omitido
  ou `void`

**`TypeSignature`:**
- `name`: identificador do tipo — `&'a str` do buffer
- `kind`: conforme tabela acima
- `members`: para `Class` → nomes de campos e métodos públicos;
  para `Interface` → nomes das propriedades e métodos;
  para `TypeAlias` → `[]` (corpo é opaco para V6);
  para `Enum` → nomes dos membros

**Normalização de tipos:**
Whitespace colapsado nos tipos de parâmetros e retorno.
`Promise < string >` → `Promise<string>`.
Tipos de utilidade preservados literalmente: `Partial<Foo>`,
`Record<string, number>`.

**`prompt_snapshot`:** via `PromptSnapshotReader::read_snapshot` —
idêntico para todas as linguagens.

---

## Interfaces declaradas (V11) — `declared_traits`

Apenas quando `file.layer == Layer::L1` e path contém `"contracts"`.

Para cada nó `interface_declaration` de nível superior com `export`:
- Extrair `name` como `&'a str` do buffer
- Adicionar a `declared_traits`

```typescript
// 01_core/contracts/file_provider.ts
export interface FileProvider { ... }   →  declared_traits = ["FileProvider"]
export interface LanguageParser { ... } →  declared_traits = ["FileProvider", "LanguageParser"]
interface InternalHelper { ... }        →  ignorado (sem export)
```

Ficheiros em L1 fora de `contracts/` não contribuem.
`type X = { ... }` não é capturado — apenas `interface` declarada
com `export` constitui um contrato de porta.

---

## Interfaces implementadas (V11) — `implemented_traits`

Apenas quando `file.layer == Layer::L2 | Layer::L3`.

Para cada nó `class_declaration` com cláusula `implements`:
- Para cada nome na cláusula `implements` (pode ser múltiplos):
  extrair nome simples (último segmento se qualificado)
- Adicionar a `implemented_traits`

```typescript
// 03_infra/walker.ts
class FileWalker implements FileProvider { ... }
  →  implemented_traits = ["FileProvider"]

class RustParser implements LanguageParser, Disposable { ... }
  →  implemented_traits = ["FileProvider", "LanguageParser", "Disposable"]

class InternalHelper { ... }  // sem implements
  →  ignorado
```

Ficheiros em L1 ou L4 não contribuem.

---

## Declarações de tipo (V12) — `declarations`

Para todos os arquivos, sem filtro de layer. V12 filtra por
`layer == L4` internamente.

| Nó | `DeclarationKind` | Condição |
|----|------------------|----------|
| `class_declaration` | `Class` | sempre capturado |
| `interface_declaration` | `Interface` | sempre capturado |
| `type_alias_declaration` | `TypeAlias` | sempre capturado |
| `enum_declaration` | `Enum` | sempre capturado |
| `class_declaration` com `implements` | **não capturado em `declarations`** | é adapter — V12 usa `Class` só para impl sem interface |

**Nota sobre `Class` com `implements`:**
`class Foo implements Bar { }` é o padrão de adapter em L4 —
equivalente a `impl Trait for Type` em Rust. Não é capturado
em `declarations`. Apenas `class Foo { }` sem `implements` é
capturado como `DeclarationKind::Class`.

**`allow_adapter_structs` e TypeScript:**
`DeclarationKind::Class` é tratado como `Struct` por V12 para
fins de `allow_adapter_structs = true`. Classes de adapter em L4
sem `implements` são permitidas com o padrão por omissão.

---

## Assinatura do construtor

```rust
pub struct TsParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
}

impl<R: PromptReader, S: PromptSnapshotReader> TsParser<R, S> {
    pub fn new(
        prompt_reader: R,
        snapshot_reader: S,
        config: CrystallineConfig,
    ) -> Self {
        Self { prompt_reader, snapshot_reader, config }
    }
}

impl<R, S> LanguageParser for TsParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        // Ordem de extracção:
        // 1. header (blocos // no topo — @prompt, @prompt-hash, @layer, @updated)
        // 2. imports (TsLayerResolver 4 passos + SubdirResolver físico)
        // 3. tokens (imports proibidos + call expressions — sem Motor de Duas Fases)
        // 4. has_test_coverage (describe/it/test + adjacência + declaration-only)
        // 5. public_interface + prompt_snapshot (V6)
        // 6. declared_traits (apenas L1/contracts, apenas interface com export) (V11)
        // 7. implemented_traits (apenas L2|L3, apenas class com implements) (V11)
        // 8. declarations — class/interface/type/enum sem implements (V12)
        //
        // Retorna ParsedFile<'a> com todas as referências apontando
        // para file.content
    }
}
```

---

## Restrições

- `parse()` recebe `&'a SourceFile` — proibido consumir ownership
- Proibido `.to_string()` para strings do buffer — apenas
  `PromptHeader.current_hash` é `String` (calculado do disco)
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para
  `Layer::Unknown`, nunca silenciado, nunca panic
- `std::fs::canonicalize` proibido — normalização algébrica apenas
- Imports dinâmicos (`import('./foo')`) não são capturados como
  `Import` — não têm string estática analisável
- `class com implements` não é capturado em `declarations` —
  é adapter, não lógica de negócio
- `declared_traits` apenas em L1/contracts/ — filtragem no parser
- `implemented_traits` apenas em L2|L3 — filtragem no parser
- `declarations` para todos os arquivos — V12 filtra por layer
- `PromptReader` e `PromptSnapshotReader` são injetados —
  `TsParser` nunca os instancia directamente
- `std::io::Error` nunca atravessa para L1 — convertido em
  `ParseError` antes de retornar

---

## Critérios de Verificação

```
Dado SourceFile .ts com header cristalino completo em comentários //
Quando parse() for chamado
Então prompt_header populado com todos os campos como &'a str

Dado SourceFile com import { X } from '../01_core/entities/layer'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1,
      target_subdir: Some("entities"), kind: EsImport, .. }

Dado SourceFile com import { Y } from '../../src/../01_core/entities'
(path com ../ excessivos mas ainda dentro da raiz)
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1, .. }
— normalização física resolve para o mesmo caminho que ../01_core/entities

Dado SourceFile com import { Z } from '../../../../../etc/passwd'
(path que escapa da raiz do projecto)
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
— normalize() retorna None, Layer::Unknown propaga

Dado [ts_aliases] com "@core" = "01_core"
E SourceFile com import { W } from '@core/entities/layer'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1,
      target_subdir: Some("entities"), .. }
— alias resolvido antes da álgebra de paths

Dado SourceFile com import { readFile } from 'fs'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
E tokens contém Token com symbol relacionado a "fs"
— package npm externo, mas símbolo proibido em V4

Dado SourceFile em L1 com import express from 'express'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
— package npm, não dispara V3 ou V10

Dado SourceFile com import { X } from '../lab/experiment'
E lab/ mapeado em [layers]
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Lab, .. }
— import de lab detectado, V10 dispara em produção

Dado SourceFile em L1 com import { readFileSync } from 'fs'
Quando parse() for chamado
Então tokens contém Token { symbol: "fs.readFileSync", .. }
— símbolo proibido V4

Dado SourceFile em L1 com chamada Date.now()
Quando parse() for chamado
Então tokens contém Token { symbol: "Date.now", .. }
— estado não-determinístico proibido em V4

Dado SourceFile em L1 com chamada Math.random()
Quando parse() for chamado
Então tokens contém Token { symbol: "Math.random", .. }

Dado SourceFile com describe('suite', () => { it('test', ...) })
Quando parse() for chamado
Então has_test_coverage = true

Dado SourceFile com source_file.has_adjacent_test = true
Quando parse() for chamado
Então has_test_coverage = true

Dado SourceFile com apenas:
  export interface FileProvider { files(): SourceFile[] }
  export type Config = { root: string }
Quando parse() for chamado
Então has_test_coverage = true — declaration-only, isento de V2

Dado SourceFile com:
  export function check(file: ParsedFile): Violation[] { ... }
Quando parse() for chamado
Então public_interface.functions contém FunctionSignature {
    name: "check",
    params: ["ParsedFile"],
    return_type: Some("Violation[]")
}

Dado SourceFile com:
  export class FileWalker implements FileProvider { ... }
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "FileWalker",
    kind: TypeKind::Class,
    ..
}

Dado SourceFile com:
  export interface LanguageParser { parse(file: SourceFile): ParsedFile }
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "LanguageParser",
    kind: TypeKind::Interface,
    ..
}

Dado SourceFile com:
  export type Layer = 'L1' | 'L2' | 'L3' | 'L4'
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "Layer",
    kind: TypeKind::TypeAlias,
    ..
}

Dado SourceFile com export { X } from './other'
Quando parse() for chamado
Então public_interface.reexports contém o path de re-export

Dado SourceFile em L1/contracts/ com:
  export interface FileProvider { ... }
  export interface LanguageParser { ... }
  interface InternalHelper { ... }
Quando parse() for chamado
Então declared_traits = ["FileProvider", "LanguageParser"]
E "InternalHelper" não aparece — sem export

Dado SourceFile em L1/rules/ com export interface HasImports { ... }
Quando parse() for chamado
Então declared_traits = []
— apenas contracts/ contribui

Dado SourceFile em L3 com:
  class FileWalker implements FileProvider { ... }
  class InternalHelper { ... }
Quando parse() for chamado
Então implemented_traits = ["FileProvider"]
E "InternalHelper" não aparece — sem implements

Dado SourceFile em L4 com:
  class L3HashAdapter implements HashRewriter { ... }
  class OutputFormatter { ... }
  interface InternalConfig { ... }
  type Mode = 'text' | 'sarif'
Quando parse() for chamado
Então declarations contém:
  Declaration { kind: Class,     name: "OutputFormatter",  .. }
  Declaration { kind: Interface, name: "InternalConfig",   .. }
  Declaration { kind: TypeAlias, name: "Mode",             .. }
E NÃO contém Declaration para "L3HashAdapter"
— class com implements é adapter, não capturado

Dado SourceFile .ts sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado SourceFile .ts vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })

Dado SourceFile com language = Language::Rust num TsParser
Quando parse() for chamado
Então retorna Err(ParseError::UnsupportedLanguage { .. })

Dado NullPromptReader e NullSnapshotReader como mocks
Quando parse() for chamado
Então nenhum acesso a disco ocorre durante testes
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-18 | Criação inicial (ADR-0009): TsParser completo com resolução física, header //, V2/V4/V6/V11/V12 adaptados para TypeScript | ts_parser.rs |
| 2026-03-19 | Fix: has_implements_clause e collect_implements actualizados para estrutura real tree-sitter-typescript 0.21 (class_declaration → class_heritage → implements_clause); resolução de alias corrigida para usar project_root como base; 295/295 testes passam | ts_parser.rs |
