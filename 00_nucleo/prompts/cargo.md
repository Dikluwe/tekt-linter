# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-15 (ADR-0006: target_subdir em Import)
**Arquivos gerados**:
  - 03_infra/rs_parser.rs + test

---

## Contexto

O núcleo L1 aguarda um `ParsedFile<'a>` completo e agnóstico para
análise. Esta camada L3 faz o trabalho impuro: recebe referência de
`SourceFile`, aciona tree-sitter-rust, e traduz a AST resultante
nos campos exatos que as regras V1–V9 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs`.

Recebe três dependências injetadas via construtor:
- `PromptReader` — para V1 e V5
- `PromptSnapshotReader` — para V6
- `CrystallineConfig` — para resolução de camadas e subdirs

**Diretivas do ADR-0004:**
- **Zero-Copy**: `parse()` recebe `&'a SourceFile` e retorna
  `ParsedFile<'a>` com referências ao buffer do fonte.
- **Duas Fases (FQN)**: tabela de aliases construída na Fase 1,
  tokens resolvidos para FQN na Fase 2.

**Errata Cow**: `Token.symbol` é `Cow<'a, str>` — não `&'a str`.

**ADR-0006**: cada `Import` carrega `target_subdir: Option<&'a str>`
resolvido pelo `LayerResolver` contra `config.l1_ports`.

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

### Header cristalino (V1, V5)

| Campo | Como extrair |
|-------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo. Parar ao primeiro não-`//!`. Field matching sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` — fatias `&'a str` do buffer |
| `prompt_file_exists` | `PromptReader::exists(&header.prompt_path)` |
| `PromptHeader.current_hash` | `PromptReader::read_hash(&header.prompt_path)` — `Option<String>` |

### Imports (V3 e V9)

| Campo | Como extrair |
|-------|--------------|
| `imports` | Nós `use_declaration` e `extern_crate_declaration`. `path` = fatia `&'a str` do buffer. `target_layer` via `LayerResolver`. `target_subdir` via `SubdirResolver` (ver abaixo) |

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

---

## LayerResolver e SubdirResolver

Dois resolvers internos a L3 — funções puras, não expostas a L1:

### `LayerResolver`
```rust
fn resolve_layer(import_path: &str, config: &CrystallineConfig) -> Layer {
    // Inspeciona segundo segmento de paths crate:: ou super::
    // "crate::entities::layer" → L1 (via config.module_layers["entities"])
    // "reqwest::Client" → Layer::Unknown (crate externa)
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
    // Só resolve subdirs de L1 — outras camadas retornam None
    if *target_layer != Layer::L1 {
        return None;
    }
    // Extrai segundo segmento de paths crate:: ou super::
    // "crate::entities::Layer" → Some("entities")
    // "crate::contracts::FileProvider" → Some("contracts")
    // "crate::internal::helper" → Some("internal")
    // "reqwest::Client" → None (crate externa, target_layer já é Unknown)
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
        // SubdirResolver aplicado em cada Import após LayerResolver
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
— subdir resolvido para imports de L1

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
