# Prompt: Types of Violations (violation-types)

**Camada**: L1 (Core - Entities)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004, ADR-0005)

---

## Contexto

Este módulo define as entidades fundamentais do linter: `ParsedFile`,
`Violation`, `Layer` e tipos auxiliares. Formam a Representação
Intermediária (IR) sobre a qual todas as regras V1–V6 operam de forma
pura e agnóstica à linguagem e filesystem.

**Diretiva Zero-Copy (ADR-0004):** As estruturas de dados não são
donas das strings do código-fonte. Todas recebem lifetime `<'a>` e
guardam referências para o conteúdo carregado em memória pelo walker.
Uma única alocação por arquivo, zero cópias intermediárias — exceto
onde documentado na tabela abaixo.

**Exceções justificadas e documentadas:**

| Campo | Tipo | Motivo |
|-------|------|--------|
| `PromptHeader.current_hash` | `Option<String>` | SHA256 calculado do disco — não existe no buffer do fonte |
| `Token.symbol` | `Cow<'a, str>` | FQN resolvido por concatenação de alias não existe no buffer original |
| `Location.path` | `Cow<'a, Path>` | Violações normais usam `Borrowed(&'a Path)`. Erros de infraestrutura (V0, PARSE) usam `Owned(PathBuf)` — elimina `Box::leak()` (ADR-0005) |
| `Violation.rule_id` | `String` | Identificador gerado pela regra, não extraído do fonte |
| `Violation.message` | `String` | Mensagem formatada pela regra, não extraída do fonte |

**Proibição de clone em L1:** As funções de regras apenas leem
referências e comparam. Usar `.to_string()` ou `String::from()`
dentro do motor de avaliação de regras é violação da política
zero-copy — exceto `rule_id` e `message` gerados pela regra.

---

## Estruturas de Dados

### `ViolationLevel` e `Violation<'a>`
```rust
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Fatal: erros de infraestrutura que impedem análise completa (V0).
/// Fatal não pode ser suprimido por --fail-on — bloqueia CI
/// independentemente de configuração.
/// Error: violações arquiteturais bloqueantes (V1–V4).
/// Warning: divergências não bloqueantes por padrão (V5–V6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationLevel {
    Fatal,
    Error,
    Warning,
}

/// ADR-0005: path usa Cow<'a, Path>.
/// Borrowed(&'a Path) — violações normais, path referencia o SourceFile.
/// Owned(PathBuf) — erros de infraestrutura (V0, PARSE), path é owned.
/// Elimina Box::leak() dos conversores em L4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location<'a> {
    pub path: Cow<'a, Path>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation<'a> {
    pub rule_id: String,   // "V0"–"V6", "PARSE" — gerado pela regra
    pub level: ViolationLevel,
    pub message: String,   // formatado pela regra
    pub location: Location<'a>,
}
```

### `ParsedFile<'a>`
```rust
pub struct ParsedFile<'a> {
    pub path: &'a Path,
    pub layer: Layer,
    pub language: Language,

    // Para V1
    pub prompt_header: Option<PromptHeader<'a>>,
    pub prompt_file_exists: bool,
    // true se prompt_header.prompt_path existe em 00_nucleo/
    // false se header ausente ou arquivo não encontrado
    // populado por L3 via PromptReader

    // Para V2
    pub has_test_coverage: bool,
    // true se #[cfg(test)] no AST ou foo_test.rs adjacente
    // true se arquivo é declaration-only (isento de V2)
    // populado por L3 (FileWalker + RustParser)

    // Para V3
    pub imports: Vec<Import<'a>>,

    // Para V4
    pub tokens: Vec<Token<'a>>,

    // Para V6
    pub public_interface: PublicInterface<'a>,
    // Interface pública extraída do AST pelo RustParser (L3)

    pub prompt_snapshot: Option<PublicInterface<'a>>,
    // Snapshot registrado em ## Interface Snapshot do prompt
    // None se prompt não tem snapshot ou não existe
    // Populado por L3 via PromptSnapshotReader
}
```

