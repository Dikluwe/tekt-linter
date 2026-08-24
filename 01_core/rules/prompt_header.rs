//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/prompt-header.md
//! @prompt-hash a94bb0e5
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::entities::layer::Layer;
use crate::entities::rule_traits::HasPromptFilesystem;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V1 — Missing or unresolvable @prompt header.
/// Fires when prompt_header is absent OR when the referenced prompt file
/// does not exist in 00_nucleo/ (prompt_file_exists == false).
pub fn check<'a, T: HasPromptFilesystem<'a>>(
    file: &T,
    strict_dirs: &[String],
) -> Vec<Violation<'a>> {
    if !matches!(file.layer(), Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4) {
        return vec![];
    }

    let is_strict = strict_dirs.iter().any(|d| file.path().starts_with(d));
    let level = if is_strict {
        ViolationLevel::Fatal
    } else {
        ViolationLevel::Error
    };

    let message = match file.prompt_header() {
        None => "Arquivo Cristalino sem linhagem causal @prompt encontrada".to_string(),
        Some(header) if !file.prompt_file_exists() => format!(
            "Arquivo Cristalino referencia prompt inexistente: '{}'",
            header.prompt_path
        ),
        Some(_) => return vec![],
    };

    vec![Violation {
        rule_id: "V1".to_string(),
        level,
        message,
        location: Location {
            path: Cow::Borrowed(file.path()),
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
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        header: Option<PromptHeader<'static>>,
        exists: bool,
        path: &'static Path,
    }

    impl HasPromptFilesystem<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &self.layer
        }
        fn prompt_header(&self) -> Option<&PromptHeader<'static>> {
            self.header.as_ref()
        }
        fn prompt_file_exists(&self) -> bool {
            self.exists
        }
        fn path(&self) -> &'static Path {
            self.path
        }
    }

    fn base_file() -> MockFile {
        MockFile {
            layer: Layer::L1,
            header: None,
            exists: false,
            path: Path::new("01_core/foo.rs"),
        }
    }

    fn valid_header() -> PromptHeader<'static> {
        PromptHeader {
            prompt_path: "00_nucleo/prompts/linter-core.md",
            prompt_hash: Some("a3f8c2d1"),
            current_hash: Some("a3f8c2d1".to_string()),
            layer: Layer::L1,
            updated: Some("2026-03-13"),
        }
    }

    #[test]
    fn no_violation_when_header_present_and_file_exists() {
        let mut file = base_file();
        file.header = Some(valid_header());
        file.exists = true;
        assert!(check(&file, &[]).is_empty());
    }

    #[test]
    fn violation_when_header_absent() {
        let file = base_file();
        let violations = check(&file, &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V1");
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }

    #[test]
    fn fatal_violation_when_header_absent_in_strict_dir() {
        let mut file = base_file();
        file.path = Path::new("02_shell/foo.rs");
        let strict_dirs = vec!["01_core".to_string(), "02_shell".to_string()];
        let violations = check(&file, &strict_dirs);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
    }

    #[test]
    fn violation_when_header_present_but_file_missing() {
        let mut file = base_file();
        file.header = Some(valid_header());
        file.exists = false;
        let violations = check(&file, &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V1");
        assert!(violations[0]
            .message
            .contains("00_nucleo/prompts/linter-core.md"));
        assert_ne!(
            violations[0].message,
            "Arquivo Cristalino sem linhagem causal @prompt encontrada"
        );
    }

    #[test]
    fn violation_points_to_line_1() {
        let file = base_file();
        let violations = check(&file, &[]);
        assert_eq!(violations[0].location.line, 1);
    }

    #[test]
    fn only_executable_layers_are_in_scope_for_all_header_states() {
        let layers = [
            Layer::L0,
            Layer::L1,
            Layer::L2,
            Layer::L3,
            Layer::L4,
            Layer::Lab,
            Layer::Unknown,
        ];
        for layer in layers {
            for (header, exists) in [
                (None, false),
                (Some(valid_header()), false),
                (Some(valid_header()), true),
            ] {
                let file = MockFile {
                    layer: layer.clone(),
                    header,
                    exists,
                    path: Path::new("some/file.rs"),
                };
                let applicable = matches!(layer, Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4);
                let expected = usize::from(applicable && (!exists || file.header.is_none()));
                assert_eq!(check(&file, &[]).len(), expected, "layer={layer:?}");
            }
        }
    }

    #[test]
    fn missing_prompt_path_is_preserved_literally() {
        let mut file = base_file();
        file.header = Some(PromptHeader {
            prompt_path: "00_nucleo/Prompts/naïve-é.md",
            ..valid_header()
        });
        let violations = check(&file, &[]);
        assert!(violations[0]
            .message
            .contains("00_nucleo/Prompts/naïve-é.md"));
    }
}
