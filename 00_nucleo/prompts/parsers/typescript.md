# Prompt: TypeScript Parser (parsers/typescript)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-typescript`
**Criado em**: 2026-03-18 (ADR-0009)
**Revisado em**: 2026-03-18 (ADR-0009 correcção: ImportKind semântico)
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

**ADR-0009 correcção**: `ImportKind` é semântico, não sintáctico.
Os nós de import TypeScript são mapeados para `Direct/Glob/Alias/Named`
— nunca para variantes específicas de linguagem como `EsImport`.

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

O bloco termina na primeira linha que não começa com `//`.

**Justificativa:** `/** */` foi rejeitado porque ferramentas de
documentação (TypeDoc, ESLint jsdoc plugin) interpretam `@prompt`
e `@layer` como anotações JSDoc.

---

## Resolução de camadas — TsLayerResolver

A resolução é física, não léxica (invariante I2 do `_template.md`).

### Passo 1 — Detecção de package externo

Se o import não começa com `./`, `../` ou uma chave de
`[ts_aliases]`, é um package npm externo → `Layer::Unknown`
directamente:

```
"express"           →  Layer::Unknown
"@angular/core"     →  Layer::Unknown  (escopo npm, não alias cristalino)
"node:fs"           →  Layer::Unknown
"./utils"           →  continua para passo 2
"../01_core/layer"  →  continua para passo 2
"@core/entities"    →  continua para passo 2 (alias configurado)
```

### Passo 2 — Resolução de alias

Se o import começa com uma chave de `[ts_aliases]`, substituir:

```toml
[ts_aliases]
"@core"  = "01_core"
"@shell" = "02_shell"
"@infra" = "03_infra"
```

```
"@core/entities/layer"  →  "01_core/entities/layer"
"@infra/walker"         →  "03_infra/walker"
```

### Passo 3 — Álgebra de paths com verificação de fuga

```rust
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if components.is_empty() {
                    return None; // tenta sair da raiz → Layer::Unknown
                }
                components.pop();
            }
            Component::CurDir => {}
            c => { components.push(c); }
        }
    }
    let result: PathBuf = components.iter().collect();
    if result.starts_with(project_root) {
        Some(result)
    } else {
        None
    }
}
```

`std::fs::canonicalize` é proibido — normalização algébrica apenas.

### Passo 4 — Reutilização de `resolve_file_layer`

```rust
let base = file.path.parent().unwrap_or(Path::new("."));
let joined = base.join(import_str_after_alias);
let target_layer = match normalize(&joined, &project_root) {
    Some(normalized) => resolve_file_layer(&normalized, &project_root, &config),
    None             => Layer::Unknown,
};
```

### `target_subdir` para V9

```rust
fn resolve_subdir<'a>(
    normalized: &Path,
    target_layer: &Layer,
    project_root: &Path,
    config: &CrystallineConfig,
) -> Option<&'a str> {
    if *target_layer != Layer::L1 { return None; }
    let layer_dir = config.layers.get("L1")?;
    let base = project_root.join(layer_dir);
    let relative = normalized.strip_prefix(&base).ok()?;
    relative.components().next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s)
}
```

---

## Extracção de imports (V3, V9, V10)

Nós AST relevantes: `import_statement`, `import_require_clause`,
`export_statement` com `from`.

| Nó AST | `ImportKind` | Exemplo |
|--------|-------------|---------|
| `import X from '...'` | `Direct` | import default |
| `import '...'` (side-effect) | `Direct` | import sem binding |
| `import * as ns from '...'` | `Glob` | namespace import |
| `import X as Y from '...'` | `Alias` | import com rename |
| `import { A, B } from '...'` | `Named` | named imports |
| `import { A as B } from '...'` | `Alias` | named com rename |
| `export { X } from '...'` | `Direct` | re-export directo |
| `export * from '...'` | `Glob` | re-export glob |
| `export { A as B } from '...'` | `Alias` | re-export com rename |
| `require('...')` | `Direct` | CommonJS require |

Para cada import: `path` = string literal sem aspas — fatia `&'a str`
do buffer. `target_layer` via TsLayerResolver (4 passos).
`target_subdir` via `resolve_subdir` após resolução física.
`Layer::Lab` resolvido para imports de `lab/` — V10 usa este valor.