### `PromptHeader<'a>`
```rust
pub struct PromptHeader<'a> {
    pub prompt_path: &'a str,
    pub prompt_hash: Option<&'a str>,  // hash declarado no header
    pub current_hash: Option<String>,  // EXCEÇÃO zero-copy:
    // SHA256[0..8] calculado pelo FsPromptReader a partir do disco.
    // Não existe no buffer do arquivo fonte.
    pub layer: Layer,
    pub updated: Option<&'a str>,
}
```

### `Import<'a>`
```rust
pub struct Import<'a> {
    pub path: &'a str,        // sempre presente no buffer — &'a str puro
    pub line: usize,
    pub kind: ImportKind,
    /// Resolvido por L3 via crystalline.toml.
    /// Layer::Unknown para crates externas — não gera violação V3.
    pub target_layer: Layer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Use,
    ExternCrate,
    ModDecl,
}
```

### `Token<'a>`
```rust
use std::borrow::Cow;

pub struct Token<'a> {
    /// FQN resolvido pelo RustParser (ADR-0004 + Errata Cow).
    ///
    /// Cow::Borrowed(&'a str) — símbolo presente literalmente no buffer:
    ///   `std::fs::read(...)`  →  Borrowed("std::fs::read")
    ///
    /// Cow::Owned(String) — FQN construído por resolução de alias:
    ///   `use std::fs as f; f::read(...)`  →  Owned("std::fs::read")
    ///
    /// V4 acessa via Deref<Target = str> — alheio à distinção.
    /// L1 permanece alheio à origem da string.
    pub symbol: Cow<'a, str>,
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    CallExpression,
    MacroInvocation,
}
```

### `PublicInterface<'a>` (V6)
```rust
/// Interface pública extraída do AST — agnóstica de linguagem.
/// Não inclui implementação, apenas contratos visíveis externamente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicInterface<'a> {
    pub functions: Vec<FunctionSignature<'a>>,
    pub types: Vec<TypeSignature<'a>>,
    pub reexports: Vec<&'a str>,
}

impl<'a> PublicInterface<'a> {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.types.is_empty()
            && self.reexports.is_empty()
    }
}
```

### `FunctionSignature<'a>` e `TypeSignature<'a>` (V6)
```rust
/// Critério de igualdade: name + params + return_type devem ser
/// todos iguais. Mudança em qualquer campo é quebra de contrato.
/// PartialEq derivado sobre a struct completa — nunca comparar só name.
/// foo(a: String) → foo(a: Vec<String>) é remoção + adição no delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature<'a> {
    pub name: &'a str,
    pub params: Vec<&'a str>,         // tipos normalizados (whitespace colapsado)
    pub return_type: Option<&'a str>, // None para fn que retorna ()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature<'a> {
    pub name: &'a str,
    pub kind: TypeKind,
    pub members: Vec<&'a str>, // campos de struct / variantes de enum /
                               // assinaturas de método de trait
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
}
```

