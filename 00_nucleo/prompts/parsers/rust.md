# Prompt: Rust Parser (parsers/rust)

> **Nota de localização:** Este prompt foi movido de `prompts/rs-parser.md`
> para `prompts/parsers/rust.md` pelo ADR-0009. É a implementação de
> referência do contrato definido em `prompts/parsers/_template.md`.
> O ficheiro `03_infra/rs_parser.rs` aponta para este caminho via
> `@prompt 00_nucleo/prompts/parsers/rust.md`.

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-18 (ADR-0009: movido para parsers/, referência ao template)
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

**ADR-0009**: `rs_parser.rs` é o parser de referência para Rust.
A resolução de camadas em Rust usa `LayerResolver` baseado em
`crate::` — absoluto por construção, sem vector de fuga léxico.
Parsers de outras linguagens usam resolução física via `normalize`
+ `resolve_file_layer` conforme documentado em `_template.md`.
Rust não precisa deste mecanismo porque `crate::` já é absoluto.

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

Regra para último segmento sem `as`: extrair o último componente
do path e mapear para o path completo. Isso garante que
`io::stdin()` seja resolvido mesmo sem alias explícito.

### Fase 2 — Extração de tokens com resolução de FQN

Para cada `call_expression` encontrado:
```rust
// 1. Extrair prefixo da chamada
// 2. Verificar na tabela de aliases
// 3a. Se encontrado → Cow::Owned(format!("{}::{}", fqn_base, suffix))
// 3b. Se não encontrado → Cow::Borrowed(&source[node.start..node.end])

// Exemplos:
// f::read(...)      + aliases["f"]="std::fs"   → Owned("std::fs::read")
// io::stdin()       + aliases["io"]="tokio::io" → Owned("tokio::io::stdin")
// std::fs::write()  sem alias                   → Borrowed("std::fs::write")
// my_fn()           sem alias, não proibido      → Borrowed("my_fn")
```

A tabela é descartada após processar o arquivo. Zero estado
global — paralelismo via rayon é seguro.

---

## Responsabilidades de extracção

Todas as strings extraídas do AST são fatias (`&'a str`) dos bytes
do buffer do `SourceFile`, obtidas via fronteiras de nó do
tree-sitter. Nunca usar `.to_string()` para conteúdo do buffer.

### Header cristalino (V1, V5)

| Campo | Como extrair |
|-------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo. Parar ao primeiro não-`//!`. Field matching sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` — fatias `&'a str` do buffer |
| `prompt_file_exists` | `PromptReader::exists(&header.prompt_path)` — booleano, sem alocação |
| `PromptHeader.current_hash` | `PromptReader::read_hash(&header.prompt_path)` — `Option<String>`, única exceção zero-copy justificada |

### Imports (V3, V9, V10)

| Campo | Como extrair |
|-------|--------------|
| `imports` | Nós `use_declaration` e `extern_crate_declaration`. `path` = fatia `&'a str` do buffer. `target_layer` via `LayerResolver` (crate:: é absoluto — sem normalização física necessária). `target_subdir` via `SubdirResolver`. `Layer::Lab` resolvido para imports de `lab/` — V10 usa este valor |

### Tokens (V4)

| Campo | Como extrair |
|-------|--------------|
| `tokens` | Nós `call_expression` e `macro_invocation` submetidos ao Motor de Duas Fases. `symbol` = `Cow<'a, str>` — Borrowed se direto, Owned se alias resolvido |

### Test coverage (V2)

| Campo | Como extrair |
|-------|--------------|
| `has_test_coverage` | Nó `attribute_item` com `cfg(test)` presente na AST → `true`. Senão, usar `source_file.has_adjacent_test`. Senão, verificar se arquivo é declaration-only (sem `impl` com corpo) → `true` (isento) |

### Interface pública (V6)

| Campo | Como extrair |
|-------|--------------|
| `public_interface.functions` | Nós `function_item` com modificador `pub`. `name`, `params`, `return_type` como `&'a str` normalizados (whitespace colapsado, comentários removidos) |
| `public_interface.types` | Nós `struct_item`, `enum_item`, `trait_item` com `pub`. `name`, `members` como `&'a str` normalizados. `TypeKind::Struct/Enum/Trait` conforme o nó |
| `public_interface.reexports` | Nós `use_declaration` com `pub` — re-exports como `&'a str` |
| `prompt_snapshot` | `PromptSnapshotReader::read_snapshot(&header.prompt_path)` — desserializa JSON da seção `## Interface Snapshot` |

### Normalização de tipos (V6)

