# ⚖️ ADR-0009: Suporte TypeScript

**Status**: `PROPOSTO`
**Data**: 2026-03-18

---

## 💎 Formalism ($\mathcal{L}_{adr}$)

* **Agnósticidade de Linguagem**: Seja $R$ o conjunto de regras V1–V12
  e $L$ o conjunto de linguagens suportadas. Para toda regra $r \in R$
  e toda linguagem $l \in L$: $r$ opera sobre $ParsedFile\langle'a\rangle$
  agnóstico — nunca sobre a AST bruta de $l$.
* **Completude de Parser**: Seja $F$ o conjunto de campos de
  $ParsedFile\langle'a\rangle$. Um `LanguageParser` para $l$ é completo
  sse $\forall f \in F : f$ é populado correctamente a partir da
  gramática de $l$.
* **Invariante de Resolução**: Seja $i$ um import TypeScript qualquer.
  $resolve(i) = resolve(normalize(canonical(file.dir, i)))$ — a camada
  resolvida depende do caminho físico no disco, nunca do texto do import.
  $\forall i_1, i_2 : canonical(i_1) = canonical(i_2) \implies resolve(i_1) = resolve(i_2)$

---

## Contexto

A Arquitectura Cristalina é agnóstica de linguagem por design — o
núcleo L1 opera sobre `ParsedFile<'a>` sem conhecer a gramática de
nenhuma linguagem. O ADR-0001 estabeleceu tree-sitter como parser
porque precisamente suporta múltiplas linguagens com a mesma
interface.

TypeScript é a primeira extensão natural após Rust porque é a
linguagem dominante em projectos de agentes de IA, e o ecossistema
de tooling que o linter pretende guardar é maioritariamente
TypeScript.

No entanto, TypeScript introduz quatro diferenças estruturais em
relação a Rust que precisam de decisão formal antes de qualquer
código:

### 1. Sintaxe de header

`//!` é comentário de módulo em Rust. Em TypeScript não existe
equivalente directo.

### 2. Resolução de camadas via imports

Rust usa `crate::module` com paths absolutos desde a raiz —
irrefutável. TypeScript usa paths relativos (`../01_core/`) ou
aliases de `tsconfig.json` (`@core/`). Uma abordagem léxica
(contar `../`, ler o primeiro segmento) é um vetor de fuga: o
compilador TypeScript aceita `../../src/../01_core/entities`
como caminho válido, mas um analisador léxico pode não reconhecê-lo
e classificar o import como `Layer::Unknown` — efectivamente
tornando-o invisível para V3, V9 e V10.

### 3. Mapeamento de contratos

V11 detecta traits Rust sem `impl`. O equivalente em TypeScript
é `interface` sem `class implements`.

### 4. Novos construtos de tipo em L4

V12 detecta `struct`/`enum`/`impl` em L4. TypeScript tem
`class`, `interface` e `type` como equivalentes.

---

## Decisão

### 1. Formato do header cristalino em TypeScript

O header cristalino em ficheiros `.ts` e `.tsx` usa comentários
de linha (`//`) em bloco contíguo no topo do ficheiro:

```typescript
// Crystalline Lineage
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
```

O bloco termina na primeira linha que não começa com `//` —
mesma semântica do `//!` em Rust.

`/** */` foi rejeitado: ferramentas de documentação (TypeDoc,
ESLint) interpretariam `@prompt` e `@layer` como anotações JSDoc,
gerando falsos positivos em pipelines de documentação.

### 2. Resolução física de camadas — `TsLayerResolver`

**A resolução é física, não léxica.** O texto do import não
determina a camada — o caminho canónico no disco determina.

O algoritmo em L3 (`TsParser`) tem acesso a `file.path` e ao
disco. Para cada import:

**Passo 1 — Resolução de alias:**
Se o import começa com uma chave de `[ts_aliases]` no
`crystalline.toml`, substituir pelo valor correspondente:
```
"@core/entities/layer"  →  "01_core/entities/layer"   (via ts_aliases)
"@infra/walker"         →  "03_infra/walker"           (via ts_aliases)
```
Imports sem alias passam directamente ao passo 2.

**Passo 2 — Álgebra de caminhos:**
Construir o caminho absoluto a partir do directório do ficheiro
actual e da string do import (após resolução de alias):
```rust
let base = file.path.parent().unwrap_or(Path::new("."));
let joined = base.join(import_str);
```

**Passo 3 — Normalização canónica:**
Resolver `.` e `..` algebricamente sem bater no disco,
usando um algoritmo de normalização de path puro:
```rust
fn normalize(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => { components.pop(); }
            Component::CurDir    => {}
            c                    => { components.push(c); }
        }
    }
    components.iter().collect()
}
```
`std::fs::canonicalize` é **evitado** — requer que o ficheiro
exista no disco no momento do parse, o que não é garantido em
CI e introduz I/O desnecessário. A normalização algébrica é
suficiente e determinística.

