# Prompt: Types of Violations (violation-types)
Hash do Código: f0776e27

**Camada**: L1 (Core - Entities)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-18 (ADR-0009 correcção: ImportKind semântico, não sintáctico)

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

**Princípio de Agnósticidade (ADR-0001, ADR-0009):** A IR modela
intenção semântica, nunca sintaxe de linguagem. Nenhum tipo ou
variante em L1 pode referenciar uma linguagem específica. A distinção
entre `use crate::foo` (Rust), `import { X } from './foo'` (TypeScript)
e `from foo import X` (Python) é responsabilidade de L3 — L1 recebe
apenas a mecânica estrutural do import.

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
zero-copy — exceto `rule_id` e `message` gerados pela própria regra.

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
    // populado por L3 via PromptReader

    // Para V2
    pub has_test_coverage: bool,
    // true se construto de teste no AST, ficheiro adjacente, ou declaration-only
    // populado por L3 (FileWalker + parser)

    // Para V3, V9, V10
    pub imports: Vec<Import<'a>>,

    // Para V4
    // V4 usa file.language para seleccionar a lista de símbolos proibidos.
    // Os tokens são agnósticos de linguagem — apenas symbol, line, column, kind.
    pub tokens: Vec<Token<'a>>,

    // Para V6
    pub public_interface: PublicInterface<'a>,
    pub prompt_snapshot: Option<PublicInterface<'a>>,

    // Para V12
    pub declarations: Vec<Declaration<'a>>,

    // Para V11 — transportados para LocalIndex via from_parsed()
    // Não consumidos por nenhuma regra de L1 directamente
    pub declared_traits: Vec<&'a str>,
    pub implemented_traits: Vec<&'a str>,
}
```

### `PromptHeader<'a>`
```rust
pub struct PromptHeader<'a> {
    pub prompt_path: &'a str,
    pub prompt_hash: Option<&'a str>,  // hash declarado no header do arquivo
    pub current_hash: Option<String>,  // EXCEÇÃO zero-copy:
    // SHA256[0..8] calculado pelo FsPromptReader a partir do disco.
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
    /// Resolvido por L3 via resolução física (ADR-0009).
    /// Layer::Unknown para packages/crates externos.
    /// Layer::Lab para imports de lab/ — dispara V10 em produção.
    pub target_layer: Layer,
    /// Subdiretório de destino dentro da camada alvo.
    /// Resolvido por L3 via strip_prefix após resolução física.
    /// None se target_layer == Unknown.
    /// Some("entities") se import aponta para 01_core/entities/.
    /// Usado por V9 para detectar imports fora das portas de L1.
    pub target_subdir: Option<&'a str>,
}

/// Mecânica estrutural do import — agnóstica de linguagem.
///
/// Descreve COMO o import foi escrito, não em que linguagem.
/// Nunca adicionar variantes que referenciem uma linguagem específica
/// (não pode existir RustUse, TsImport, PyImport).
///
/// Mapeamento por linguagem (responsabilidade de L3):
///   Rust:       use crate::foo        → Direct
///               use crate::foo::*     → Glob
///               use std::fs as f      → Alias
///               use crate::{A, B}     → Named
///               extern crate foo      → Direct
///               mod foo;              → Direct
///   TypeScript: import X from '...'   → Direct
///               import * from '...'   → Glob
///               import X as Y         → Alias
///               import { A, B }       → Named
///               export { X } from     → Direct
///   Python:     import os             → Direct
///               from os import *      → Glob
///               import numpy as np    → Alias
///               from os import path   → Named
///               from . import X       → Named (relativo)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    /// Import directo de um módulo ou símbolo único.
    /// Rust: `use crate::foo`, `extern crate foo`, `mod foo;`
    /// TypeScript: `import X from '...'`, `import '...'`
    /// Python: `import os`
    Direct,

    /// Import de todos os símbolos públicos de um módulo.
    /// Rust: `use crate::foo::*`
    /// TypeScript: `import * as ns from '...'`
    /// Python: `from os import *`
    Glob,

    /// Import com renomeação local.
    /// Rust: `use std::fs as f`
    /// TypeScript: `import X as Y from '...'`
    /// Python: `import numpy as np`
    Alias,

    /// Import de um subconjunto nomeado de símbolos.
    /// Rust: `use crate::{A, B, C}`
    /// TypeScript: `import { A, B } from '...'`
    /// Python: `from os import path, getcwd`
    Named,
}
```