Strings de tipo são normalizadas antes de criar `FunctionSignature`
e `TypeSignature`:
- Whitespace colapsado — `&mut Vec < String >` → `&mut Vec<String>`
- Comentários removidos
- Lifetimes preservados — fazem parte da assinatura pública

Normalização usa fatias do buffer quando possível. Quando collapse
de whitespace requer nova string, aloca `String` localmente —
mas isso só afeta campos de tipo internos, não `Token.symbol`.

### Traits declaradas (V11) — `declared_traits`

Apenas quando `file.layer == Layer::L1` e path contém `"contracts"`.

Para cada nó `trait_item` de nível superior com modificador `pub`:
- Extrair campo `name` como `&'a str`
- Adicionar a `declared_traits`

```
pub trait FileProvider { ... }   →  declared_traits = ["FileProvider"]
trait InternalHelper { ... }     →  ignorado (sem pub)
```

Ficheiros em L1 fora de `contracts/` (ex: `entities/`, `rules/`)
não contribuem para `declared_traits`.

### Traits implementadas (V11) — `implemented_traits`

Apenas quando `file.layer == Layer::L2 | Layer::L3`.

Para cada nó `impl_item` de nível superior com campo `trait`:
- Extrair nome simples da trait — último segmento se for path
  (`crate::contracts::FileProvider` → `"FileProvider"`)
- Adicionar a `implemented_traits`

```
impl FileProvider for FileWalker { ... }         →  ["FileProvider"]
impl LanguageParser for RustParser<R, S> { ... } →  ["FileProvider", "LanguageParser"]
impl FileWalker { ... }  // sem trait             →  ignorado aqui
```

Ficheiros em L1 ou L4 não contribuem para `implemented_traits`.

### Declarações de tipo (V12) — `declarations`

Para todos os arquivos, sem filtro de layer.
V12 filtra por `layer == L4` internamente — o parser não filtra.

| Nó | `DeclarationKind` | Condição |
|----|------------------|----------|
| `struct_item` | `Struct` | sempre capturado |
| `enum_item` | `Enum` | sempre capturado |
| `impl_item` sem campo `trait` | `Impl` | `impl Type { ... }` |
| `impl_item` com campo `trait` | **não capturado** | `impl Trait for Type` — adapter, permitido em L4 |

A distinção entre `impl_item` com e sem trait é feita verificando
se o nó tree-sitter tem o campo `trait` definido.

---

## LayerResolver e SubdirResolver

Dois resolvers internos a L3 — funções puras, não expostas a L1:

### `LayerResolver`
```rust
fn resolve_layer(import_path: &str, config: &CrystallineConfig) -> Layer {
    // Inspeciona segundo segmento de paths crate:: ou super::
    let segments: Vec<&str> = import_path.splitn(4, "::").collect();
    if segments[0] != "crate" && segments[0] != "super" {
        return Layer::Unknown;
    }
    segments.get(1)
        .map(|module| config.layer_for_module(module))
        .unwrap_or(Layer::Unknown)
}
```

`crate::` é absoluto — não requer normalização física.
Rust não tem o vector de fuga de paths relativos que afecta
linguagens como TypeScript. Ver `_template.md` para o algoritmo
de resolução física obrigatório em outras linguagens.

