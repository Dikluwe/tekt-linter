//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/rule-traits.md
//! @prompt-hash 7e9688c7
//! @layer L1
//! @updated 2026-03-22

use std::path::Path;

use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{Declaration, Import, ModuleDecl, PromptHeader, PublicInterface, StaticDeclaration, Token};

// ── V1 ────────────────────────────────────────────────────────────────────────

/// Para V1 — verifica presença e validade do @prompt header.
pub trait HasPromptFilesystem<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn prompt_file_exists(&self) -> bool;
    fn path(&self) -> &'a Path;
}

// ── V2 ────────────────────────────────────────────────────────────────────────

/// Para V2 — verifica cobertura de testes em L1.
pub trait HasCoverage<'a> {
    fn layer(&self) -> &Layer;
    fn has_test_coverage(&self) -> bool;
    fn path(&self) -> &'a Path;
}

// ── V3 ────────────────────────────────────────────────────────────────────────

/// Para V3 — verifica imports proibidos por camada.
pub trait HasImports<'a> {
    fn layer(&self) -> &Layer;
    fn imports(&self) -> &[Import<'a>];
    fn path(&self) -> &'a Path;
}

// ── V4 ────────────────────────────────────────────────────────────────────────

/// Para V4 — verifica tokens de I/O em L1.
pub trait HasTokens<'a> {
    fn layer(&self) -> &Layer;
    fn tokens(&self) -> &[Token<'a>];
    fn path(&self) -> &'a Path;
    fn language(&self) -> &Language;
}

// ── V5 ────────────────────────────────────────────────────────────────────────

/// Para V5 — verifica drift de hash entre prompt e código.
pub trait HasHashes<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn path(&self) -> &'a Path;
}

// ── V6 ────────────────────────────────────────────────────────────────────────

/// Para V6 — verifica drift de interface pública.
pub trait HasPublicInterface<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn public_interface(&self) -> &PublicInterface<'a>;
    fn prompt_snapshot(&self) -> Option<&PublicInterface<'a>>;
    fn path(&self) -> &'a Path;
}

// ── V9 ────────────────────────────────────────────────────────────────────────

/// Para V9 — verifica imports de subdiretórios não-porta de L1.
pub trait HasPubLeak<'a> {
    fn layer(&self) -> &Layer;
    fn imports(&self) -> &[Import<'a>];
    fn path(&self) -> &'a Path;
}

// ── ModuleDecls (ADR-0013) ────────────────────────────────────────────────────

/// Para regras que inspeccionam estrutura de módulos (futura).
/// Declarada agora para fechar o modelo — ADR-0013.
/// TypeScript e Python produzem sempre `vec![]`.
pub trait HasModuleDecls<'a> {
    fn module_decls(&self) -> &[ModuleDecl<'a>];
}

// ── V13 ───────────────────────────────────────────────────────────────────────

/// Para V13 — verifica static declarations mutáveis em L1.
pub trait HasStaticDeclarations<'a> {
    fn layer(&self) -> &Layer;
    fn static_declarations(&self) -> &[StaticDeclaration<'a>];
    fn path(&self) -> &'a Path;
}

// ── V15 ───────────────────────────────────────────────────────────────────────

/// Para V15 — verifica unicidade da linhagem @prompt (um ficheiro, um prompt).
/// `prompt_refs()` expõe todos os valores `@prompt` do bloco de doc-header,
/// em ordem. `len() >= 2` em L1–L4 é violação.
pub trait HasPromptRefs<'a> {
    fn layer(&self) -> &Layer;
    fn prompt_refs(&self) -> &[&'a str];
    fn path(&self) -> &'a Path;
}

// ── V12 ───────────────────────────────────────────────────────────────────────

/// Para V12 — verifica declarações de tipo em L4.
///
/// `declarations()` expõe struct/enum/impl-sem-trait de nível superior.
/// V12 filtra por `layer() == Layer::L4` internamente.
/// `impl Trait for Type` não aparece em `declarations()` —
/// o RustParser só captura `impl Type { ... }` sem trait.
pub trait HasWiringPurity<'a> {
    fn layer(&self) -> &Layer;
    fn declarations(&self) -> &[Declaration<'a>];
    fn path(&self) -> &'a Path;
}