### `Token<'a>`
```rust
use std::borrow::Cow;

pub struct Token<'a> {
    /// FQN resolvido pelo parser (ADR-0004 + Errata Cow).
    ///
    /// Cow::Borrowed(&'a str) — símbolo presente literalmente no buffer.
    /// Cow::Owned(String) — FQN construído por resolução de alias.
    ///
    /// V4 acessa via Deref<Target = str> — alheio à distinção.
    /// V4 usa file.language para seleccionar a lista de símbolos
    /// proibidos — nunca usa ImportKind para distinguir linguagens.
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
/// Agnóstica de linguagem — descreve o papel estrutural do tipo,
/// não a sintaxe com que foi escrito.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'a> {
    pub kind: DeclarationKind,
    pub name: &'a str,
    pub line: usize,
}

/// ADR-0009: estendido com variantes para linguagens OO.
/// A extensão é universal — não específica de nenhuma linguagem.
/// V12 trata DeclarationKind::Class como Struct para fins de
/// allow_adapter_structs — ambos são tipos concretos de dados.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationKind {
    // ── Rust ──────────────────────────────────────────────────────────────
    /// struct_item de nível superior.
    /// Permitido em L4 se allow_adapter_structs = true (padrão).
    Struct,
    /// enum_item de nível superior.
    /// Sempre proibido em L4.
    Enum,
    /// impl_item sem trait: `impl Type { ... }`.
    /// Sempre proibido em L4.
    /// `impl Trait for Type` NÃO é capturado aqui.
    Impl,

    // ── Linguagens OO (TypeScript, Python, Go...) ─────────────────────────
    /// class_declaration ou equivalente.
    /// Tratado como Struct por V12 para allow_adapter_structs.
    /// Proibido em L4 por padrão (allow_adapter_structs = true permite).
    Class,
    /// interface_declaration ou equivalente.
    /// Sempre proibido em L4.
    Interface,
    /// type_alias_declaration ou equivalente.
    /// Sempre proibido em L4.
    TypeAlias,
}
```

### `PublicInterface<'a>` (V6)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicInterface<'a> {
    pub functions: Vec<FunctionSignature<'a>>,
    pub types: Vec<TypeSignature<'a>>,
    pub reexports: Vec<&'a str>,
}

impl<'a> PublicInterface<'a> {
    pub fn empty() -> Self {
        Self { functions: vec![], types: vec![], reexports: vec![] }
    }

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature<'a> {
    pub name: &'a str,
    pub params: Vec<&'a str>,
    pub return_type: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature<'a> {
    pub name: &'a str,
    pub kind: TypeKind,
    pub members: Vec<&'a str>,
}

/// ADR-0009: estendido com variantes para linguagens OO.
/// Universal — não específico de nenhuma linguagem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    // ── Rust ──────────────────────────────────────────────────────────────
    Struct,
    Enum,
    Trait,

    // ── Linguagens OO (TypeScript, Python, Go...) ─────────────────────────
    /// class ou equivalente.
    Class,
    /// interface ou equivalente.
    Interface,
    /// type alias ou equivalente.
    TypeAlias,
}
```

### `InterfaceDelta<'a>` e `compute_delta` (V6)
```rust
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
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for f in &self.added_functions   { parts.push(format!("+fn {}", f.name)); }
        for f in &self.removed_functions { parts.push(format!("-fn {}", f.name)); }
        for t in &self.added_types       { parts.push(format!("+{} {}", type_kind_str(&t.kind), t.name)); }
        for t in &self.removed_types     { parts.push(format!("-{} {}", type_kind_str(&t.kind), t.name)); }
        for r in &self.added_reexports   { parts.push(format!("+use {}", r)); }
        for r in &self.removed_reexports { parts.push(format!("-use {}", r)); }
        parts.join(", ")
    }
}

fn type_kind_str(kind: &TypeKind) -> &'static str {
    match kind {
        TypeKind::Struct    => "struct",
        TypeKind::Enum      => "enum",
        TypeKind::Trait     => "trait",
        TypeKind::Class     => "class",
        TypeKind::Interface => "interface",
        TypeKind::TypeAlias => "type",
    }
}

pub fn compute_delta<'a>(
    current: &PublicInterface<'a>,
    snapshot: &PublicInterface<'a>,
) -> InterfaceDelta<'a> {
    InterfaceDelta {
        added_functions: current.functions.iter()
            .filter(|f| !snapshot.functions.contains(f)).cloned().collect(),
        removed_functions: snapshot.functions.iter()
            .filter(|f| !current.functions.contains(f)).cloned().collect(),
        added_types: current.types.iter()
            .filter(|t| !snapshot.types.contains(t)).cloned().collect(),
        removed_types: snapshot.types.iter()
            .filter(|t| !current.types.contains(t)).cloned().collect(),
        added_reexports: current.reexports.iter()
            .filter(|r| !snapshot.reexports.contains(r)).cloned().collect(),
        removed_reexports: snapshot.reexports.iter()
            .filter(|r| !current.reexports.contains(r)).cloned().collect(),
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
#[derive(Debug, Clone)]
pub struct WiringConfig {
    /// Se true, struct_item (Rust) e class_declaration (linguagens OO)
    /// em L4 são permitidos (padrão: true).
    /// enum_item, impl_item sem trait, interface_declaration e
    /// type_alias_declaration são sempre proibidos.
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
| `has_test_coverage` | `FileWalker` + parser | adjacência + construto de teste no AST + declaration-only |
| `imports` com `target_layer` | parser L3 | resolução física via `normalize` + `resolve_file_layer` (ADR-0009). Rust usa `LayerResolver` baseado em `crate::` — única excepção documentada |
| `imports` com `target_subdir` | parser L3 | `strip_prefix` após resolução física contra valor de `[layers]` |
| `imports` com `kind` | parser L3 | mapeamento de nós AST para `ImportKind` semântico: `Direct/Glob/Alias/Named`. Ver tabela de mapeamento em `ImportKind` |
| `tokens` com FQN | parser L3 | Rust: Fase 1 (aliases) + Fase 2 (call expressions). TS/Python: call expressions directamente. `symbol` é `Cow<'a, str>` |
| `PromptHeader.prompt_hash` | parser L3 | fatia `&'a str` do buffer |
| `PromptHeader.current_hash` | `FsPromptReader` | SHA256[0..8] — `Option<String>` |
| `public_interface` | parser L3 | nós `pub`/`export`/públicos do AST |
| `prompt_snapshot` | `PromptSnapshotReader` | JSON em `## Interface Snapshot` |
| `declarations` | parser L3 | struct/enum/impl-sem-trait/class/interface/type-alias de nível superior |
| `declared_traits` | parser L3 | traits/interfaces/Protocols públicos em L1/contracts/ |
| `implemented_traits` | parser L3 | traits/interfaces implementadas em L2/L3 |