**Imports dinâmicos:** `import('./foo')` não são capturados como
`Import` — não têm string literal estática analisável.

---

## Extracção de tokens — símbolos proibidos (V4)

V4 usa `file.language()` para seleccionar a lista de símbolos
proibidos — não usa `ImportKind`. A lista TypeScript vive em
`impure_core.rs` via `forbidden_symbols_for(Language::TypeScript)`.

**Mecanismo 1 — Import de módulo proibido:**
Se `target_layer == Layer::Unknown` e o path do import está na
lista de módulos proibidos, os identificadores importados são
registados como tokens proibidos com o FQN do módulo.

**Mecanismo 2 — Call expression proibida:**
Nós `call_expression` cujo identificador resolve para símbolo
proibido:
```
process.env   →  Token { symbol: "process.env", .. }
Date.now()    →  Token { symbol: "Date.now", .. }
Math.random() →  Token { symbol: "Math.random", .. }
```

**Sem Motor de Duas Fases:** TypeScript não tem o sistema de
aliases de módulo de Rust (`use X as Y`). V4 opera directamente
sobre os `call_expression` do AST.

---

## Test coverage (V2)

`has_test_coverage = true` se qualquer das condições:

1. **Construto de teste no AST:** nós `call_expression` de nível
   superior com identificador `describe`, `it`, `test` ou `suite`.

2. **Ficheiro adjacente:** `source_file.has_adjacent_test` — `true`
   se existe `.test.ts`, `.spec.ts`, `.test.tsx` ou `.spec.tsx`.

3. **Declaration-only (isento):** ficheiro que exporta apenas
   `interface`, `type` e `export type` sem implementação.

---

## Interface pública (V6)

| Nó AST | Resultado | `TypeKind` |
|--------|-----------|------------|
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
```typescript
export interface FileProvider { ... }   →  declared_traits = ["FileProvider"]
interface InternalHelper { ... }        →  ignorado (sem export)
```

`type X = { ... }` não é capturado — apenas `interface` com `export`.

---

## Interfaces implementadas (V11) — `implemented_traits`

Apenas quando `file.layer == Layer::L2 | Layer::L3`.

Para cada nó `class_declaration` com cláusula `implements`:
- Para cada nome na cláusula `implements` (pode ser múltiplos):
  extrair nome simples (último segmento se qualificado)
- Adicionar a `implemented_traits`

```typescript
class FileWalker implements FileProvider { ... }
  →  implemented_traits = ["FileProvider"]

class RustParser implements LanguageParser, Disposable { ... }
  →  implemented_traits = ["LanguageParser", "Disposable"]
  — múltiplos implements capturados individualmente

class InternalHelper { ... }  // sem implements
  →  ignorado
```

---

## Declarações de tipo (V12) — `declarations`

Para todos os arquivos, sem filtro de layer. V12 filtra internamente.

| Nó AST | `DeclarationKind` | Condição |
|--------|------------------|----------|
| `class_declaration` sem `implements` | `Class` | sempre |
| `class_declaration` com `implements` | **não capturado** | adapter |
| `interface_declaration` | `Interface` | sempre |
| `type_alias_declaration` | `TypeAlias` | sempre |
| `enum_declaration` | `Enum` | sempre |

**`class com implements`** é o padrão de adapter em L4 — equivalente
a `impl Trait for Type` em Rust. Não é capturado em `declarations`.

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
        // 1. header (blocos // no topo)
        // 2. imports (TsLayerResolver 4 passos + ImportKind semântico)
        //    import X from       → Direct
        //    import * as ns from → Glob
        //    import X as Y from  → Alias
        //    import { A, B }     → Named
        //    import { A as B }   → Alias
        //    export { X } from   → Direct
        //    export * from       → Glob
        // 3. tokens (imports proibidos + call expressions)
        //    V4 usa file.language(), não ImportKind
        // 4. has_test_coverage
        // 5. public_interface + prompt_snapshot
        // 6. declared_traits (L1/contracts, export interface)
        // 7. implemented_traits (L2|L3, class implements)
        // 8. declarations (class-sem-implements/interface/type/enum)
    }
}
```

---

## Restrições

- `parse()` recebe `&'a SourceFile` — proibido consumir ownership
- Proibido `.to_string()` para strings do buffer
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para
  `Layer::Unknown`, nunca silenciado, nunca panic