### `InterfaceDelta<'a>` e `compute_delta` (V6)
```rust
/// Diferença entre interface atual e snapshot do prompt.
/// Produzida por compute_delta() — função pura, zero I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDelta<'a> {
    pub added_functions: Vec<FunctionSignature<'a>>,
    pub removed_functions: Vec<FunctionSignature<'a>>,
    pub added_types: Vec<TypeSignature<'a>>,
    pub removed_types: Vec<TypeSignature<'a>>,
    pub added_reexports: Vec<&'a str>,
    pub removed_reexports: Vec<&'a str>,
}

impl<'a> InterfaceDelta<'a> {
    pub fn is_empty(&self) -> bool {
        self.added_functions.is_empty()
            && self.removed_functions.is_empty()
            && self.added_types.is_empty()
            && self.removed_types.is_empty()
            && self.added_reexports.is_empty()
            && self.removed_reexports.is_empty()
    }

    /// Produz string legível para mensagem de violação.
    /// Ordem: adições antes de remoções, funções antes de tipos.
    /// Exemplo: "+fn check, -fn validate, +struct Delta"
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for f in &self.added_functions {
            parts.push(format!("+fn {}", f.name));
        }
        for f in &self.removed_functions {
            parts.push(format!("-fn {}", f.name));
        }
        for t in &self.added_types {
            parts.push(format!("+{} {}", type_kind_str(&t.kind), t.name));
        }
        for t in &self.removed_types {
            parts.push(format!("-{} {}", type_kind_str(&t.kind), t.name));
        }
        for r in &self.added_reexports {
            parts.push(format!("+use {}", r));
        }
        for r in &self.removed_reexports {
            parts.push(format!("-use {}", r));
        }
        parts.join(", ")
    }
}

fn type_kind_str(kind: &TypeKind) -> &'static str {
    match kind {
        TypeKind::Struct => "struct",
        TypeKind::Enum   => "enum",
        TypeKind::Trait  => "trait",
    }
}

/// Computa delta entre interface atual e snapshot.
/// Usa PartialEq completo (name + params + return_type) —
/// nunca compara apenas por name.
/// Mudança de assinatura aparece como remoção + adição.
/// Função pura — zero I/O, zero alocações além dos Vec de resultado.
pub fn compute_delta<'a>(
    current: &PublicInterface<'a>,
    snapshot: &PublicInterface<'a>,
) -> InterfaceDelta<'a> {
    InterfaceDelta {
        added_functions: current.functions.iter()
            .filter(|f| !snapshot.functions.contains(f))
            .cloned().collect(),
        removed_functions: snapshot.functions.iter()
            .filter(|f| !current.functions.contains(f))
            .cloned().collect(),
        added_types: current.types.iter()
            .filter(|t| !snapshot.types.contains(t))
            .cloned().collect(),
        removed_types: snapshot.types.iter()
            .filter(|t| !current.types.contains(t))
            .cloned().collect(),
        added_reexports: current.reexports.iter()
            .filter(|r| !snapshot.reexports.contains(r))
            .cloned().collect(),
        removed_reexports: snapshot.reexports.iter()
            .filter(|r| !current.reexports.contains(r))
            .cloned().collect(),
    }
}
```

### `Layer` e `Language`
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    L0, L1, L2, L3, L4, Lab, Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust, TypeScript, Python, Unknown,
}
```

---

## Responsabilidades de População (L3)

L3 constrói `ParsedFile<'a>` inteiramente antes de entregar a L1.
L1 apenas lê — nunca deriva, nunca aloca além das exceções documentadas.

| Campo | Quem popula | Como |
|-------|-------------|------|
| `prompt_file_exists` | `FsPromptReader` | `PromptReader::exists()` |
| `has_test_coverage` | `FileWalker` + `RustParser` | adjacência em disco + nó `#[cfg(test)]` no AST + detection de declaration-only |
| `imports` com `target_layer` | `RustParser` | `use_declaration` no AST + `LayerResolver` via `crystalline.toml` |
| `tokens` com FQN | `RustParser` | Fase 1 (tabela de aliases) + Fase 2 (call_expression). Alias → `Cow::Owned`, direto → `Cow::Borrowed` |
| `PromptHeader.prompt_hash` | `RustParser` | fatia `&'a str` do buffer do arquivo |
| `PromptHeader.current_hash` | `FsPromptReader` | SHA256[0..8] calculado do disco — `Option<String>` |
| `public_interface` | `RustParser` | nós `pub` do AST, strings normalizadas como `&'a str` |
| `prompt_snapshot` | `PromptSnapshotReader` | desserialização do JSON em `## Interface Snapshot` do prompt |

**Responsabilidades de L4 (wiring) para construção de `Location`:**

| Violação | Como construir `Location.path` |
|----------|-------------------------------|
| V1–V6 normais | `Cow::Borrowed(parsed_file.path)` |
| V0 (SourceError) | `Cow::Owned(path)` — path vem de `SourceError::Unreadable` |
| PARSE (ParseError) | `Cow::Owned(path)` — path vem das variants de `ParseError` |

---

## Restrições (L1)

- Zero I/O — todas as structs referenciam o buffer do `SourceFile`
- `ParsedFile<'a>` é construído inteiramente por L3 antes de chegar a L1
- `compute_delta` é função pura sobre dois `PublicInterface<'a>`
- **Proibido** `.to_string()` ou `String::from()` dentro das funções
  de regras — exceto `rule_id` e `message` gerados pela própria regra