**Responsabilidades de L4 para construção de `Location`:**

| Violação | Como construir `Location.path` |
|----------|-------------------------------|
| V1–V10, V12 normais | `Cow::Borrowed(parsed_file.path)` |
| V0 (SourceError) | `Cow::Owned(path)` |
| PARSE (ParseError) | `Cow::Owned(path)` |
| V11 (global) | `Cow::Owned(PathBuf::from("01_core/contracts"))` |

---

## Restrições (L1)

- Zero I/O — todas as structs referenciam o buffer do `SourceFile`
- `ParsedFile<'a>` é construído inteiramente por L3 antes de chegar a L1
- `compute_delta` é função pura sobre dois `PublicInterface<'a>`
- **Proibido** `.to_string()` ou `String::from()` dentro das funções
  de regras — exceto `rule_id` e `message`
- `Token.symbol` é `Cow<'a, str>` — V4 acessa via `Deref<Target = str>`
- `Location.path` é `Cow<'a, Path>` — regras usam `Borrowed`,
  wiring usa `Owned` para erros de infraestrutura
- Comparação de `FunctionSignature` e `TypeSignature` usa `PartialEq`
  derivado — nunca comparar apenas por `name`
- `Declaration` populado por L3 para todos os arquivos —
  V12 filtra por `layer == L4` internamente
- `WiringConfig` injetado por L4 em V12 — nunca lido de disco por L1
- `DeclarationKind::Class` tratado como `Struct` por V12 para
  `allow_adapter_structs`
- **`ImportKind` nunca deve conter variantes específicas de linguagem**
  — descreve mecânica estrutural, não sintaxe. V3 e V4 não usam
  `ImportKind` para distinguir linguagens — usam `target_layer` e
  `file.language()` respectivamente
- V4 usa `file.language()` para seleccionar a lista de símbolos
  proibidos — a lista vive em `impure-core.md` e no `impure_core.rs`,
  não na IR

---

## Critérios de Verificação