- `std::fs::canonicalize` proibido
- Imports dinâmicos não são capturados como `Import`
- `class com implements` não é capturado em `declarations`
- `declared_traits` apenas em L1/contracts/
- `implemented_traits` apenas em L2|L3
- `declarations` para todos os arquivos — V12 filtra por layer
- `PromptReader` e `PromptSnapshotReader` são injetados
- `std::io::Error` nunca atravessa para L1
- **`ImportKind` nunca contém variantes específicas de linguagem:**
  os nós TypeScript mapeiam para `Direct/Glob/Alias/Named` —
  nunca para `EsImport` ou outra variante TS
- **V4 usa `file.language()`, não `ImportKind`**, para seleccionar
  a lista de símbolos proibidos

---

## Critérios de Verificação

```
Dado SourceFile .ts com header cristalino completo em comentários //
Quando parse() for chamado
Então prompt_header populado com todos os campos como &'a str

Dado SourceFile com import { X } from '../01_core/entities/layer'
Quando parse() for chamado
Então imports contém Import {
    kind: ImportKind::Named,
    target_layer: Layer::L1,
    target_subdir: Some("entities"),
    ..
}

Dado SourceFile com import X from '../01_core/entities/layer'
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Direct, .. }

Dado SourceFile com import * as ns from '../01_core/entities'
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Glob, .. }

Dado SourceFile com import { X as Y } from '../01_core/entities'
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Alias, .. }

Dado SourceFile com export * from '../01_core/entities'
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Glob, .. }

Dado SourceFile com import { X } from '../../src/../01_core/entities'
(path com ../ que normaliza correctamente)
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1, .. }

Dado SourceFile com import { X } from '../../../../../etc/passwd'
(path que escapa da raiz)
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }

Dado [ts_aliases] com "@core" = "01_core"
E SourceFile com import { W } from '@core/entities/layer'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1,
      target_subdir: Some("entities"), .. }

Dado SourceFile com import { X } from '../lab/experiment'
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Lab, .. }

Dado SourceFile com export { X } from '../01_core/entities'
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Direct, .. }
— re-export directo mapeia para Direct

Dado SourceFile em L1 com import { readFileSync } from 'fs'
Quando parse() for chamado
Então tokens contém Token com symbol relacionado a "fs"
E V4 usa file.language() = TypeScript para detectar "fs" como proibido

Dado SourceFile em L1 com chamada Date.now()
Quando parse() for chamado
Então tokens contém Token { symbol: "Date.now", .. }

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

Dado SourceFile em L1/contracts/ com:
  export interface FileProvider { ... }
  export interface LanguageParser { ... }
  interface InternalHelper { ... }
Quando parse() for chamado
Então declared_traits = ["FileProvider", "LanguageParser"]
E "InternalHelper" não aparece

Dado SourceFile em L1/rules/ com export interface HasImports { ... }
Quando parse() for chamado
Então declared_traits = []
— apenas contracts/ contribui

Dado SourceFile em L3 com:
  class FileWalker implements FileProvider { ... }
  class InternalHelper { ... }
Quando parse() for chamado
Então implemented_traits = ["FileProvider"]
E "InternalHelper" não aparece

Dado SourceFile em L4 com:
  class L3HashAdapter implements HashRewriter { ... }
  class OutputFormatter { ... }
  interface InternalConfig { ... }
  type Mode = 'text' | 'sarif'
Quando parse() for chamado
Então declarations contém:
  Declaration { kind: Class,     name: "OutputFormatter", .. }
  Declaration { kind: Interface, name: "InternalConfig",  .. }
  Declaration { kind: TypeAlias, name: "Mode",            .. }
E NÃO contém Declaration para "L3HashAdapter"
— class com implements é adapter, não capturado

Dado SourceFile com export { X } from './other'
Quando parse() for chamado
Então public_interface.reexports contém o path de re-export

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
Então nenhum acesso a disco ocorre
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-18 | Criação inicial (ADR-0009) | ts_parser.rs |
| 2026-03-18 | ADR-0009 correcção: ImportKind semântico — tabela de mapeamento TS→Direct/Glob/Alias/Named; EsImport removido; nota sobre V4 usar file.language(); restrição de agnósticidade adicionada; critérios de ImportKind adicionados | ts_parser.rs |
