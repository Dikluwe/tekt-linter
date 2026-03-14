# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-rust`
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

---

## Contexto

O núcleo L1 aguarda um `ParsedFile` completo e agnóstico para
análise. Esta camada L3 faz o trabalho impuro: lê source text,
aciona tree-sitter-rust, e traduz a AST resultante nos campos
exatos que as regras V1–V6 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs`.

Recebe dois leitores injetados via construtor:
- `PromptReader` — para V1 e V5 (existência e hash do prompt)
- `PromptSnapshotReader` — para V6 (snapshot de interface do prompt)

---

## Responsabilidades

Do source text, extrair e popular todos os campos de `ParsedFile`:

### Header cristalino (para V1, V5)

| Campo | Como extrair |
|-------|--------------|
| `prompt_header` | Primeira sequência de linhas `//!` no topo do arquivo. Field matching sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` — não regex sobre o arquivo inteiro. Para ao primeiro não-`//!` |
| `prompt_file_exists` | Delegar para `PromptReader::exists(prompt_path)` após extrair o header |
| `PromptHeader.current_hash` | Delegar para `PromptReader::read_hash(prompt_path)` |

### Imports (para V3)

| Campo | Como extrair |
|-------|--------------|
| `imports` | Nós `use_declaration` e `extern_crate_declaration` na AST. Para cada um: extrair `path`, `line`, `kind`. Resolver `target_layer` via `LayerResolver` (ver abaixo) |

### Tokens (para V4)

| Campo | Como extrair |
|-------|--------------|
| `tokens` | Nós `call_expression` e `macro_invocation` na AST. Extrair `symbol` completo (path qualificado), `line`, `column`, `kind` |

### Test coverage (para V2)

| Campo | Como extrair |
|-------|--------------|
| `has_test_coverage` | Verificar presença de nó `attribute_item` com conteúdo `cfg(test)` na AST. Se ausente, usar `SourceFile.has_adjacent_test`. Se arquivo é declaration-only (sem `impl` com corpo), marcar como `true` (isento) |

### Interface pública (para V6)

| Campo | Como extrair |
|-------|--------------|
| `public_interface.functions` | Nós `function_item` com modificador `pub` na AST. Para cada um: extrair `name`, tipos dos `parameters` normalizados, `return_type` normalizado |
| `public_interface.types` | Nós `struct_item`, `enum_item`, `trait_item` com modificador `pub`. Para cada um: extrair `name`, `kind`, `members` (campos de struct, variantes de enum, assinaturas de método de trait) |
| `public_interface.reexports` | Nós `use_declaration` com modificador `pub` — re-exports visíveis externamente |
| `prompt_snapshot` | Delegar para `PromptSnapshotReader::read_snapshot(prompt_path)` após extrair o header. None se prompt não tem snapshot ou não existe |

---

## LayerResolver

Função pura interna a L3 — não exposta a L1:
```rust
fn resolve_layer(import_path: &str, config: &CrystallineConfig) -> Layer {
    // Inspeciona apenas o segundo segmento de paths crate:: ou super::
    // "crate::entities::layer" → L1 (via config.module_layers["entities"])
    // "reqwest::Client" → Layer::Unknown (crate externa)
    // Retorna Layer::Unknown para qualquer path não reconhecido
}
```

---

## Normalização de tipos (para V6)

Tipos extraídos do AST são normalizados antes de serem armazenados
em `FunctionSignature.params`, `FunctionSignature.return_type`, e
`TypeSignature.members`:

- Whitespace colapsado — `&mut Vec < String >` → `&mut Vec<String>`
- Comentários removidos
- Lifetimes preservados — fazem parte da assinatura pública

Isso garante que reformatação de código não dispara V6 — apenas
mudanças semânticas na interface.

---

## Restrições

- Implementa `LanguageParser` — retorna `Result<ParsedFile, ParseError>`
- Erros de gramática tree-sitter viram `ParseError::SyntaxError`
- Linguagem não suportada vira `ParseError::UnsupportedLanguage`
- Arquivo vazio vira `ParseError::EmptySource`
- Não contém nenhuma regra de violação — apenas tradução
- `std::io::Error` nunca atravessa para L1 — absorvido aqui
- `PromptReader` e `PromptSnapshotReader` são injetados —
  o parser nunca os instancia diretamente

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
```

---

## Pipeline de parse
```rust
impl<R, S> LanguageParser for RustParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    fn parse(&self, file: SourceFile) -> Result<ParsedFile, ParseError> {
        // 1. Validações de entrada
        //    → EmptySource se content vazio
        //    → UnsupportedLanguage se language != Rust

        // 2. Parse tree-sitter
        //    → SyntaxError se root.has_error()

        // 3. Extração do header cristalino
        //    → extract_header(&file.content)
        //    → prompt_file_exists via PromptReader::exists()
        //    → current_hash via PromptReader::read_hash()

        // 4. Extração de imports
        //    → collect_imports(root, source, &self.config)
        //    → cada import tem target_layer resolvido

        // 5. Extração de tokens
        //    → collect_tokens(root, source)

        // 6. Test coverage
        //    → has_cfg_test(root, source)
        //    → || file.has_adjacent_test
        //    → || is_declaration_only(root, source)

        // 7. Interface pública
        //    → extract_public_interface(root, source)
        //    → prompt_snapshot via SnapshotReader::read_snapshot()

        // 8. Construir e retornar ParsedFile
    }
}
```

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

Dado source Rust com apenas pub trait Foo { fn bar(&self); }
Quando parse() for chamado
Então has_test_coverage = true (isento — declaration only)

Dado source Rust com pub fn check(file: &ParsedFile) -> Vec<Violation>
Quando parse() for chamado
Então public_interface.functions contém FunctionSignature {
    name: "check",
    params: ["&ParsedFile"],
    return_type: Some("Vec<Violation>")
}

Dado source Rust com pub struct Violation { pub rule_id: String, .. }
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "Violation",
    kind: TypeKind::Struct,
    members: ["rule_id: String", ...]
}

Dado source Rust com fn helper() privado
Quando parse() for chamado
Então helper não aparece em public_interface.functions

Dado source Rust com pub use crate::entities::Layer
Quando parse() for chamado
Então public_interface.reexports contém "crate::entities::Layer"

Dado dois sources com mesma interface mas whitespace diferente
Quando parse() for chamado em ambos
Então public_interface é idêntica nos dois

Dado prompt com seção Interface Snapshot válida
Quando parse() for chamado
Então prompt_snapshot é Some(PublicInterface) desserializada

Dado prompt sem seção Interface Snapshot
Quando parse() for chamado
Então prompt_snapshot é None — V6 não dispara sem baseline

Dado source Rust sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado source Rust vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })

Dado PromptReader e PromptSnapshotReader como mocks
Quando parse() for chamado
Então nenhum acesso a disco ocorre nas regras de L1
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | rs_parser.rs |
| 2025-03-13 | Gap 4: responsabilidades explícitas, LayerResolver, FsPromptReader | rs_parser.rs |
| 2025-03-13 | V6: extração de PublicInterface, normalização de tipos, injeção de PromptSnapshotReader, pipeline explícito | rs_parser.rs |
