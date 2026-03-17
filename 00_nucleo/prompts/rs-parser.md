# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-16 (ADR-0007: declared_traits, implemented_traits, declarations)
**Arquivos gerados**:
  - 03_infra/rs_parser.rs + test

---

## Contexto

O núcleo L1 aguarda um `ParsedFile<'a>` completo e agnóstico para
análise. Esta camada L3 faz o trabalho impuro: recebe referência de
`SourceFile`, aciona tree-sitter-rust, e traduz a AST resultante
nos campos exatos que as regras V1–V12 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs`.

Recebe três dependências injetadas via construtor:
- `PromptReader` — para V1 e V5
- `PromptSnapshotReader` — para V6
- `CrystallineConfig` — para resolução de camadas, subdirs e
  configuração de wiring

**Diretivas do ADR-0004:**
- **Zero-Copy**: `parse()` recebe `&'a SourceFile` e retorna
  `ParsedFile<'a>` com referências ao buffer do fonte.
- **Duas Fases (FQN)**: tabela de aliases construída na Fase 1,
  tokens resolvidos para FQN na Fase 2.

**Errata Cow**: `Token.symbol` é `Cow<'a, str>` — não `&'a str`.

**ADR-0006**: cada `Import` carrega `target_subdir: Option<&'a str>`
resolvido pelo `LayerResolver` contra `config.l1_ports`.

**ADR-0007**: `ParsedFile` ganha três campos novos populados aqui:
- `declared_traits` — traits públicas em L1/contracts/
- `implemented_traits` — traits implementadas via `impl Trait for` em L2/L3
- `declarations` — struct/enum/impl-sem-trait de nível superior, para V12

---

## Motor de Duas Fases

### Fase 1 — Tabela de aliases (local ao arquivo)

Varre todos os `use_declaration` da AST antes de processar
qualquer `call_expression`. Constrói `HashMap<&'a str, &'a str>`
local ao arquivo — não compartilhado entre threads:
```
use std::fs as f;     →  aliases["f"]   = "std::fs"
use tokio::io as tio; →  aliases["tio"] = "tokio::io"
use tokio::io;        →  aliases["io"]  = "tokio::io"  (último segmento)
use std::fs;          →  aliases["fs"]  = "std::fs"    (último segmento)
```

### Fase 2 — Extração de tokens com resolução de FQN

Para cada `call_expression`:
```
f::read(...)      + aliases["f"]="std::fs"   → Owned("std::fs::read")
io::stdin()       + aliases["io"]="tokio::io" → Owned("tokio::io::stdin")
std::fs::write()  sem alias                   → Borrowed("std::fs::write")
my_fn()           sem alias, não proibido      → Borrowed("my_fn")
```

A tabela é descartada após processar o arquivo. Zero estado
global — paralelismo via rayon é seguro.

---

## Responsabilidades de extração

Todas as strings extraídas do AST são fatias (`&'a str`) dos bytes
do buffer do `SourceFile`, obtidas via fronteiras de nó do
tree-sitter. Nunca usar `.to_string()` para conteúdo do buffer.

### Header cristalino (V1, V5)

| Campo | Como extrair |
|-------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo. Parar ao primeiro não-`//!`. Field matching sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` — fatias `&'a str` do buffer |
| `prompt_file_exists` | `PromptReader::exists(&header.prompt_path)` |
| `PromptHeader.current_hash` | `PromptReader::read_hash(&header.prompt_path)` — `Option<String>` |

### Imports (V3, V9, V10)

| Campo | Como extrair |
|-------|--------------|
| `imports` | Nós `use_declaration` e `extern_crate_declaration`. `path` = fatia `&'a str` do buffer. `target_layer` via `LayerResolver`. `target_subdir` via `SubdirResolver`. `Layer::Lab` é resolvido para imports do diretório `lab/` — V10 usa este valor |

### Tokens (V4)

| Campo | Como extrair |
|-------|--------------|
| `tokens` | Nós `call_expression` e `macro_invocation` submetidos ao Motor de Duas Fases. `symbol` = `Cow<'a, str>` |

### Test coverage (V2)

| Campo | Como extrair |
|-------|--------------|
| `has_test_coverage` | Nó `attribute_item` com `cfg(test)` → `true`. Senão, `source_file.has_adjacent_test`. Senão, declaration-only → `true` (isento) |

### Interface pública (V6)

