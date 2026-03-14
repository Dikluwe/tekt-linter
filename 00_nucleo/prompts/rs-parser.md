# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004 + Errata Cow)
**Arquivos gerados**:
  - 03_infra/rs_parser.rs + test

---

## Contexto

O núcleo L1 aguarda um `ParsedFile<'a>` completo e agnóstico para
análise. Esta camada L3 faz o trabalho impuro: recebe referência de
`SourceFile`, aciona tree-sitter-rust, e traduz a AST resultante
nos campos exatos que as regras V1–V6 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs`.

Recebe três dependências injetadas via construtor:
- `PromptReader` — para V1 e V5
- `PromptSnapshotReader` — para V6
- `CrystallineConfig` — para resolução de camadas de imports

**Diretivas do ADR-0004:**
- **Zero-Copy**: `parse()` recebe `&'a SourceFile` e retorna
  `ParsedFile<'a>` com referências ao buffer do fonte. Proibido
  `.to_string()` para conteúdo já presente no buffer.
- **Duas Fases (FQN)**: tabela de aliases construída na Fase 1,
  tokens resolvidos para FQN na Fase 2.

**Errata Cow**: `Token.symbol` é `Cow<'a, str>` — não `&'a str`.
FQN construído por concatenação de alias não existe no buffer
original e não pode ser uma referência. Ver tabela de exceções
em `violation-types.md`.

---

## Motor de Duas Fases

### Fase 1 — Tabela de aliases (local ao arquivo)

Varre todos os `use_declaration` da AST antes de processar
qualquer `call_expression`. Constrói `HashMap<&'a str, &'a str>`
local ao arquivo — não compartilhado entre threads:
```rust
// use std::fs as f;     →  aliases["f"]   = "std::fs"
// use tokio::io as tio; →  aliases["tio"] = "tokio::io"
// use tokio::io;        →  aliases["io"]  = "tokio::io"  (último segmento)
// use std::fs;          →  aliases["fs"]  = "std::fs"    (último segmento)
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

## Responsabilidades de extração

Todas as strings extraídas do AST são fatias (`&'a str`) dos bytes
do buffer do `SourceFile`, obtidas via fronteiras de nó do
tree-sitter. Nunca usar `.to_string()` para conteúdo do buffer.

### Header cristalino (V1, V5)

| Campo | Como extrair |
|-------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo. Parar ao primeiro não-`//!`. Field matching sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` — fatias `&'a str` do buffer |
| `prompt_file_exists` | `PromptReader::exists(&header.prompt_path)` — booleano, sem alocação |
| `PromptHeader.current_hash` | `PromptReader::read_hash(&header.prompt_path)` — `Option<String>`, única exceção zero-copy justificada |

### Imports (V3)

| Campo | Como extrair |
|-------|--------------|
| `imports` | Nós `use_declaration` e `extern_crate_declaration`. `path` = fatia `&'a str` do buffer. `target_layer` via `LayerResolver` (segundo segmento de `crate::` contra `config.module_layers`) |

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
| `public_interface.types` | Nós `struct_item`, `enum_item`, `trait_item` com `pub`. `name`, `members` como `&'a str` normalizados |
| `public_interface.reexports` | Nós `use_declaration` com `pub` — re-exports como `&'a str` |
| `prompt_snapshot` | `PromptSnapshotReader::read_snapshot(&header.prompt_path)` — desserializa JSON da seção `## Interface Snapshot` |

### Normalização de tipos (V6)

Strings de tipo são normalizadas antes de criar `FunctionSignature`
e `TypeSignature`:
- Whitespace colapsado — `&mut Vec < String >` → `&mut Vec<String>`
- Comentários removidos
- Lifetimes preservados — fazem parte da assinatura pública

Normalização usa fatias do buffer quando possível. Quando collapse
de whitespace requer nova string, aloca `String` localmente e
armazena via `Cow::Owned` — mas isso só afeta `Token.symbol`.
Para `FunctionSignature` e `TypeSignature`, a normalização é feita
na hora da comparação via helper, não na struct.

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
    pub fn new(prompt_reader: R, snapshot_reader: S, config: CrystallineConfig) -> Self {
        Self { prompt_reader, snapshot_reader, config }
    }
}

impl<R: PromptReader, S: PromptSnapshotReader> LanguageParser for RustParser<R, S> {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        // Fase 1: build_alias_table(&file.content)
        // Fase 2: extract_all_fields(root, &file.content, &aliases)
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
- Fase 1 (aliases) deve preceder Fase 2 (tokens) — ordem incorreta
  produz FQNs errados para aliases
- Tabela de aliases é local ao arquivo — não há estado entre chamadas
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
Então imports contém Import { target_layer: Layer::L2, .. }

Dado SourceFile com use reqwest::Client
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }

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
    members: ["rule_id: String", ...]
}

Dado SourceFile com fn helper() privado
Quando parse() for chamado
Então helper não aparece em public_interface.functions

Dado SourceFile com pub use crate::entities::Layer
Quando parse() for chamado
Então public_interface.reexports contém "crate::entities::Layer"

Dado prompt com seção Interface Snapshot válida
Quando parse() for chamado
Então prompt_snapshot = Some(PublicInterface) desserializada

Dado prompt sem seção Interface Snapshot
Quando parse() for chamado
Então prompt_snapshot = None — V6 não dispara sem baseline

Dado dois SourceFiles com mesma interface mas whitespace diferente
Quando parse() for chamado em ambos
Então public_interface é idêntica — normalização correta

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
| 2026-03-14 | ADR-0004: parse() recebe &'a SourceFile, Motor de Duas Fases documentado, tabela de aliases com regra de último segmento, zero-copy enforcement | rs_parser.rs |
| 2026-03-14 | Errata Cow: Token.symbol é Cow<'a, str> — Borrowed para direto, Owned para alias resolvido | rs_parser.rs |
