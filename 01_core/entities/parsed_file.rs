//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/violation-types.md
//! @prompt-hash 25d39a93
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;
use std::path::Path;

use crate::entities::layer::{Language, Layer};

// ── Import ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Use,
    ExternCrate,
    ModDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import<'a> {
    pub path: &'a str,  // sempre presente no buffer — &'a str puro
    pub line: usize,
    pub kind: ImportKind,
    /// Resolvido por L3 via crystalline.toml [layers].
    /// Layer::Unknown para crates externas — não gera violação V3.
    pub target_layer: Layer,
    /// Subdiretório de destino dentro da camada alvo.
    /// Resolvido por L3 via crystalline.toml [l1_ports].
    /// None se target_layer == Unknown (crate externa).
    /// Some("entities") se import aponta para 01_core/entities/.
    /// Some("internal") se import aponta para subdir não-porta.
    /// Usado por V9 para detectar imports fora das portas de L1.  (ADR-0006)
    pub target_subdir: Option<&'a str>,
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    CallExpression,
    MacroInvocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    /// FQN resolvido pelo RustParser (ADR-0004 + Errata).
    ///
    /// Cow::Borrowed(&'a str) — símbolo presente literalmente no buffer:
    ///   `std::fs::read(...)`  →  Borrowed("std::fs::read")
    ///
    /// Cow::Owned(String) — FQN construído por resolução de alias:
    ///   `use std::fs as f; f::read(...)`  →  Owned("std::fs::read")
    ///
    /// V4 trata ambos identicamente via Deref<Target = str> —
    /// compara &str sem conhecer a origem da string.
    pub symbol: Cow<'a, str>,
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

// ── Declaration (V12) ─────────────────────────────────────────────────────────

/// Declaração de tipo de nível superior num arquivo fonte.
/// Populado por RustParser para todos os arquivos —
/// V12 filtra por `layer == L4` internamente, não o parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'a> {
    pub kind: DeclarationKind,
    pub name: &'a str,
    pub line: usize,
}

/// Tipo de declaração de nível superior capturado pelo RustParser.
/// `impl Trait for Type` NÃO é capturado como Impl — apenas
/// `impl Type { ... }` sem trait gera DeclarationKind::Impl.
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
    Impl,
}

// ── WiringConfig (V12) ────────────────────────────────────────────────────────

/// Configuração de exceções para V12, injetada por L4 a partir de
/// crystalline.toml [wiring_exceptions]. V12 nunca lê o toml diretamente.
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

// ── PublicInterface (V6) ──────────────────────────────────────────────────────

/// Interface pública extraída do AST — agnóstica de linguagem.
/// Não inclui implementação, apenas contratos visíveis externamente.
/// Critério de igualdade: PartialEq derivado sobre a struct completa.
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
}

/// Critério de igualdade: name + params + return_type devem ser
/// todos iguais. Mudança em qualquer campo é quebra de contrato.
/// PartialEq derivado sobre a struct completa — nunca comparar só name.
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

// ── InterfaceDelta ─────────────────────────────────────────────────────────────

/// Diferença entre interface atual e snapshot do prompt.
/// Produzida por compute_delta() em prompt_stale.rs — função pura, zero I/O.
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
            parts.push(format!("+type {}", t.name));
        }
        for t in &self.removed_types {
            parts.push(format!("-type {}", t.name));
        }
        for r in &self.added_reexports {
            parts.push(format!("+reexport {r}"));
        }
        for r in &self.removed_reexports {
            parts.push(format!("-reexport {r}"));
        }
        if parts.is_empty() { "(no diff)".to_string() } else { parts.join(", ") }
    }

    pub fn is_empty(&self) -> bool {
        self.added_functions.is_empty()
            && self.removed_functions.is_empty()
            && self.added_types.is_empty()
            && self.removed_types.is_empty()
            && self.added_reexports.is_empty()
            && self.removed_reexports.is_empty()
    }
}

// ── PromptHeader ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptHeader<'a> {
    pub prompt_path: &'a str,
    pub prompt_hash: Option<&'a str>,  // declarado no header do arquivo
    pub current_hash: Option<String>,  // EXCEÇÃO: SHA256 calculado do disco
    pub layer: Layer,
    pub updated: Option<&'a str>,
}

// ── Trait Implementations for Rules (via entities::rule_traits) ─────────────
// Direção correta: entities → contracts (nunca entities → rules)

use crate::entities::rule_traits::{
    HasCoverage, HasHashes, HasImports, HasPromptFilesystem,
    HasPublicInterface, HasPubLeak, HasTokens, HasWiringPurity,
};

