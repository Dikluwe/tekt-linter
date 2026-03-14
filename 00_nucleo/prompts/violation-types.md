# Prompt: Types of Violations (violation-types)

**Camada**: L1 (Core - Entities)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

Este módulo define as entidades fundamentais do linter: `ParsedFile`,
`Violation`, `Layer` e tipos auxiliares. Formam a Representação
Intermediária (IR) sobre a qual todas as regras V1–V5 operam de forma
pura e agnóstica a linguagem e filesystem.

L3 é responsável por popular todos os campos — incluindo os booleanos
derivados de filesystem e os `target_layer` de cada import. L1 nunca
deriva esses valores por conta própria.

---

## Estruturas de Dados

### `ParsedFile`
```rust
pub struct ParsedFile {
    pub path: PathBuf,
    pub layer: Layer,
    pub language: Language,

    // Para V1
    pub prompt_header: Option<PromptHeader>,
    pub prompt_file_exists: bool,
    // true se prompt_header.prompt_path existe em 00_nucleo/
    // false se header ausente ou arquivo não encontrado
    // populado por L3 via PromptReader

    // Para V2
    pub has_test_coverage: bool,
    // true se arquivo contém #[cfg(test)] no AST
    // ou se arquivo _test.rs adjacente existe em disco
    // populado por L3 (FileWalker + LanguageParser)

    // Para V3
    pub imports: Vec<Import>,

    // Para V4
    pub tokens: Vec<Token>,
}
```

### `PromptHeader`
```rust
pub struct PromptHeader {
    pub prompt_path: String,
    pub prompt_hash: Option<String>,  // hash declarado no header
    pub current_hash: Option<String>, // hash real do arquivo em disco
    // populado por L3 via PromptReader::read_hash()
    // None se arquivo não existe
    pub layer: Layer,
    pub updated: Option<String>,
}
```

`current_hash` resolve o Gap 4 parcialmente — V5 compara
`prompt_hash` com `current_hash` sem nenhum acesso a disco.

### `Import`
```rust
pub struct Import {
    pub path: String,
    pub line: usize,
    pub kind: ImportKind,
    pub target_layer: Layer,
    // Layer::Unknown se o path não mapeia para nenhuma camada cristalina
    // populado por L3 baseado no prefix do path e crystalline.toml
}

pub enum ImportKind {
    Use,
    ExternCrate,
    ModDecl,
}
```

### `Token`
```rust
pub struct Token {
    pub symbol: String,   // ex: "std::fs::read"
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

pub enum TokenKind {
    CallExpression,
    MacroInvocation,
}
```

### `Violation`
```rust
pub struct Violation {
    pub rule_id: String,      // "V1" | "V2" | "V3" | "V4" | "V5"
    pub level: ViolationLevel,
    pub message: String,
    pub location: Location,
}

pub struct Location {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

pub enum ViolationLevel {
    Error,
    Warning,
}
```

### `Layer`
```rust
pub enum Layer {
    L0,
    L1,
    L2,
    L3,
    L4,
    Lab,
    Unknown,
}
```

### `Language`
```rust
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Unknown,
}
```

---

## Responsabilidades de População (L3)

| Campo | Quem popula | Como |
|-------|-------------|------|
| `prompt_file_exists` | `FsPromptReader` | `PromptReader::exists()` |
| `has_test_coverage` | `FileWalker` + `RustParser` | adjacência em disco + nó `#[cfg(test)]` no AST |
| `Import.target_layer` | `RustParser` | prefix matching do path contra `crystalline.toml` |
| `PromptHeader.current_hash` | `FsPromptReader` | `PromptReader::read_hash()` |

---

## Restrições (L1)

- Zero I/O — todas as structs são dados puros
- `ParsedFile` é construído inteiramente por L3 antes de chegar a L1
- Regras em L1 apenas leem os campos — nunca os derivam

---

## Critérios de Verificação
```
Dado ParsedFile com prompt_file_exists = false
Quando V1::check() for chamado
Então retorna Violation com rule_id "V1"

Dado ParsedFile com has_test_coverage = false e layer = L1
Quando V2::check() for chamado
Então retorna Violation com rule_id "V2"

Dado Import com target_layer = L3 em arquivo com layer = L2
Quando V3::check() for chamado
Então retorna Violation com rule_id "V3"

Dado PromptHeader com prompt_hash = "a3f8c2d1"
e current_hash = Some("b9e4f7a2")
Quando V5::check() for chamado
Então retorna Violation com rule_id "V5" e level Warning

Dado ParsedFile construído com todos os campos populados
Quando qualquer regra for chamada
Então nenhuma regra acessa disco ou faz I/O
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | parsed_file.rs, violation.rs, layer.rs |
| 2025-03-13 | Gap 2: adicionado prompt_file_exists, has_test_coverage, Import.target_layer, PromptHeader.current_hash | parsed_file.rs |