// ── V16–V20 (Decisões Mecânicas — ADR-0016) ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrutineeForm {
    Path,
    FieldAccess,
    MethodCall,
    Index,
    Literal,
    Tuple,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyForm {
    ErrorBarrier,
    MessageProducer,
    EnumPath,
    LiteralNeutral,
    LiteralOther,
    Call,
    EmptyBlock,
    Continue,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionExpr<'a> {
    pub snippet_scrutinee: &'a str,
    pub scrutinee_form: ScrutineeForm,
    pub arms: Vec<DecisionArm<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionArm<'a> {
    pub pattern_snippet: &'a str,
    pub is_catchall: bool,
    pub bound_ident_used_in_body: bool,
    pub qualified_prefixes: Vec<&'a str>,
    pub has_guard: bool,
    pub guard_is_compound: bool,
    pub pattern_is_range: bool,
    pub pattern_depth: u8,
    pub or_alternatives: u16,
    pub body_form: BodyForm,
    pub body_snippet: &'a str,
    pub line: usize,
    pub column: usize,
}

/// Para V16–V20 — inspeciona braços de decisão (match/switch/case).
pub trait HasDecisionArms<'a> {
    fn layer(&self) -> &Layer;
    fn decision_exprs(&self) -> &[DecisionExpr<'a>];
    fn path(&self) -> &'a Path;
    fn language(&self) -> &Language;
}

