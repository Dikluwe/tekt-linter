//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-header.md
//! @prompt-hash 7cd76f6d
//! @layer L1
//! @updated 2026-03-14

use crate::entities::parsed_file::PromptHeader;
use crate::entities::violation::{Location, Violation, ViolationLevel};
use std::path::Path;

pub trait HasPromptFilesystem {
    fn prompt_header(&self) -> Option<&PromptHeader>;
    fn prompt_file_exists(&self) -> bool;
    fn path(&self) -> &Path;
}

/// V1 — Missing or unresolvable @prompt header.
/// Fires when prompt_header is absent OR when the referenced prompt file
/// does not exist in 00_nucleo/ (prompt_file_exists == false).
pub fn check<T: HasPromptFilesystem>(file: &T) -> Vec<Violation> {
    let has_valid_header = file.prompt_header().is_some() && file.prompt_file_exists();

    if has_valid_header {
        return vec![];
    }

    vec![Violation {
        rule_id: "V1".to_string(),
        level: ViolationLevel::Error,
        message: "Arquivo Cristalino sem linhagem causal @prompt encontrada".to_string(),
        location: Location {
            path: file.path().to_path_buf(),
            line: 1,
            column: 0,
        },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::PromptHeader;
    use std::path::{Path, PathBuf};

    struct MockFile {
        header: Option<PromptHeader>,
        exists: bool,
        path: PathBuf,
    }

    impl HasPromptFilesystem for MockFile {
        fn prompt_header(&self) -> Option<&PromptHeader> {
            self.header.as_ref()
        }
        fn prompt_file_exists(&self) -> bool {
            self.exists
        }
        fn path(&self) -> &Path {
            &self.path
        }
    }

    fn base_file() -> MockFile {
        MockFile {
            header: None,
            exists: false,
            path: PathBuf::from("01_core/foo.rs"),
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
        file.header = Some(valid_header());
        file.exists = true;
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
        file.header = Some(valid_header());
        file.exists = false; // prompt file not found in 00_nucleo/
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