impl<'a> HasPromptFilesystem<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn prompt_file_exists(&self) -> bool {
        self.prompt_file_exists
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasCoverage<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn has_test_coverage(&self) -> bool {
        self.has_test_coverage
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasImports<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn imports(&self) -> &[Import<'a>] {
        &self.imports
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasTokens<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn tokens(&self) -> &[Token<'a>] {
        &self.tokens
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasHashes<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasPublicInterface<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn public_interface(&self) -> &PublicInterface<'a> {
        &self.public_interface
    }
    fn prompt_snapshot(&self) -> Option<&PublicInterface<'a>> {
        self.prompt_snapshot.as_ref()
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasPubLeak<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn imports(&self) -> &[Import<'a>] {
        &self.imports
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

impl<'a> HasWiringPurity<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn declarations(&self) -> &[Declaration<'a>] {
        &self.declarations
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

// ── ParsedFile ────────────────────────────────────────────────────────────────

/// Intermediate representation consumed by all V1–V6 rules.
/// All fields are populated by L3 before reaching L1.
/// L1 rules only read — never derive, never allocate (except Cow::Owned
/// in tokens with resolved alias, transparent to L1 via Deref).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile<'a> {
    pub path: &'a Path,
    pub layer: Layer,
    pub language: Language,

    /// For V1: None means header is absent.
    pub prompt_header: Option<PromptHeader<'a>>,
    /// For V1: true if prompt_header.prompt_path exists in 00_nucleo/.
    pub prompt_file_exists: bool,

    /// For V2: true if #[cfg(test)] is present in AST or foo_test.rs exists adjacent.
    pub has_test_coverage: bool,

    /// For V3: each Import carries its resolved target_layer.
    pub imports: Vec<Import<'a>>,

    /// For V4: call expressions and macro invocations extracted from AST.
    pub tokens: Vec<Token<'a>>,

    /// For V6: public interface extracted from the AST by L3 (RustParser).
    pub public_interface: PublicInterface<'a>,

    /// For V6: snapshot of the public interface registered in the origin prompt.
    /// None if the prompt has no Interface Snapshot section yet.
    pub prompt_snapshot: Option<PublicInterface<'a>>,

    /// For V11: trait names declared `pub` in contracts/ files of L1.
    /// Populated by L3 (RustParser) for files with layer == L1 and subdir == "contracts".
    pub declared_traits: Vec<&'a str>,

    /// For V11: trait names referenced in `impl <Trait> for` blocks in L2/L3 files.
    /// Populated by L3 (RustParser) for files with layer == L2 | L3.
    pub implemented_traits: Vec<&'a str>,

    /// For V12: top-level struct/enum/impl-without-trait declarations.
    /// Populated by L3 (RustParser) for all files — V12 filters by layer == L4 internally.
    pub declarations: Vec<Declaration<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::path::Path;

    fn base_file() -> ParsedFile<'static> {
        ParsedFile {
            path: Path::new("01_core/foo.rs"),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: None,
            prompt_file_exists: false,
            has_test_coverage: false,
            imports: vec![],
            tokens: vec![],
            public_interface: PublicInterface::empty(),
            prompt_snapshot: None,
            declared_traits: vec![],
            implemented_traits: vec![],
            declarations: vec![],
        }
    }

    #[test]
    fn parsed_file_clone_and_eq() {
        let f = base_file();
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn prompt_header_hash_comparison() {
        let header = PromptHeader {
            prompt_path: "00_nucleo/prompts/linter-core.md",
            prompt_hash: Some("a3f8c2d1"),
            current_hash: Some("b9e4f7a2".to_string()),
            layer: Layer::L1,
            updated: Some("2026-03-13"),
        };
        // V5 detects drift by comparing these two fields
        assert_ne!(header.prompt_hash, header.current_hash.as_deref());
    }

    #[test]
    fn prompt_header_no_drift_when_hashes_match() {
        let header = PromptHeader {
            prompt_path: "00_nucleo/prompts/linter-core.md",
            prompt_hash: Some("a3f8c2d1"),
            current_hash: Some("a3f8c2d1".to_string()),
            layer: Layer::L1,
            updated: None,
        };
        assert_eq!(header.prompt_hash, header.current_hash.as_deref());
    }

    #[test]
    fn import_unknown_layer_for_external_crate() {
        let import = Import {
            path: "reqwest::Client",
            line: 3,
            kind: ImportKind::Use,
            target_layer: Layer::Unknown,
            target_subdir: None,
        };
        assert_eq!(import.target_layer, Layer::Unknown);
    }

    #[test]
    fn token_call_expression() {
        let token = Token {
            symbol: Cow::Borrowed("std::fs::read"),
            line: 10,
            column: 4,
            kind: TokenKind::CallExpression,
        };
        assert_eq!(token.kind, TokenKind::CallExpression);
        assert_eq!(&*token.symbol, "std::fs::read");
    }

    #[test]
    fn parsed_file_with_imports_and_tokens() {
        let mut f = base_file();
        f.imports.push(Import {
            path: "crate::shell::api",
            line: 2,
            kind: ImportKind::Use,
            target_layer: Layer::L2,
            target_subdir: None,
        });
        f.tokens.push(Token {
            symbol: Cow::Borrowed("std::net::TcpStream"),
            line: 7,
            column: 0,
            kind: TokenKind::CallExpression,
        });
        assert_eq!(f.imports.len(), 1);
        assert_eq!(f.tokens.len(), 1);
    }

    // ── Declaration / DeclarationKind ─────────────────────────────────────────

    #[test]
    fn declaration_kind_variants_are_distinct() {
        assert_ne!(DeclarationKind::Struct, DeclarationKind::Enum);
        assert_ne!(DeclarationKind::Enum, DeclarationKind::Impl);
        assert_ne!(DeclarationKind::Struct, DeclarationKind::Impl);
    }

    #[test]
    fn declaration_clone_and_eq() {
        let d = Declaration { kind: DeclarationKind::Struct, name: "Adapter", line: 5 };
        assert_eq!(d.clone(), d);
    }

    #[test]
    fn declarations_field_accepts_multiple_kinds() {
        let mut f = base_file();
        f.declarations.push(Declaration { kind: DeclarationKind::Struct, name: "Adapter", line: 2 });
        f.declarations.push(Declaration { kind: DeclarationKind::Enum, name: "Mode", line: 5 });
        f.declarations.push(Declaration { kind: DeclarationKind::Impl, name: "Adapter", line: 8 });
        assert_eq!(f.declarations.len(), 3);
        assert_eq!(f.declarations[1].kind, DeclarationKind::Enum);
    }

    #[test]
    fn declaration_name_is_borrowed_from_source() {
        // name is &'a str — no allocation
        let d = Declaration { kind: DeclarationKind::Impl, name: "L3HashRewriter", line: 10 };
        assert_eq!(d.name, "L3HashRewriter");
        assert_eq!(d.line, 10);
    }

    // ── WiringConfig ──────────────────────────────────────────────────────────

    #[test]
    fn wiring_config_default_allows_adapter_structs() {
        let cfg = WiringConfig::default();
        assert!(cfg.allow_adapter_structs);
    }

    #[test]
    fn wiring_config_can_disable_adapter_structs() {
        let cfg = WiringConfig { allow_adapter_structs: false };
        assert!(!cfg.allow_adapter_structs);
    }

    // ── declared_traits / implemented_traits ──────────────────────────────────

    #[test]
    fn declared_traits_empty_by_default() {
        let f = base_file();
        assert!(f.declared_traits.is_empty());
        assert!(f.implemented_traits.is_empty());
    }

    #[test]
    fn declared_traits_accepts_static_str_slices() {
        let mut f = base_file();
        f.declared_traits.push("FileProvider");
        f.declared_traits.push("LanguageParser");
        assert_eq!(f.declared_traits.len(), 2);
        assert_eq!(f.declared_traits[0], "FileProvider");
    }

    #[test]
    fn implemented_traits_distinct_from_declared() {
        let mut f = base_file();
        f.declared_traits.push("FileProvider");
        f.implemented_traits.push("LanguageParser");
        assert_ne!(f.declared_traits, f.implemented_traits);
    }

    // ── HasWiringPurity impl on ParsedFile ────────────────────────────────────

    #[test]
    fn has_wiring_purity_returns_layer_and_declarations() {
        use crate::entities::rule_traits::HasWiringPurity;
        let mut f = base_file();
        f.layer = Layer::L4;
        f.declarations.push(Declaration { kind: DeclarationKind::Enum, name: "Mode", line: 3 });
        assert_eq!(HasWiringPurity::layer(&f), &Layer::L4);
        assert_eq!(HasWiringPurity::declarations(&f).len(), 1);
        assert_eq!(HasWiringPurity::declarations(&f)[0].kind, DeclarationKind::Enum);
    }

    #[test]
    fn has_wiring_purity_empty_declarations_for_non_l4() {
        use crate::entities::rule_traits::HasWiringPurity;
        let f = base_file(); // layer == L1, declarations empty
        assert_eq!(HasWiringPurity::layer(&f), &Layer::L1);
        assert!(HasWiringPurity::declarations(&f).is_empty());
    }
}
