//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/parse-error.md
//! @prompt-hash a7d7a5ef
//! @layer L1
//! @updated 2026-03-13

use std::path::PathBuf;

use crate::entities::layer::Language;

/// Domain error for grammar failures. Never carries std::io::Error —
/// L3 converts IO failures before crossing the L3→L1 boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// tree-sitter parsed the file but found an ERROR node in the AST.
    SyntaxError {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },
    /// File language has no registered grammar.
    UnsupportedLanguage {
        path: PathBuf,
        language: Language,
    },
    /// Empty content — nothing to parse.
    EmptySource {
        path: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Language;

    #[test]
    fn syntax_error_clone_and_eq() {
        let a = ParseError::SyntaxError {
            path: PathBuf::from("01_core/foo.rs"),
            line: 5,
            column: 3,
            message: "unexpected token".to_string(),
        };
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn syntax_error_location() {
        let err = ParseError::SyntaxError {
            path: PathBuf::from("foo.rs"),
            line: 5,
            column: 3,
            message: "err".to_string(),
        };
        if let ParseError::SyntaxError { line, column, .. } = err {
            assert_eq!(line, 5);
            assert_eq!(column, 3);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn unsupported_language_debug_readable() {
        let err = ParseError::UnsupportedLanguage {
            path: PathBuf::from("foo.ts"),
            language: Language::Unknown,
        };
        let s = format!("{:?}", err);
        assert!(s.contains("UnsupportedLanguage"));
    }

    #[test]
    fn empty_source_clone_and_eq() {
        let a = ParseError::EmptySource { path: PathBuf::from("empty.rs") };
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn variants_are_distinct() {
        let a = ParseError::EmptySource { path: PathBuf::from("x.rs") };
        let b = ParseError::UnsupportedLanguage {
            path: PathBuf::from("x.rs"),
            language: Language::Unknown,
        };
        assert_ne!(a, b);
    }
}
