# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

O núcleo L1 aguarda um `ParsedFile` completo e agnóstico para
análise. Esta camada L3 faz o trabalho impuro: lê source text,
aciona tree-sitter-rust, e traduz a AST resultante nos campos
exatos que as regras V1–V5 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs`.

---

## Responsabilidades

**Do source text, extrair e popular:**

| Campo ParsedFile | Como extrair |
|------------------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo do arquivo. Parsear `@prompt`, `@prompt-hash`, `@layer`, `@updated` via field matching — não regex sobre o arquivo inteiro |
| `prompt_file_exists` | Delegar para `FsPromptReader::exists(prompt_path)` após extrair o header |
| `PromptHeader.current_hash` | Delegar para `FsPromptReader::read_hash(prompt_path)` |
| `imports` | Nós `use_declaration` e `extern_crate` na AST. Para cada um, extrair `path` e `line`. Resolver `target_layer` via `LayerResolver` (ver abaixo) |
| `tokens` | Nós `call_expression` e `macro_invocation` na AST. Extrair `symbol` completo (path qualificado), `line`, `column` |
| `has_test_coverage` | Verificar presença de nó `attribute_item` com conteúdo `cfg(test)` na AST. Se ausente, delegar ao `FileWalker` via metadado injetado em `SourceFile` |

**`LayerResolver`** — função pura interna a L3:
```rust
fn resolve_layer(import_path: &str, config: &CrystallineConfig) -> Layer {
    // config.layers contém mapeamento "01_core" → Layer::L1 etc.
    // prefix matching sobre import_path
    // retorna Layer::Unknown para crates externas
}
```

Não é uma trait — é uma função auxiliar de L3, não exposta a L1.

---

## Restrições

- Implementa `LanguageParser` — retorna `Result<ParsedFile, ParseError>`
- Erros de gramática tree-sitter viram `ParseError::SyntaxError`
- Linguagem não suportada vira `ParseError::UnsupportedLanguage`
- Não contém nenhuma regra de violação — apenas tradução
- `std::io::Error` nunca atravessa para L1 — absorvido aqui

---

## Critérios de Verificação
```
Dado source Rust com header cristalino completo
Quando parse() for chamado
Então prompt_header está populado com todos os campos

Dado source Rust com import use crate::shell::api
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L2, .. }

Dado source Rust com import use reqwest::Client
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }

Dado source Rust com std::fs::read em call_expression
Quando parse() for chamado
Então tokens contém Token { symbol: "std::fs::read", kind: CallExpression }

Dado source Rust com #[cfg(test)] presente na AST
Quando parse() for chamado
Então has_test_coverage = true

Dado source Rust sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | rs_parser.rs |
| 2025-03-13 | Gap 4: declaradas responsabilidades explícitas de população de ParsedFile, LayerResolver, delegação para FsPromptReader | rs_parser.rs |
