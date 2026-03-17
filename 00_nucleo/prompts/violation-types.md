# Prompt: Types of Violations (violation-types)

**Camada**: L1 (Core - Entities)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-16 (ADR-0007: Declaration, HasWiringPurity, V11 fields em LocalIndex/ProjectIndex)

---

## Contexto

Este módulo define as entidades fundamentais do linter: `ParsedFile`,
`Violation`, `Layer` e tipos auxiliares. Formam a Representação
Intermediária (IR) sobre a qual todas as regras V1–V12 operam de forma
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
| `Location.path` | `Cow<'a, Path>` | Violações normais usam `Borrowed(&'a Path)`. Erros de infraestrutura (V0, PARSE) e violações globais (V11) usam `Owned(PathBuf)` — elimina `Box::leak()` (ADR-0005) |
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

/// Fatal: erros de infraestrutura que impedem análise completa (V0, V8)
/// e violações de quarentena que comprometem a garantia de produção (V10).
/// Fatal não pode ser suprimido por --fail-on — bloqueia CI
/// independentemente de configuração.
/// Error: violações arquiteturais bloqueantes (V1–V4, V9, V11).
/// Warning: divergências não bloqueantes por padrão (V5–V7, V12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationLevel {
    Fatal,
    Error,
    Warning,
}

/// ADR-0005: path usa Cow<'a, Path>.
/// Borrowed(&'a Path) — violações normais, path referencia o SourceFile.
/// Owned(PathBuf) — erros de infraestrutura (V0, PARSE) e violações
/// globais sem arquivo específico (V11), path é owned.
/// Elimina Box::leak() dos conversores em L4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location<'a> {
    pub path: Cow<'a, Path>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation<'a> {
    pub rule_id: String,   // "V0"–"V12", "PARSE" — gerado pela regra
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

    // Para V3, V9, V10
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

    // Para V12
    pub declarations: Vec<Declaration<'a>>,
    // Declarações de tipo de nível superior extraídas do AST.
    // Usado por V12 para detectar struct/enum/impl sem trait em L4.
    // Populado por L3 (RustParser) para todos os arquivos —
    // V12 filtra por layer == L4 internamente.
}
```

### `PromptHeader<'a>`
```rust
pub struct PromptHeader<'a> {
    pub prompt_path: &'a str,
    pub prompt_hash: Option<&'a str>,  // hash declarado no header do arquivo
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
    /// Resolvido por L3 via crystalline.toml [layers].
    /// Layer::Unknown para crates externas — não gera violação V3.
    /// Layer::Lab para imports de lab/ — dispara V10 em produção.
    pub target_layer: Layer,
    /// Subdiretório de destino dentro da camada alvo.
    /// Resolvido por L3 via crystalline.toml [l1_ports].
    /// None se target_layer == Unknown (crate externa).
    /// Some("entities") se import aponta para 01_core/entities/.
    /// Some("internal") se import aponta para subdir não-porta.
    /// Usado por V9 para detectar imports fora das portas de L1.
    pub target_subdir: Option<&'a str>,  // ADR-0006
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

### `Declaration<'a>` (V12)
```rust
/// Declaração de tipo de nível superior num arquivo fonte.
/// Usado por V12 para detectar struct/enum/impl sem trait em L4.
/// Populado por RustParser para todos os arquivos — V12 filtra
/// por layer == L4 internamente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'a> {
    pub kind: DeclarationKind,
    pub name: &'a str,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationKind {
    /// struct_item de nível superior.
    /// Permitido em L4 se allow_adapter_structs = true (padrão).
    Struct,
    /// enum_item de nível superior.
    /// Sempre proibido em L4 — enums pertencem a L1 ou L2.
    Enum,
    /// impl_item sem trait: `impl Type { ... }`.
    /// Sempre proibido em L4 — indica lógica de negócio no wiring.
    /// `impl Trait for Type` NÃO é capturado aqui — é o padrão de adapter.
    Impl,
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

### `WiringConfig` (V12)
```rust
/// Configuração de exceções para V12, lida de crystalline.toml
/// [wiring_exceptions] e injetada por L4.
/// V12 nunca lê o toml diretamente.
#[derive(Debug, Clone)]
pub struct WiringConfig {
    /// Se true, struct_item em L4 é permitido (padrão: true).
    /// Structs de adapter são comuns em fases de migração.
    /// enum_item e impl_item sem trait são sempre proibidos.
    pub allow_adapter_structs: bool,
}

