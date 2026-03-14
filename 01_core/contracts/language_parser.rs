//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/language-parser.md
//! @prompt-hash 3e245443
//! @layer L1
//! @updated 2026-03-13

use crate::contracts::file_provider::SourceFile;
use crate::contracts::parse_error::ParseError;
use crate::entities::parsed_file::ParsedFile;

/// Boundary between L3 (tree-sitter grammar) and L1 (rules).
/// L3 implements, L1 consumes.
/// Receives the full SourceFile — not just &str — because the parser
/// needs path to determine Layer and Language before invoking the grammar.
pub trait LanguageParser {
    fn parse(&self, file: SourceFile) -> Result<ParsedFile, ParseError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::ParsedFile;
    use std::path::PathBuf;

    struct MockParser {
        result: Result<ParsedFile, ParseError>,
    }

    impl LanguageParser for MockParser {
        fn parse(&self, _file: SourceFile) -> Result<ParsedFile, ParseError> {
            self.result.clone()
        }
    }

    fn valid_parsed_file(path: &str) -> ParsedFile {
        ParsedFile {
            path: PathBuf::from(path),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: None,
            prompt_file_exists: false,
            has_test_coverage: false,
            imports: vec![],
            tokens: vec![],
            public_interface: crate::entities::parsed_file::PublicInterface::empty(),
            prompt_snapshot: None,
        }
    }

    fn make_source_file(path: &str) -> SourceFile {
        SourceFile {
            path: PathBuf::from(path),
            content: "fn main() {}".to_string(),
            language: Language::Rust,
            layer: Layer::L1,
            has_adjacent_test: false,
        }
    }

    #[test]
    fn mock_parser_returns_ok_for_valid_source() {
        let parser = MockParser {
            result: Ok(valid_parsed_file("01_core/foo.rs")),
        };
        let file = make_source_file("01_core/foo.rs");
        let result = parser.parse(file);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_parser_returns_err_for_invalid_source() {
        let parser = MockParser {
            result: Err(ParseError::SyntaxError {
                path: PathBuf::from("01_core/bad.rs"),
                line: 3,
                column: 5,
                message: "unexpected token".to_string(),
            }),
        };
        let file = make_source_file("01_core/bad.rs");
        let result = parser.parse(file);
        assert!(result.is_err());
    }
}