### `SubdirResolver` (ADR-0006)
```rust
fn resolve_subdir<'a>(
    import_path: &'a str,
    target_layer: &Layer,
    config: &CrystallineConfig,
) -> Option<&'a str> {
    // Só resolve subdirs de L1 — outras camadas retornam None
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
        // Ordem de extracção:
        // 1. header (prompt_header, prompt_file_exists, current_hash)
        // 2. imports (LayerResolver + SubdirResolver)
        // 3. tokens (Motor de Duas Fases — Fase 1 aliases, Fase 2 FQN)
        // 4. has_test_coverage (cfg(test) + adjacência + declaration-only)
        // 5. public_interface + prompt_snapshot (V6)
        // 6. declared_traits (apenas L1/contracts) (V11)
        // 7. implemented_traits (apenas L2|L3) (V11)
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
- Proibido `.to_string()` para strings presentes no buffer —
  apenas as exceções documentadas em `violation-types.md`
- `PromptHeader.current_hash` é a única `String` alocada —
  exceção documentada
- Fase 1 (aliases) deve preceder Fase 2 (tokens) — ordem incorreta
  produz FQNs errados para aliases
- Tabela de aliases é local ao arquivo — não há estado entre chamadas
- `SubdirResolver` retorna `None` para camadas que não sejam L1
- `target_subdir` resolvido para todos os imports de L1,
  incluindo subdirs não listados em `[l1_ports]` — V9 decide
- `declared_traits` apenas em L1/contracts/ — filtragem no parser,
  não em V11
- `implemented_traits` apenas em L2|L3 — filtragem no parser,
  não em V11
- `declarations` para todos os arquivos — V12 filtra por layer
- `impl Trait for Type` não é capturado em `declarations`
- `PromptReader` e `PromptSnapshotReader` são injetados —
  o parser nunca os instancia diretamente
- Erros de `std::io` nunca atravessam para L1 — convertidos
  em `ParseError` antes de retornar

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
— último segmento sem alias explícito também é resolvido

Dado SourceFile com std::fs::write(...) sem nenhum alias
Quando parse() for chamado
Então tokens contém Token { symbol: Cow::Borrowed("std::fs::write"), .. }
— FQN direto usa referência ao buffer

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
— crate externa: target_layer Unknown, target_subdir None

Dado SourceFile em L1, subdir = "contracts", com:
  pub trait FileProvider { ... }
  pub trait LanguageParser { ... }
  trait InternalHelper { ... }
Quando parse() for chamado
Então declared_traits = ["FileProvider", "LanguageParser"]
E "InternalHelper" não aparece em declared_traits
— apenas traits públicas de L1/contracts/

Dado SourceFile em L1, subdir = "rules"
Com pub trait HasImports { ... }
Quando parse() for chamado
Então declared_traits = []
— L1/rules não é subdir de contratos

Dado SourceFile em L3 com:
  impl FileProvider for FileWalker { ... }
  impl LanguageParser for RustParser<R, S> { ... }
  impl FileWalker { ... }
Quando parse() for chamado
Então implemented_traits = ["FileProvider", "LanguageParser"]
E "FileWalker" não aparece em implemented_traits
— impl sem trait não é registado aqui

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
  Declaration { kind: Impl,   name: "L3HashRewriter", .. }
  Declaration { kind: Enum,   name: "OutputMode", .. }
E NÃO contém Declaration para "HashRewriter for L3HashRewriter"
— impl com trait não é capturado

Dado SourceFile em L3 com struct FileWalker { ... }
Quando parse() for chamado
Então declarations contém Declaration { kind: Struct, name: "FileWalker", .. }
— declarations extraído para todos os arquivos sem filtro de layer

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

Dado SourceFile com pub struct Violation { pub rule_id: String, .. }
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "Violation",
    kind: TypeKind::Struct,
    members: ["rule_id", ...]
}

Dado SourceFile com fn helper() privado
Quando parse() for chamado
Então helper não aparece em public_interface.functions

Dado SourceFile com pub use crate::entities::Layer
Quando parse() for chamado
Então public_interface.reexports contém "crate::entities::Layer"

Dado dois SourceFiles com mesma interface mas whitespace diferente:
  pub fn check( file : &ParsedFile ) -> Vec<Violation>
  pub fn check(file: &ParsedFile) -> Vec<Violation>
Quando parse() for chamado em ambos
Então public_interface é idêntica — normalização correcta

Dado prompt com seção Interface Snapshot válida
Quando parse() for chamado
Então prompt_snapshot = Some(PublicInterface) desserializada

Dado prompt sem seção Interface Snapshot
Quando parse() for chamado
Então prompt_snapshot = None — V6 não dispara sem baseline

Dado SourceFile sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado SourceFile vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })

Dado language = TypeScript num RustParser
Quando parse() for chamado
Então retorna Err(ParseError::UnsupportedLanguage { .. })

Dado NullPromptReader e NullSnapshotReader como mocks
Quando parse() for chamado
Então nenhum acesso a disco ocorre durante testes de L1
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
| 2026-03-15 | ADR-0006: SubdirResolver, Import.target_subdir | rs_parser.rs |
| 2026-03-16 | ADR-0007: declared_traits, implemented_traits, declarations; ordem de extracção documentada | rs_parser.rs |
| 2026-03-18 | ADR-0009: movido de rs-parser.md para parsers/rust.md; nota de referência ao _template.md; nota sobre LayerResolver Rust vs resolução física de outras linguagens; TypeKind::Struct/Enum/Trait explicitados na secção de interface pública | rs_parser.rs |
| 2026-03-19 | Passo 1: extract_type_sig actualizado para cobrir TypeKind::Class/Interface/TypeAlias (arm de retorno vazio — RustParser não emite estes kinds, mas exaustividade é obrigatória) | rs_parser.rs |