impl Default for WiringConfig {
    fn default() -> Self {
        Self { allow_adapter_structs: true }
    }
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
| `imports` com `target_layer` | `RustParser` | `use_declaration` no AST + `LayerResolver` via `crystalline.toml [layers]`. `Layer::Lab` é resolvido para imports de `lab/` |
| `imports` com `target_subdir` | `RustParser` | segundo segmento do path resolvido contra `crystalline.toml [l1_ports]`. None para crates externas e imports de Lab |
| `tokens` com FQN | `RustParser` | Fase 1 (tabela de aliases) + Fase 2 (call_expression). Alias → `Cow::Owned`, direto → `Cow::Borrowed` |
| `PromptHeader.prompt_hash` | `RustParser` | fatia `&'a str` do buffer do arquivo |
| `PromptHeader.current_hash` | `FsPromptReader` | SHA256[0..8] calculado do disco — `Option<String>` |
| `public_interface` | `RustParser` | nós `pub` do AST, strings normalizadas como `&'a str` |
| `prompt_snapshot` | `PromptSnapshotReader` | desserialização do JSON em `## Interface Snapshot` do prompt |
| `declarations` | `RustParser` | nós `struct_item`, `enum_item` e `impl_item` sem trait de nível superior. `impl Trait for Type` não é capturado |

**Responsabilidades de L3 para V11 (via LocalIndex):**

| Campo | Quem popula | Como |
|-------|-------------|------|
| `LocalIndex.declared_traits` | `RustParser` | nós `trait_item` com `pub` em arquivos com `layer == L1` e `subdir == "contracts"` |
| `LocalIndex.implemented_traits` | `RustParser` | nós `impl_item` com `impl <TraitName> for` em arquivos com `layer == L2 \| L3` |

**Responsabilidades de L4 (wiring) para construção de `Location`:**

| Violação | Como construir `Location.path` |
|----------|-------------------------------|
| V1–V10, V12 normais | `Cow::Borrowed(parsed_file.path)` |
| V0 (SourceError) | `Cow::Owned(path)` — path vem de `SourceError::Unreadable` |
| PARSE (ParseError) | `Cow::Owned(path)` — path vem das variants de `ParseError` |
| V11 (global) | `Cow::Owned(PathBuf::from("01_core/contracts"))` — violação global sem arquivo específico |

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
- `Declaration` é populado por L3 para todos os arquivos —
  V12 filtra por `layer == L4` internamente, não o parser
- `WiringConfig` é injetado por L4 em V12 — nunca lido diretamente
  de disco por L1

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

Dado Import { target_layer: L1, target_subdir: Some("internal"), .. }
em arquivo com layer = L2
E "internal" não listado em [l1_ports]
Quando V9::check() for chamado
Então retorna Violation { rule_id: "V9", level: Error }

Dado Import { target_layer: L1, target_subdir: Some("entities"), .. }
em arquivo com layer = L2
E "entities" listado em [l1_ports]
Quando V9::check() for chamado
Então retorna vec![] — porta válida

Dado Import { target_layer: Layer::Lab, line: 5, .. }
em arquivo com layer = L1
Quando V10::check() for chamado
Então retorna Violation { rule_id: "V10", level: Fatal,
      location: Location { path: Cow::Borrowed(..), line: 5 } }

Dado Import { target_layer: Layer::Lab, .. }
em arquivo com layer = Lab
Quando V10::check() for chamado
Então retorna vec![] — lab pode importar lab

Dado all_declared_traits = {"FileProvider"}
E all_implemented_traits = {}
Quando check_dangling_contracts() for chamado
Então retorna Violation { rule_id: "V11", level: Error,
      location: Location { path: Cow::Owned("01_core/contracts"), .. } }

Dado all_declared_traits = {"FileProvider"}
E all_implemented_traits = {"FileProvider"}
Quando check_dangling_contracts() for chamado
Então retorna vec![]

Dado Declaration { kind: Enum, name: "OutputMode", line: 3 }
em arquivo com layer = L4
Quando V12::check() for chamado com WiringConfig::default()
Então retorna Violation { rule_id: "V12", level: Warning, location.line: 3 }
— enum nunca permitido em L4

Dado Declaration { kind: Struct, name: "L3HashRewriter", line: 7 }
em arquivo com layer = L4
Quando V12::check() for chamado com WiringConfig { allow_adapter_structs: true }
Então retorna vec![] — struct de adapter explicitamente permitida

Dado Declaration { kind: Impl, name: "L3HashRewriter", line: 10 }
(impl sem trait)
em arquivo com layer = L4
Quando V12::check() for chamado
Então retorna Violation { rule_id: "V12", level: Warning }
— impl sem trait é lógica de negócio no wiring

Dado Declaration { kind: Impl, name: "HashRewriter for L3HashRewriter", .. }
(impl com trait — NÃO capturado como DeclarationKind::Impl)
em arquivo com layer = L4
Quando V12::check() for chamado
Então retorna vec![] — impl de trait é o padrão de adapter

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
| 2026-03-14 | ADR-0004: lifetimes `<'a>`, ViolationLevel::Fatal, Cow<'a, str> em Token.symbol, proibição de clone documentada | parsed_file.rs, violation.rs |
| 2026-03-14 | Errata Cow: Token.symbol de &'a str para Cow<'a, str> | parsed_file.rs |
| 2026-03-14 | ADR-0005: Location.path de &'a Path para Cow<'a, Path>, elimina Box::leak(), tabela L4 adicionada | violation.rs |
| 2026-03-14 | ADR-0006: Import.target_subdir adicionado para V9, linha de população adicionada na tabela L3, critérios V9 adicionados, ViolationLevel::Fatal atualizado para incluir V8 | parsed_file.rs |
| 2026-03-16 | ADR-0007: Declaration e DeclarationKind para V12; ParsedFile.declarations; WiringConfig; V10 Fatal na tabela de níveis e critérios; V11 na tabela L4 de Location e critérios; tabela de responsabilidades L3 para V11 (declared_traits, implemented_traits via LocalIndex) | parsed_file.rs, violation.rs |
| 2026-03-16 | Materialização ADR-0007: Declaration, DeclarationKind, WiringConfig adicionados a parsed_file.rs; ParsedFile recebe declared_traits, implemented_traits e declarations; impl HasWiringPurity para ParsedFile | parsed_file.rs |