| Campo | Como extrair |
|-------|--------------|
| `public_interface.functions` | Nós `function_item` com `pub`. `name`, `params`, `return_type` como `&'a str` normalizados |
| `public_interface.types` | Nós `struct_item`, `enum_item`, `trait_item` com `pub`. `name`, `members` normalizados |
| `public_interface.reexports` | Nós `use_declaration` com `pub` |
| `prompt_snapshot` | `PromptSnapshotReader::read_snapshot(&header.prompt_path)` |

### Traits declaradas (V11) — `declared_traits`

Extraído **apenas** quando `file.layer == Layer::L1` e
`file.subdir == "contracts"` (segundo segmento do path relativo
à raiz do projeto).

Para cada nó `trait_item` de nível superior com modificador `pub`:
- Extrair o campo `name` como `&'a str` do buffer
- Adicionar a `declared_traits`

```
// Exemplo de arquivo 01_core/contracts/file_provider.rs:
pub trait FileProvider { ... }   →  declared_traits = ["FileProvider"]
trait InternalHelper { ... }     →  ignorado (sem pub)
pub trait PromptReader { ... }   →  declared_traits = ["FileProvider", "PromptReader"]
```

Arquivos em L1 fora de `contracts/` (ex: `entities/`, `rules/`)
não contribuem para `declared_traits` — V11 cobre apenas contratos
declarados como portas explícitas.

### Traits implementadas (V11) — `implemented_traits`

Extraído **apenas** quando `file.layer == Layer::L2 | Layer::L3`.

Para cada nó `impl_item` de nível superior que seja
`impl <TraitName> for <Type>` (com campo `trait` presente no nó):
- Extrair o nome da trait — último segmento se for path
  (`crate::contracts::FileProvider` → `"FileProvider"`)
- Adicionar a `implemented_traits`

```
// Exemplo de arquivo 03_infra/walker.rs:
impl FileProvider for FileWalker { ... }
  →  implemented_traits = ["FileProvider"]

impl FileWalker { ... }           // sem trait — ignorado aqui,
                                  // capturado em declarations (V12)

impl LanguageParser for RustParser<R, S> { ... }
  →  implemented_traits = ["FileProvider", "LanguageParser"]
```

Arquivos em L1 ou L4 não contribuem para `implemented_traits` —
V11 verifica se contratos de L1 têm implementação em L2 ou L3.

### Declarações de tipo (V12) — `declarations`

Extraído para **todos** os arquivos, independente de layer.
V12 filtra por `layer == L4` internamente — o parser não filtra.

Para cada nó de nível superior do AST:

| Nó | `DeclarationKind` | Condição |
|----|------------------|----------|
| `struct_item` | `Struct` | sempre capturado |
| `enum_item` | `Enum` | sempre capturado |
| `impl_item` sem campo `trait` | `Impl` | `impl Type { ... }` sem trait |
| `impl_item` com campo `trait` | **não capturado** | `impl Trait for Type` é adapter — permitido em L4 |

```
// Exemplo de arquivo 04_wiring/main.rs:
struct L3HashRewriter { ... }
  →  Declaration { kind: Struct, name: "L3HashRewriter", line: N }

impl L3HashRewriter { ... }       // impl sem trait
  →  Declaration { kind: Impl, name: "L3HashRewriter", line: N }

impl HashRewriter for L3HashRewriter { ... }  // impl com trait
  →  NÃO capturado — é padrão de adapter

enum OutputMode { ... }
  →  Declaration { kind: Enum, name: "OutputMode", line: N }
```

A distinção entre `impl_item` com e sem trait é feita verificando
se o nó tree-sitter tem o campo `trait` definido. Se sim, é
`impl Trait for Type` — não capturado. Se não, é `impl Type { ... }`
— capturado como `DeclarationKind::Impl`.

---

## LayerResolver e SubdirResolver

Dois resolvers internos a L3 — funções puras, não expostas a L1:

### `LayerResolver`
```rust
fn resolve_layer(import_path: &str, config: &CrystallineConfig) -> Layer {
    let segments: Vec<&str> = import_path.splitn(4, "::").collect();
    if segments[0] != "crate" && segments[0] != "super" {
        return Layer::Unknown;
    }
    segments.get(1)
        .map(|module| config.layer_for_module(module))
        .unwrap_or(Layer::Unknown)
}
```

### `SubdirResolver` (ADR-0006)
```rust
fn resolve_subdir<'a>(
    import_path: &'a str,
    target_layer: &Layer,
    config: &CrystallineConfig,
) -> Option<&'a str> {
    if *target_layer != Layer::L1 {
        return None;
    }
    let segments: Vec<&str> = import_path.splitn(4, "::").collect();
    if segments.get(0).copied() != Some("crate")
        && segments.get(0).copied() != Some("super") {
        return None;
    }
    segments.get(1).copied()
}
```