**Passo 4 — Reutilização de `resolve_file_layer`:**
O caminho normalizado é passado directamente para
`resolve_file_layer(path, root, config)` — a mesma função que
o `FileWalker` usa para resolver a camada de ficheiros
descobertos no disco.

```rust
let normalized = normalize(&base.join(import_str_after_alias));
let target_layer = resolve_file_layer(&normalized, &project_root, &config);
```

**Resultado:** `../../src/../01_core/entities`, `../01_core/entities`
e `@core/entities` resolvem todos para o mesmo caminho canónico
e portanto para a mesma camada. O texto do import é irrelevante.

**Imports de packages npm:**
Se o import não começa com `./`, `../` ou uma chave de
`[ts_aliases]`, é um package npm externo → `Layer::Unknown`
directamente, sem álgebra de paths.

**`target_subdir` para V9:**
Extraído do segmento do path normalizado que corresponde ao
directório mapeado em `[layers]`, usando `strip_prefix` contra
o valor de cada layer. Exemplo: caminho normalizado
`/project/01_core/entities/layer.ts` com `L1 = "01_core"` →
`target_subdir = Some("entities")`.

### 3. Mapeamento de regras V1–V12 para TypeScript

| Regra | Rust | TypeScript | Estado |
|-------|------|------------|--------|
| V1 | `//! @prompt` | `// @prompt` | Adaptado |
| V2 | `#[cfg(test)]` ou `_test.rs` | `describe`/`it`/`test` no AST ou `.test.ts`/`.spec.ts` adjacente | Adaptado |
| V3 | `use crate::L3` | import com path resolvido para L3 | Adaptado via TsLayerResolver físico |
| V4 | `std::fs`, `tokio::io`... | `fs`, `child_process`, `net`... | Adaptado — lista TS |
| V5 | `@prompt-hash` | `@prompt-hash` | Idêntico |
| V6 | `export fn/struct/trait` | `export function/class/interface` | Adaptado |
| V7 | `@prompt` em `.rs` | `@prompt` em `.ts`/`.tsx` | Idêntico |
| V8 | `Layer::Unknown` fora de `[layers]` | Idêntico | Idêntico |
| V9 | `target_subdir` vs `[l1_ports]` | `target_subdir` via TsLayerResolver físico | Adaptado |
| V10 | import com `target_layer == Lab` | import com `target_layer == Lab` | Idêntico via TsLayerResolver físico |
| V11 | `trait` sem `impl` em L2/L3 | `interface` em `contracts/` sem `class implements` em L2/L3 | Adaptado |
| V12 | `struct`/`enum`/`impl` em L4 | `class`/`interface`/`type` em L4 | Adaptado |

**V3, V9, V10 são invioláveis:** a resolução física elimina o
vector de fuga léxico. Qualquer import que resolva fisicamente
para um directório de produção é tratado como import de produção,
independentemente de como o texto está escrito.

**V2 em TypeScript:** `has_test_coverage` é `true` se o AST
contém chamadas a `describe`, `it`, `test` ou `suite` de nível
superior, ou se existe ficheiro `.test.ts` ou `.spec.ts`
adjacente. Ficheiros que exportam apenas `interface` e `type`
são isentos (equivalente ao declaration-only de Rust).

**V4 em TypeScript — símbolos proibidos em L1:**
```
node:fs, node:fs/promises, fs, fs/promises,
node:child_process, child_process,
node:net, net, node:http, http, node:https, https,
node:dgram, node:dns, node:readline,
process.env, Date.now, Math.random
```
Detectados como imports de nível superior ou call expressions
no AST. Packages npm externos (`axios`, `node-fetch`) não são
proibidos por padrão — apenas os módulos Node.js built-in de I/O.

**V6 em TypeScript:** `PublicInterface` extrai:
- `export function` / `export const` com tipo função → `FunctionSignature`
- `export class` → `TypeSignature { kind: TypeKind::Class }`
- `export interface` → `TypeSignature { kind: TypeKind::Interface }`
- `export type` → `TypeSignature { kind: TypeKind::TypeAlias }`
- `export { X }` e `export * from` → `reexports`

**V11 em TypeScript:** `declared_traits` extrai nós
`interface_declaration` com `export` em ficheiros L1/contracts/.
`implemented_traits` extrai o nome da interface da cláusula
`implements` de nós `class_declaration` em ficheiros L2/L3.

**V12 em TypeScript:** declarações proibidas em L4:
- `class_declaration` → `DeclarationKind::Struct` (equivalente funcional)
- `interface_declaration` → `DeclarationKind::Interface`
- `type_alias_declaration` → `DeclarationKind::TypeAlias`

---

## Impacto na IR e nos ficheiros

### `TypeKind` — variantes adicionais
```rust
pub enum TypeKind {
    // existentes — Rust
    Struct, Enum, Trait,
    // novas — TypeScript
    Class, Interface, TypeAlias,
}
```

