//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/test-file.md
//! @prompt-hash ce046580
//! @layer L1
//! @updated 2026-03-13

use crate::entities::layer::Layer;
use crate::entities::parsed_file::ParsedFile;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V2 — Missing test coverage for L1 modules.
/// Fires when layer == L1 AND has_test_coverage == false.
///
/// Exemption: files that only declare traits/structs/enums without impl
/// bodies are exempt. L3 (RustParser) encodes this exemption by setting
/// has_test_coverage = true for such files — L1 never re-derives it.
pub fn check(file: &ParsedFile) -> Vec<Violation> {
    if file.layer != Layer::L1 {
        return vec![];
    }

    if file.has_test_coverage {
        return vec![];
    }

    vec![Violation {
        rule_id: "V2".to_string(),
        level: ViolationLevel::Error,
        message: "Módulo do núcleo carece de verificação simultânea (test file ou bloco cfg(test))"
            .to_string(),
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
    use crate::entities::parsed_file::ParsedFile;
    use std::path::PathBuf;

    fn base_file(layer: Layer, has_test_coverage: bool) -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("01_core/foo.rs"),
            layer,
            language: Language::Rust,
            prompt_header: None,
            prompt_file_exists: false,
            has_test_coverage,
            imports: vec![],
            tokens: vec![],
        }
    }

    #[test]
    fn violation_when_l1_without_coverage() {
        let file = base_file(Layer::L1, false);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V2");
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }

    #[test]
    fn no_violation_when_l1_with_coverage() {
        let file = base_file(Layer::L1, true);
        assert!(check(&file).is_empty());
    }

    #[test]
    fn no_violation_for_non_l1_layers() {
        for layer in [Layer::L2, Layer::L3, Layer::L4, Layer::Lab, Layer::L0] {
            let file = base_file(layer, false);
            assert!(check(&file).is_empty(), "expected no V2 for layer {:?}", file.layer);
        }
    }

    #[test]
    fn exempt_file_has_coverage_set_by_l3() {
        // L3 sets has_test_coverage = true for trait-only files — L1 trusts it
        let file = base_file(Layer::L1, true);
        assert!(check(&file).is_empty());
    }
}