`target_subdir` é `Some(subdir)` para imports de L1 —
independentemente de o subdir estar ou não em `[l1_ports]`.
V9 em L1 decide se o subdir é válido comparando com `L1Ports`.

---

## Assinatura do construtor
```rust
pub struct RustParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
}

impl<R: PromptReader, S: PromptSnapshotReader> RustParser<R, S> {
    pub fn new(
        prompt_reader: R,
        snapshot_reader: S,
        config: CrystallineConfig,
    ) -> Self {
        Self { prompt_reader, snapshot_reader, config }
    }
}

impl<R, S> LanguageParser for RustParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        // Fase 1: build_alias_table(&file.content)
        // Fase 2: extract_all_fields(root, &file.content, &aliases)
        //
        // Ordem de extração:
        // 1. header (prompt_header, prompt_file_exists, current_hash)
        // 2. imports (com LayerResolver + SubdirResolver)
        // 3. tokens (Motor de Duas Fases)
        // 4. has_test_coverage (cfg(test) + adjacência + declaration-only)
        // 5. public_interface + prompt_snapshot (V6)
        // 6. declared_traits (apenas se L1/contracts) (V11)
        // 7. implemented_traits (apenas se L2|L3) (V11)
        // 8. declarations — nível superior struct/enum/impl-sem-trait (V12)
        //
        // SubdirResolver aplicado em cada Import após LayerResolver.
        // declared_traits e implemented_traits condicionados por layer e subdir.
        // declarations extraídas para todos os arquivos sem filtro de layer.
        //
        // Retorna ParsedFile<'a> com todas as referências apontando
        // para file.content
    }
}
```

---

## Restrições

- `parse()` recebe `&'a SourceFile` — proibido consumir ownership
- Proibido `.to_string()` para strings presentes no buffer
- `PromptHeader.current_hash` é a única `String` alocada —
  exceção documentada em `violation-types.md`
- Fase 1 (aliases) deve preceder Fase 2 (tokens)
- `SubdirResolver` retorna `None` para camadas que não sejam L1
- `target_subdir` é resolvido para todos os imports de L1,
  incluindo subdirs não listados em `[l1_ports]` — V9 decide
- `declared_traits` extraídas apenas em L1/contracts/ —
  filtragem por layer e subdir feita pelo parser, não por V11
- `implemented_traits` extraídas apenas em L2 e L3 —
  filtragem por layer feita pelo parser, não por V11
- `declarations` extraídas para todos os arquivos —
  filtragem por layer feita por V12, não pelo parser
- `impl Trait for Type` não é capturado em `declarations` —
  apenas `impl Type { ... }` sem trait é `DeclarationKind::Impl`
- `PromptReader` e `PromptSnapshotReader` são injetados —
  o parser nunca os instancia diretamente

---

