//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-drift.md
//! @prompt-hash 9e9f8641
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::contracts::rule_traits::HasHashes;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V5 — Prompt drift detection.
/// Fires when prompt_hash (declared in header) diverges from
/// current_hash (real hash of the L0 prompt file, populated by L3).
/// Warning level — does not block CI by default.
pub fn check<'a, T: HasHashes<'a>>(file: &T) -> Vec<Violation<'a>> {
    let header = match file.prompt_header() {
        Some(h) => h,
        None => return vec![], // V1 already handles absent headers
    };

    let declared = match header.prompt_hash {
        Some(h) => h,
        None => return vec![], // no hash declared — not a drift violation
    };

    let current = match &header.current_hash {
        Some(h) => h,
        None => return vec![], // prompt file missing — V1 handles this
    };

    if declared == current.as_str() {
        return vec![];
    }

    vec![Violation {
        rule_id: "V5".to_string(),
        level: ViolationLevel::Warning,
        message: format!(
            "Deriva detectada (Drift): o arquivo @prompt original foi modificado sem atualização \
             condizente da implementação. Hash L0: {}, Código: {}",
            current, declared
        ),
        location: Location { path: Cow::Borrowed(file.path()), line: 1, column: 0 },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::PromptHeader;
    use std::path::Path;

    struct MockFile {
        header: Option<PromptHeader<'static>>,
        path: &'static Path,
    }

    impl HasHashes<'static> for MockFile {
        fn prompt_header(&self) -> Option<&PromptHeader<'static>> {
            self.header.as_ref()
        }
        fn path(&self) -> &'static Path {
            self.path
        }
    }

    fn file_with_hashes(declared: Option<&'static str>, current: Option<&str>) -> MockFile {
        MockFile {
            header: Some(PromptHeader {
                prompt_path: "00_nucleo/prompts/linter-core.md",
                prompt_hash: declared,
                current_hash: current.map(str::to_string),
                layer: Layer::L1,
                updated: None,
            }),
            path: Path::new("01_core/foo.rs"),
        }
    }

    #[test]
    fn violation_when_hashes_diverge() {
        let file = file_with_hashes(Some("a3f8c2d1"), Some("b9e4f7a2"));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V5");
        assert_eq!(violations[0].level, ViolationLevel::Warning);
    }

    #[test]
    fn no_violation_when_hashes_match() {
        let file = file_with_hashes(Some("a3f8c2d1"), Some("a3f8c2d1"));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn no_violation_when_prompt_header_absent() {
        let file = MockFile {
            header: None,
            path: Path::new("01_core/foo.rs"),
        };
        assert!(check(&file).is_empty());
    }

    #[test]
    fn no_violation_when_declared_hash_absent() {
        let file = file_with_hashes(None, Some("b9e4f7a2"));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn no_violation_when_current_hash_absent() {
        let file = file_with_hashes(Some("a3f8c2d1"), None);
        assert!(check(&file).is_empty());
    }

    #[test]
    fn violation_message_contains_both_hashes() {
        let file = file_with_hashes(Some("a3f8c2d1"), Some("b9e4f7a2"));
        let violations = check(&file);
        assert!(violations[0].message.contains("b9e4f7a2")); // current (L0)
        assert!(violations[0].message.contains("a3f8c2d1")); // declared (code)
    }
}
