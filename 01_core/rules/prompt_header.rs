//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-header.md
//! @prompt-hash cf6a7071
//! @layer L1
//! @updated 2026-03-13

use crate::entities::parsed_file::ParsedFile;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V1 — Missing or unresolvable @prompt header.
/// Fires when prompt_header is absent OR when the referenced prompt file
/// does not exist in 00_nucleo/ (prompt_file_exists == false).
pub fn check(file: &ParsedFile) -> Vec<Violation> {
    let has_valid_header = file.prompt_header.is_some() && file.prompt_file_exists;

    if has_valid_header {
        return vec![];
    }

    vec![Violation {
        rule_id: "V1".to_string(),
        level: ViolationLevel::Error,
        message: "Arquivo Cristalino sem linhagem causal @prompt encontrada".to_string(),
        location: Location {
            path: file.path.clone(),
            line: 1,
            column: 0,
        },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{ParsedFile, PromptHeader};
    use std::path::PathBuf;

    fn base_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("01_core/foo.rs"),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: None,
            prompt_file_exists: false,
            has_test_coverage: true,
            imports: vec![],
            tokens: vec![],
        }
    }

    fn valid_header() -> PromptHeader {
        PromptHeader {
            prompt_path: "00_nucleo/prompts/linter-core.md".to_string(),
            prompt_hash: Some("a3f8c2d1".to_string()),
            current_hash: Some("a3f8c2d1".to_string()),
            layer: Layer::L1,
            updated: Some("2026-03-13".to_string()),
        }
    }

    #[test]
    fn no_violation_when_header_present_and_file_exists() {
        let mut file = base_file();
        file.prompt_header = Some(valid_header());
        file.prompt_file_exists = true;
        assert!(check(&file).is_empty());
    }

    #[test]
    fn violation_when_header_absent() {
        let file = base_file(); // prompt_header: None
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V1");
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }

    #[test]
    fn violation_when_header_present_but_file_missing() {
        let mut file = base_file();
        file.prompt_header = Some(valid_header());
        file.prompt_file_exists = false; // prompt file not found in 00_nucleo/
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V1");
    }

    #[test]
    fn violation_points_to_line_1() {
        let file = base_file();
        let violations = check(&file);
        assert_eq!(violations[0].location.line, 1);
    }
}