- `Token.symbol` é `Cow<'a, str>` — V4 acessa via `Deref<Target = str>`
  sem precisar saber se é Borrowed ou Owned
- `Location.path` é `Cow<'a, Path>` — regras usam `Borrowed`,
  wiring usa `Owned` para erros de infraestrutura sem `Box::leak()`
- Comparação de `FunctionSignature` e `TypeSignature` usa `PartialEq`
  derivado sobre a struct completa — nunca comparar apenas por `name`

---

## Critérios de Verificação
```
Dado ParsedFile com prompt_file_exists = false
Quando V1::check() for chamado
Então retorna Violation { rule_id: "V1", level: Error,
      location: Location { path: Cow::Borrowed(..), .. } }

Dado ParsedFile com has_test_coverage = false e layer = L1
Quando V2::check() for chamado
Então retorna Violation { rule_id: "V2", level: Error,
      location: Location { path: Cow::Borrowed(..), .. } }

Dado Import com target_layer = L3 em arquivo com layer = L2
Quando V3::check() for chamado
Então retorna Violation { rule_id: "V3", level: Error }

Dado Token { symbol: Cow::Borrowed("std::fs::read"), .. } em arquivo L1
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", level: Error }
— Borrowed tratado identicamente via Deref

Dado Token { symbol: Cow::Owned("std::fs::read"), .. } em arquivo L1
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", level: Error }
— Owned tratado identicamente via Deref

Dado PromptHeader com prompt_hash = "a3f8c2d1"
e current_hash = Some("b9e4f7a2")
Quando V5::check() for chamado
Então retorna Violation { rule_id: "V5", level: Warning }

Dado PublicInterface com foo(a: String) -> bool
E prompt_snapshot com foo(a: Vec<String>) -> bool
Quando compute_delta() for chamado
Então delta contém removed_functions [foo(String)]
e added_functions [foo(Vec<String>)]
— PartialEq completo detecta mudança de assinatura

Dado PublicInterface idêntica ao prompt_snapshot
Quando compute_delta() for chamado
Então InterfaceDelta.is_empty() == true

Dado InterfaceDelta com +fn check e -fn validate e +struct Delta
Quando describe() for chamado
Então retorna "+fn check, -fn validate, +struct Delta"
— ordem: adições antes de remoções, funções antes de tipos

Dado SourceError::Unreadable { path, reason }
Quando source_error_to_violation() for chamado em L4
Então retorna Violation { rule_id: "V0", level: Fatal,
      location: Location { path: Cow::Owned(path), .. } }
— sem Box::leak(), sem 'static desnecessário

Dado ParseError::SyntaxError { path, line, .. }
Quando parse_error_to_violation() for chamado em L4
Então retorna Violation { level: Error,
      location: Location { path: Cow::Owned(path), .. } }

Dado Violation com level Fatal
Quando should_fail() for chamado independentemente de --fail-on
Então retorna true — Fatal não configurável
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | parsed_file.rs, violation.rs, layer.rs |
| 2025-03-13 | Gap 2: prompt_file_exists, has_test_coverage, Import.target_layer, PromptHeader.current_hash | parsed_file.rs |
| 2025-03-13 | V6: PublicInterface, FunctionSignature, TypeSignature, InterfaceDelta, compute_delta | parsed_file.rs |
| 2026-03-14 | ADR-0004: lifetimes `<'a>` em todas as structs, ViolationLevel::Fatal, Cow<'a, str> em Token.symbol, proibição de clone documentada | parsed_file.rs, violation.rs |
| 2026-03-14 | Errata Cow: Token.symbol alterado de &'a str para Cow<'a, str> | parsed_file.rs |
| 2026-03-14 | ADR-0005: Location.path alterado de &'a Path para Cow<'a, Path>, elimina Box::leak() nos conversores, tabela de responsabilidades L4 adicionada, compute_delta com implementação completa, describe() com implementação completa | violation.rs, parsed_file.rs |
