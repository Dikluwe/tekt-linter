//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/violation-types.md
//! @prompt-hash 9b91c41b
//! @layer L1
//! @updated 2026-03-13

use std::path::PathBuf;

use crate::entities::layer::{Language, Layer};

// ── Import ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Use,
    ExternCrate,
    ModDecl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub path: String,
    pub line: usize,
    pub kind: ImportKind,
    /// Resolved by L3 (RustParser) via crystalline.toml prefix matching.
    /// Layer::Unknown for external crates.
    pub target_layer: Layer,
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    CallExpression,
    MacroInvocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub symbol: String,
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

// ── PromptHeader ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptHeader {
    pub prompt_path: String,
    /// Hash declared in the file header (@prompt-hash).
    pub prompt_hash: Option<String>,
    /// Hash of the actual prompt file in 00_nucleo/, populated by L3 via PromptReader::read_hash().
    /// None if the prompt file does not exist.
    pub current_hash: Option<String>,
    pub layer: Layer,
    pub updated: Option<String>,
}

// ── Trait Implementations for Rules (OCP) ───────────────────────────────────

impl crate::rules::prompt_header::HasPromptFilesystem for ParsedFile {
    fn prompt_header(&self) -> Option<&PromptHeader> {
        self.prompt_header.as_ref()
    }
    fn prompt_file_exists(&self) -> bool {
        self.prompt_file_exists
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl crate::rules::test_file::HasCoverage for ParsedFile {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn has_test_coverage(&self) -> bool {
        self.has_test_coverage
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl crate::rules::forbidden_import::HasImports for ParsedFile {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn imports(&self) -> &[Import] {
        &self.imports
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl crate::rules::impure_core::HasTokens for ParsedFile {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn tokens(&self) -> &[Token] {
        &self.tokens
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl crate::rules::prompt_drift::HasHashes for ParsedFile {
    fn prompt_header(&self) -> Option<&PromptHeader> {
        self.prompt_header.as_ref()
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

// ── ParsedFile ────────────────────────────────────────────────────────────────

/// Intermediate representation consumed by all V1–V5 rules.
/// All fields are populated by L3 before reaching L1.
/// L1 rules only read — never derive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub layer: Layer,
    pub language: Language,

    /// For V1: None means header is absent.
    pub prompt_header: Option<PromptHeader>,
    /// For V1: true if prompt_header.prompt_path exists in 00_nucleo/.
    /// Populated by L3 via PromptReader::exists().
    pub prompt_file_exists: bool,

    /// For V2: true if #[cfg(test)] is present in AST or foo_test.rs exists adjacent.
    /// Populated by L3 (FileWalker + LanguageParser).
    pub has_test_coverage: bool,

    /// For V3: each Import carries its resolved target_layer.
    pub imports: Vec<Import>,

    /// For V4: call expressions and macro invocations extracted from AST.
    pub tokens: Vec<Token>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("01_core/foo.rs"),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: None,
            prompt_file_exists: false,
            has_test_coverage: false,
            imports: vec![],
            tokens: vec![],
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
            prompt_path: "00_nucleo/prompts/linter-core.md".to_string(),
            prompt_hash: Some("a3f8c2d1".to_string()),
            current_hash: Some("b9e4f7a2".to_string()),
            layer: Layer::L1,
            updated: Some("2026-03-13".to_string()),
        };
        // V5 detects drift by comparing these two fields
        assert_ne!(header.prompt_hash, header.current_hash);
    }

    #[test]
    fn prompt_header_no_drift_when_hashes_match() {
        let header = PromptHeader {
            prompt_path: "00_nucleo/prompts/linter-core.md".to_string(),
            prompt_hash: Some("a3f8c2d1".to_string()),
            current_hash: Some("a3f8c2d1".to_string()),
            layer: Layer::L1,
            updated: None,
        };
        assert_eq!(header.prompt_hash, header.current_hash);
    }

    #[test]
    fn import_unknown_layer_for_external_crate() {
        let import = Import {
            path: "reqwest::Client".to_string(),
            line: 3,
            kind: ImportKind::Use,
            target_layer: Layer::Unknown,
        };
        assert_eq!(import.target_layer, Layer::Unknown);
    }

    #[test]
    fn token_call_expression() {
        let token = Token {
            symbol: "std::fs::read".to_string(),
            line: 10,
            column: 4,
            kind: TokenKind::CallExpression,
        };
        assert_eq!(token.kind, TokenKind::CallExpression);
        assert_eq!(token.symbol, "std::fs::read");
    }

    #[test]
    fn parsed_file_with_imports_and_tokens() {
        let mut f = base_file();
        f.imports.push(Import {
            path: "crate::shell::api".to_string(),
            line: 2,
            kind: ImportKind::Use,
            target_layer: Layer::L2,
        });
        f.tokens.push(Token {
            symbol: "std::net::TcpStream".to_string(),
            line: 7,
            column: 0,
            kind: TokenKind::CallExpression,
        });
        assert_eq!(f.imports.len(), 1);
        assert_eq!(f.tokens.len(), 1);
    }
}