## Critérios de Verificação
```
Dado SourceFile com header cristalino completo
Quando parse() for chamado
Então prompt_header populado com todos os campos como &'a str

Dado SourceFile com use std::fs as f; e f::read(...)
Quando parse() for chamado
Então tokens contém Token { symbol: Cow::Owned("std::fs::read"), .. }

Dado SourceFile com use tokio::io; e io::stdin()
Quando parse() for chamado
Então tokens contém Token { symbol: Cow::Owned("tokio::io::stdin"), .. }
— último segmento sem alias também é resolvido

Dado SourceFile com std::fs::write(...) sem nenhum alias
Quando parse() for chamado
Então tokens contém Token { symbol: Cow::Borrowed("std::fs::write"), .. }

Dado SourceFile com use crate::shell::api
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L2, target_subdir: None, .. }
— target_subdir é None para camadas que não L1

Dado SourceFile com use crate::entities::Layer
Quando parse() for chamado
Então imports contém Import {
    target_layer: Layer::L1,
    target_subdir: Some("entities"),
    ..
}

Dado SourceFile com use crate::contracts::FileProvider
Quando parse() for chamado
Então imports contém Import {
    target_layer: Layer::L1,
    target_subdir: Some("contracts"),
    ..
}

Dado SourceFile com use crate::internal::helper
(onde "internal" não está em [l1_ports])
Quando parse() for chamado
Então imports contém Import {
    target_layer: Layer::L1,
    target_subdir: Some("internal"),
    ..
}
— parser não julga se é porta válida, apenas resolve o subdir

Dado SourceFile com use reqwest::Client
Quando parse() for chamado
Então imports contém Import {
    target_layer: Layer::Unknown,
    target_subdir: None,
    ..
}

Dado SourceFile em L1, subdir = "contracts", com:
  pub trait FileProvider { ... }
  pub trait LanguageParser { ... }
  trait InternalHelper { ... }   // sem pub
Quando parse() for chamado
Então declared_traits = ["FileProvider", "LanguageParser"]
E "InternalHelper" não aparece em declared_traits
— apenas traits públicas de L1/contracts/

Dado SourceFile em L1, subdir = "rules"
Com pub trait HasImports { ... }
Quando parse() for chamado
Então declared_traits = []
— L1/rules não é subdir de contratos

Dado SourceFile em L2 com:
  impl FileProvider for MockProvider { ... }
  impl LanguageParser for RustParser<R, S> { ... }
  impl RustParser<R, S> { ... }   // sem trait
Quando parse() for chamado
Então implemented_traits = ["FileProvider", "LanguageParser"]
E "RustParser" não aparece em implemented_traits
— impl sem trait não é registered aqui (vai para declarations)

Dado SourceFile em L3 com:
  impl PromptReader for FsPromptReader { ... }
Quando parse() for chamado
Então implemented_traits = ["PromptReader"]

Dado SourceFile em L1 com impl HasImports for ParsedFile { ... }
Quando parse() for chamado
Então implemented_traits = []
— L1 não contribui para implemented_traits

Dado SourceFile em L4 com:
  struct L3HashRewriter { ... }
  impl L3HashRewriter { ... }
  impl HashRewriter for L3HashRewriter { ... }
  enum OutputMode { Text, Sarif }
Quando parse() for chamado
Então declarations contém:
  Declaration { kind: Struct, name: "L3HashRewriter", .. }
  Declaration { kind: Impl, name: "L3HashRewriter", .. }
  Declaration { kind: Enum, name: "OutputMode", .. }
E NÃO contém Declaration para "HashRewriter for L3HashRewriter"
— impl com trait não é capturado

Dado SourceFile em L3 com struct FileWalker { ... }
Quando parse() for chamado
Então declarations contém Declaration { kind: Struct, name: "FileWalker", .. }
— declarations é extraído para todos os arquivos sem filtro de layer
E V12 filtra por layer == L4 internamente

Dado SourceFile com #[cfg(test)] na AST
Quando parse() for chamado
Então has_test_coverage = true

Dado SourceFile com apenas pub trait Foo { fn bar(&self); }
Quando parse() for chamado
Então has_test_coverage = true — declaration-only, isento de V2

Dado SourceFile com has_adjacent_test = true e sem #[cfg(test)]
Quando parse() for chamado
Então has_test_coverage = true — fallback para adjacência

Dado SourceFile com pub fn check(file: &ParsedFile) -> Vec<Violation>
Quando parse() for chamado
Então public_interface.functions contém FunctionSignature {
    name: "check",
    params: ["&ParsedFile"],
    return_type: Some("Vec<Violation>")
}

Dado SourceFile com fn helper() privado
Quando parse() for chamado
Então helper não aparece em public_interface.functions

Dado prompt com seção Interface Snapshot válida
Quando parse() for chamado
Então prompt_snapshot = Some(PublicInterface) desserializada

Dado prompt sem seção Interface Snapshot
Quando parse() for chamado
Então prompt_snapshot = None

Dado SourceFile sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado SourceFile vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | rs_parser.rs |
| 2025-03-13 | Gap 4: responsabilidades explícitas, LayerResolver, FsPromptReader | rs_parser.rs |
| 2025-03-13 | V6: PublicInterface, PromptSnapshotReader, pipeline explícito | rs_parser.rs |
| 2026-03-14 | ADR-0004: parse() recebe &'a SourceFile, Motor de Duas Fases, zero-copy | rs_parser.rs |
| 2026-03-14 | Errata Cow: Token.symbol é Cow<'a, str> | rs_parser.rs |
| 2026-03-15 | ADR-0006: SubdirResolver adicionado, Import.target_subdir resolvido para todos os imports de L1, critérios de target_subdir adicionados | rs_parser.rs |
| 2026-03-16 | ADR-0007: declared_traits (L1/contracts, apenas pub trait_item), implemented_traits (L2|L3, apenas impl com trait), declarations (todos os arquivos, struct/enum/impl-sem-trait de nível superior); ordem de extração documentada no construtor; restrições e critérios adicionados | rs_parser.rs |
| 2026-03-16 | Materialização ADR-0007: path_contains_segment e trait_last_segment adicionados; extract_declared_traits, extract_implemented_traits e extract_declarations implementados; parse() conectado às três extrações; 12 novos testes | rs_parser.rs |
