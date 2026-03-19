//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/rule-traits.md
//! @prompt-hash 1b36408c
//! @layer L1
//! @updated 2026-03-16

use std::path::Path;

use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::{Declaration, Import, PromptHeader, PublicInterface, Token};

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
}