/// Termo idiomático para catch-all na linguagem do arquivo analisado (ADR-0016 §5).
pub fn decision_arm_term_for(language: &Language) -> &'static str {
    match language {
        Language::Rust => "wildcard `_ =>`",
        Language::Python => "`case _`",
        Language::TypeScript => "cláusula `default:`",
        Language::Go => "cláusula `default:`",
        Language::Zig => "—",
        Language::C | Language::Cpp | Language::Java | Language::Elixir | Language::Unknown => "wildcard",
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::path::Path;

    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{DeclarationKind, ImportKind, TokenKind};

    // ── Minimal mocks verifying each trait is implementable independently ──

    struct MockV1 {
        path: &'static Path,
    }
    impl HasPromptFilesystem<'static> for MockV1 {
        fn prompt_header(&self) -> Option<&PromptHeader<'static>> { None }
        fn prompt_file_exists(&self) -> bool { false }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV2 {
        layer: Layer,
        path: &'static Path,
    }
    impl HasCoverage<'static> for MockV2 {
        fn layer(&self) -> &Layer { &self.layer }
        fn has_test_coverage(&self) -> bool { true }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV3 {
        layer: Layer,
        imports: Vec<Import<'static>>,
        path: &'static Path,
    }
    impl HasImports<'static> for MockV3 {
        fn layer(&self) -> &Layer { &self.layer }
        fn imports(&self) -> &[Import<'static>] { &self.imports }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV4 {
        layer: Layer,
        language: Language,
        tokens: Vec<Token<'static>>,
        path: &'static Path,
    }
    impl HasTokens<'static> for MockV4 {
        fn layer(&self) -> &Layer { &self.layer }
        fn tokens(&self) -> &[Token<'static>] { &self.tokens }
        fn path(&self) -> &'static Path { self.path }
        fn language(&self) -> &Language { &self.language }
    }

    struct MockV5 {
        path: &'static Path,
    }
    impl HasHashes<'static> for MockV5 {
        fn prompt_header(&self) -> Option<&PromptHeader<'static>> { None }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV6 {
        iface: PublicInterface<'static>,
        path: &'static Path,
    }
    impl HasPublicInterface<'static> for MockV6 {
        fn prompt_header(&self) -> Option<&PromptHeader<'static>> { None }
        fn public_interface(&self) -> &PublicInterface<'static> { &self.iface }
        fn prompt_snapshot(&self) -> Option<&PublicInterface<'static>> { None }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV9 {
        layer: Layer,
        imports: Vec<Import<'static>>,
        path: &'static Path,
    }
    impl HasPubLeak<'static> for MockV9 {
        fn layer(&self) -> &Layer { &self.layer }
        fn imports(&self) -> &[Import<'static>] { &self.imports }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV12 {
        layer: Layer,
        declarations: Vec<Declaration<'static>>,
        path: &'static Path,
    }
    impl HasWiringPurity<'static> for MockV12 {
        fn layer(&self) -> &Layer { &self.layer }
        fn declarations(&self) -> &[Declaration<'static>] { &self.declarations }
        fn path(&self) -> &'static Path { self.path }
    }

    struct MockV15 {
        layer: Layer,
        refs: Vec<&'static str>,
        path: &'static Path,
    }
    impl HasPromptRefs<'static> for MockV15 {
        fn layer(&self) -> &Layer { &self.layer }
        fn prompt_refs(&self) -> &[&'static str] { &self.refs }
        fn path(&self) -> &'static Path { self.path }
    }

    #[test]
    fn mock_v1_implements_has_prompt_filesystem() {
        let m = MockV1 { path: Path::new("foo.rs") };
        assert!(!m.prompt_file_exists());
        assert!(m.prompt_header().is_none());
    }

    #[test]
    fn mock_v2_implements_has_coverage() {
        let m = MockV2 { layer: Layer::L1, path: Path::new("foo.rs") };
        assert_eq!(m.layer(), &Layer::L1);
        assert!(m.has_test_coverage());
    }

    #[test]
    fn mock_v3_implements_has_imports() {
        let m = MockV3 { layer: Layer::L2, imports: vec![], path: Path::new("foo.rs") };
        assert_eq!(m.layer(), &Layer::L2);
        assert!(m.imports().is_empty());
    }

    #[test]
    fn mock_v4_implements_has_tokens() {
        let tok = Token {
            symbol: Cow::Borrowed("std::fs::read"),
            line: 1,
            column: 0,
            kind: TokenKind::CallExpression,
        };
        let m = MockV4 { layer: Layer::L1, language: Language::Rust, tokens: vec![tok], path: Path::new("foo.rs") };
        assert_eq!(m.tokens().len(), 1);
    }

    #[test]
    fn mock_v5_implements_has_hashes() {
        let m = MockV5 { path: Path::new("foo.rs") };
        assert!(m.prompt_header().is_none());
    }

    #[test]
    fn mock_v6_implements_has_public_interface() {
        let m = MockV6 { iface: PublicInterface::empty(), path: Path::new("foo.rs") };
        assert!(m.public_interface().functions.is_empty());
        assert!(m.public_interface().types.is_empty());
        assert!(m.prompt_snapshot().is_none());
    }

    #[test]
    fn mock_v9_implements_has_pub_leak() {
        let imp = Import {
            path: "crate::entities::Layer",
            line: 3,
            kind: ImportKind::Direct,
            target_layer: Layer::L1,
            target_subdir: Some("entities"),
            is_test_origin: false,
        };
        let m = MockV9 { layer: Layer::L2, imports: vec![imp], path: Path::new("foo.rs") };
        assert_eq!(m.imports().len(), 1);
    }

    #[test]
    fn mock_v12_implements_has_wiring_purity() {
        let decl = Declaration { kind: DeclarationKind::Enum, name: "OutputMode", line: 3 };
        let m = MockV12 {
            layer: Layer::L4,
            declarations: vec![decl],
            path: Path::new("04_wiring/main.rs"),
        };
        assert_eq!(m.layer(), &Layer::L4);
        assert_eq!(m.declarations().len(), 1);
        assert_eq!(m.declarations()[0].kind, DeclarationKind::Enum);
        assert_eq!(m.declarations()[0].name, "OutputMode");
    }

    #[test]
    fn mock_v12_empty_declarations_for_non_l4() {
        let m = MockV12 {
            layer: Layer::L3,
            declarations: vec![],
            path: Path::new("03_infra/walker.rs"),
        };
        assert_eq!(m.layer(), &Layer::L3);
        assert!(m.declarations().is_empty());
    }

    #[test]
    fn mock_v15_implements_has_prompt_refs() {
        let m = MockV15 {
            layer: Layer::L2,
            refs: vec!["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"],
            path: Path::new("02_shell/cli.rs"),
        };
        assert_eq!(m.layer(), &Layer::L2);
        assert_eq!(m.prompt_refs().len(), 2);
        assert_eq!(m.path(), Path::new("02_shell/cli.rs"));
    }

    struct MockV16 {
        layer: Layer,
        language: Language,
        exprs: Vec<DecisionExpr<'static>>,
        path: &'static Path,
    }
    impl HasDecisionArms<'static> for MockV16 {
        fn layer(&self) -> &Layer { &self.layer }
        fn decision_exprs(&self) -> &[DecisionExpr<'static>] { &self.exprs }
        fn path(&self) -> &'static Path { self.path }
        fn language(&self) -> &Language { &self.language }
    }

    #[test]
    fn mock_v16_implements_has_decision_arms() {
        let m = MockV16 {
            layer: Layer::L1,
            language: Language::Rust,
            exprs: vec![],
            path: Path::new("01_core/entities/layer.rs"),
        };
        assert_eq!(m.layer(), &Layer::L1);
        assert_eq!(m.language(), &Language::Rust);
        assert!(m.decision_exprs().is_empty());
    }

    #[test]
    fn decision_arm_terms_are_idiomatic() {
        assert_eq!(decision_arm_term_for(&Language::Rust), "wildcard `_ =>`");
        assert_eq!(decision_arm_term_for(&Language::Python), "`case _`");
        assert_eq!(decision_arm_term_for(&Language::TypeScript), "cláusula `default:`");
        assert_eq!(decision_arm_term_for(&Language::Go), "cláusula `default:`");
        assert_eq!(decision_arm_term_for(&Language::Zig), "—");
    }

}