### `DeclarationKind` — variantes adicionais
```rust
pub enum DeclarationKind {
    // existentes — Rust
    Struct, Enum, Impl,
    // novas — TypeScript
    Interface, TypeAlias,
}
```

### `CrystallineConfig` — campo novo
```rust
pub ts_aliases: HashMap<String, String>,
```

### Novo ficheiro em L3
```
03_infra/ts_parser.rs + test
```

Implementa `LanguageParser` para TypeScript usando
`tree-sitter-typescript`. Estrutura paralela ao `RustParser`:
- `TsLayerResolver` (resolução física, 4 passos)
- `TsSubdirResolver` (via `strip_prefix` após resolução física)
- Extracção de header, imports, tokens, test coverage,
  public interface, declared_traits, implemented_traits,
  declarations

### `Cargo.toml`
```toml
tree-sitter-typescript = "0.21"
```

### `crystalline.toml`
```toml
[languages]
rust       = { grammar = "tree-sitter-rust",       enabled = true }
typescript = { grammar = "tree-sitter-typescript", enabled = true }

[ts_aliases]
# Opcional — apenas se o projecto usa path aliases em tsconfig.json
# "@core"  = "01_core"
# "@shell" = "02_shell"
# "@infra" = "03_infra"
```

---

## Prompts a criar

| Prompt | Conteúdo |
|--------|----------|
| `ts-parser.md` | `TsParser` em L3 — paralelo a `rs-parser.md`, com TsLayerResolver físico documentado |

## Prompts a revisar

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `TypeKind` e `DeclarationKind` com variantes TS |
| `linter-core.md` | TypeScript em `[languages]`, `[ts_aliases]`, `TsParser` no pipeline |
| `rs-parser.md` | Nota sobre `TsParser` como segundo implementador de `LanguageParser` |
| `cargo.md` | `tree-sitter-typescript` adicionado |

---

## Consequências

### ✅ Positivas

- **Resolução física inviolável:** o texto do import é irrelevante.
  `../../src/../01_core/foo`, `../01_core/foo` e `@core/foo`
  resolvem para a mesma camada — V3, V9 e V10 são incontornáveis
- **Reutilização de `resolve_file_layer`:** zero duplicação de
  lógica de resolução entre walker e parser
- **Parity com Rust:** TypeScript atinge o mesmo rigor estrutural
  que `crate::` oferece — a fonte de verdade é o disco, não o texto
- O núcleo L1 não muda — agnósticidade de linguagem do ADR-0001
  validada na prática

### ❌ Negativas

- `TsParser` duplica estruturalmente o `RustParser` em código de
  extracção AST — inevitável porque as gramáticas são diferentes
- Normalização algébrica de paths sem `canonicalize` não detecta
  symlinks — caso de uso raro mas documentado como limitação
- `TypeKind` e `DeclarationKind` crescem com variantes que ficheiros
  Rust nunca produzem

### ⚙️ Neutras

- `tree-sitter-typescript` aumenta o tempo de compilação do linter
  em CI próprio — irrelevante para consumidores de binário
- `[ts_aliases]` é opcional — projectos sem aliases funcionam
  com paths relativos directamente
- `TypeKind::Class/Interface/TypeAlias` é compatível com a
  serialização JSON de snapshots existente

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Resolução léxica (primeiro segmento do path) | Simples, sem I/O | Vector de fuga: `../../src/../01_core/` classifica como `Unknown` |
| `std::fs::canonicalize` em vez de normalização algébrica | Resolve symlinks | Requer que o ficheiro exista no disco; falha em CI com repos parciais ou stubs |
| Parsing de `tsconfig.json` para aliases | Fonte de verdade do ecossistema TS | `tsconfig` ramifica-se, estende outras configs, contém lixo não relacionado; acopla o linter à Microsoft |
| Header `/** @prompt */` em JSDoc | Familiar para devs TS | Colide com JSDoc; `@prompt` e `@layer` seriam interpretados como anotações por TypeDoc e ESLint |
| V11 apenas para Rust | Zero adaptação | Contratos TypeScript ficam sem verificação de circuito fechado |
| Parser TypeScript separado do trait `LanguageParser` | Flexibilidade máxima | Expõe L1 ao conhecimento de linguagens — quebra invariante do ADR-0001 |

---

## Referências

- ADR-0001: Tree-sitter Intermediate Representation
- ADR-0004: Reformulação do Motor de Análise (zero-copy, `resolve_file_layer`)
- ADR-0007: Fechamento Comportamental (V11 — base para adaptação TS)
- `rs-parser.md` — modelo estrutural para `ts-parser.md`
- `violation-types.md` — IR a estender
- `file-walker.md` — `resolve_file_layer` a reutilizar
- `linter-core.md` — pipeline e configuração