```
Dado ParsedFile com prompt_file_exists = false
Quando V1::check() for chamado
Então retorna Violation { rule_id: "V1", level: Error }

Dado ParsedFile com has_test_coverage = false e layer = L1
Quando V2::check() for chamado
Então retorna Violation { rule_id: "V2", level: Error }

Dado Import com target_layer = L3 em arquivo com layer = L2
Quando V3::check() for chamado
Então retorna Violation { rule_id: "V3", level: Error }
— V3 usa target_layer, nunca ImportKind

Dado Import { kind: ImportKind::Direct, target_layer: L3 }
em arquivo com layer = L2
Quando V3::check() for chamado
Então retorna Violation { rule_id: "V3", level: Error }
— ImportKind::Direct é Rust/TS/Python — V3 é agnóstico

Dado Import { kind: ImportKind::Named, target_layer: L3 }
em arquivo com layer = L2
Quando V3::check() for chamado
Então retorna Violation { rule_id: "V3", level: Error }
— ImportKind::Named pode ser TS ou Python — V3 é agnóstico

Dado Token { symbol: Cow::Borrowed("std::fs::read"), .. }
em arquivo L1 com language = Rust
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", level: Error }

Dado Token { symbol: Cow::Borrowed("os.path.join"), .. }
em arquivo L1 com language = Python
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", level: Error }
— V4 usa file.language() para seleccionar lista proibida

Dado Token { symbol: Cow::Owned("std::fs::read"), .. }
em arquivo L1 com language = Rust
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

Dado PublicInterface idêntica ao prompt_snapshot
Quando compute_delta() for chamado
Então InterfaceDelta.is_empty() == true

Dado InterfaceDelta com +fn check e -fn validate e +struct Delta
Quando describe() for chamado
Então retorna "+fn check, -fn validate, +struct Delta"

Dado InterfaceDelta com +class Foo e -interface Bar
Quando describe() for chamado
Então retorna "+class Foo, -interface Bar"

Dado Import { target_layer: L1, target_subdir: Some("internal") }
em arquivo com layer = L2
E "internal" não listado em [l1_ports]
Quando V9::check() for chamado
Então retorna Violation { rule_id: "V9", level: Error }

Dado Import { target_layer: Layer::Lab, line: 5 }
em arquivo com layer = L1
Quando V10::check() for chamado
Então retorna Violation { rule_id: "V10", level: Fatal }

Dado all_declared_traits = {"FileProvider"}
E all_implemented_traits = {}
Quando check_dangling_contracts() for chamado
Então retorna Violation { rule_id: "V11", level: Error }

Dado Declaration { kind: DeclarationKind::Enum, name: "Mode", line: 3 }
em arquivo com layer = L4
Quando V12::check() for chamado com WiringConfig::default()
Então retorna Violation { rule_id: "V12", level: Warning }

Dado Declaration { kind: DeclarationKind::Class, name: "Adapter", line: 5 }
em arquivo com layer = L4
Quando V12::check() for chamado com WiringConfig { allow_adapter_structs: true }
Então retorna vec![]
— Class tratado como Struct para allow_adapter_structs

Dado Declaration { kind: DeclarationKind::Interface, name: "Foo", line: 7 }
em arquivo com layer = L4
Quando V12::check() for chamado
Então retorna Violation { rule_id: "V12", level: Warning }

Dado Declaration { kind: DeclarationKind::TypeAlias, name: "Cfg", line: 9 }
em arquivo com layer = L4
Quando V12::check() for chamado
Então retorna Violation { rule_id: "V12", level: Warning }

Dado SourceError::Unreadable { path, reason }
Quando source_error_to_violation() for chamado em L4
Então retorna Violation { rule_id: "V0", level: Fatal,
      location: Location { path: Cow::Owned(path), .. } }

Dado Violation com level Fatal
Quando should_fail() for chamado independentemente de --fail-on
Então retorna true
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | parsed_file.rs, violation.rs, layer.rs |
| 2025-03-13 | Gap 2: prompt_file_exists, has_test_coverage, Import.target_layer, PromptHeader.current_hash | parsed_file.rs |
| 2025-03-13 | V6: PublicInterface, FunctionSignature, TypeSignature, InterfaceDelta, compute_delta | parsed_file.rs |
| 2026-03-14 | ADR-0004: lifetimes `<'a>`, ViolationLevel::Fatal, Cow<'a, str> em Token.symbol | parsed_file.rs, violation.rs |
| 2026-03-14 | Errata Cow: Token.symbol de &'a str para Cow<'a, str> | parsed_file.rs |
| 2026-03-14 | ADR-0005: Location.path de &'a Path para Cow<'a, Path> | violation.rs |
| 2026-03-14 | ADR-0006: Import.target_subdir para V9 | parsed_file.rs |
| 2026-03-16 | ADR-0007: Declaration, DeclarationKind, WiringConfig, campos novos em ParsedFile, V10/V11/V12 | parsed_file.rs, violation.rs |
| 2026-03-18 | ADR-0009 extensão: TypeKind/DeclarationKind com variantes OO; ImportKind::EsImport adicionado | parsed_file.rs |
| 2026-03-18 | ADR-0009 correcção: ImportKind reescrito como semântico (Direct/Glob/Alias/Named) — remove Use/ExternCrate/ModDecl/EsImport/PyImport que eram sintácticos; tabela de mapeamento por linguagem adicionada; nota em Token sobre V4 usar file.language(); restrição de agnósticidade de ImportKind documentada | parsed_file.rs |
| 2026-03-22 | ADR-0011: StaticDeclaration<'a>, HasStaticDeclarations<'a>, static_declarations em ParsedFile | parsed_file.rs, rule_traits.rs |
| 2026-03-22 | ADR-0012: L1AllowedExternal em entities/l1_allowed_external.rs | l1_allowed_external.rs |
