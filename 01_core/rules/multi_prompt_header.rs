//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/multi-prompt-header.md
//! @prompt-hash 868d3a92
//! @layer L1
//! @updated 2026-07-23

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::entities::layer::Layer;
use crate::entities::rule_traits::HasPromptRefs;
use crate::entities::violation::{Location, Violation, ViolationLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptOwnershipLayer {
    L0,
    L1,
    L2,
    L3,
    L4,
    Lab,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptOwnership {
    pub code_path: PathBuf,
    pub layer: PromptOwnershipLayer,
    pub prompt_refs: Vec<String>,
}

fn is_productive(layer: PromptOwnershipLayer) -> bool {
    matches!(
        layer,
        PromptOwnershipLayer::L1
            | PromptOwnershipLayer::L2
            | PromptOwnershipLayer::L3
            | PromptOwnershipLayer::L4
    )
}

/// V15 integral: preserva a regra local e rejeita qualquer prompt proprietário
/// consumido por mais de um código produtivo.
pub fn check_prompt_ownership(entries: &[PromptOwnership]) -> Vec<Violation<'static>> {
    let mut violations = Vec::new();
    let mut consumers: BTreeMap<&str, BTreeSet<&PathBuf>> = BTreeMap::new();

    for entry in entries.iter().filter(|entry| is_productive(entry.layer)) {
        if entry.prompt_refs.len() >= 2 {
            violations.push(Violation {
                rule_id: "V15".into(),
                level: ViolationLevel::Error,
                message: format!(
                    "Arquivo com {} headers @prompt ({}). Regra biunívoca: um código, um prompt proprietário.",
                    entry.prompt_refs.len(),
                    entry.prompt_refs.join(", ")
                ),
                location: Location {
                    path: Cow::Owned(entry.code_path.clone()),
                    line: 1,
                    column: 0,
                },
            });
        }
        for prompt in &entry.prompt_refs {
            consumers
                .entry(prompt)
                .or_default()
                .insert(&entry.code_path);
        }
    }

    for (prompt, paths) in consumers.into_iter().filter(|(_, paths)| paths.len() >= 2) {
        let listed = paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join(", ");
        violations.push(Violation {
            rule_id: "V15".into(),
            level: ViolationLevel::Error,
            message: format!(
                "Prompt proprietário {prompt} possui {} consumers: {listed}. Regra biunívoca: um prompt, um código.",
                paths.len()
            ),
            location: Location {
                path: Cow::Owned((**paths.first().expect("non-empty collision")).clone()),
                line: 1,
                column: 0,
            },
        });
    }

    violations.sort_by(|left, right| {
        left.message
            .as_bytes()
            .cmp(right.message.as_bytes())
            .then_with(|| {
                left.location
                    .path
                    .as_os_str()
                    .cmp(right.location.path.as_os_str())
            })
    });
    violations
}

/// V15 — Multiple @prompt headers in one file.
/// Regra de linhagem: um ficheiro, um prompt. Com 2+ linhas `@prompt` no
/// bloco de doc-header, `extract_header` fica com o último valor e
/// `--fix-hashes` é indefinido (hash certo no header errado). V15 bloqueia
/// esse estado com Error em vez de silêncio ou correcção ambígua.
pub fn check<'a, T: HasPromptRefs<'a>>(file: &T) -> Vec<Violation<'a>> {
    let refs = file.prompt_refs();
    if refs.len() < 2 || !matches!(file.layer(), Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4) {
        return vec![];
    }

    vec![Violation {
        rule_id: "V15".to_string(),
        level: ViolationLevel::Error,
        message: format!(
            "Arquivo com {} headers @prompt ({}). \
             Regra: um ficheiro, um prompt — dividir o ficheiro ou remover as \
             linhagens extra. --fix-hashes é indefinido com multi-@prompt.",
            refs.len(),
            refs.join(", "),
        ),
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
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        refs: Vec<&'static str>,
        path: &'static Path,
    }

    impl HasPromptRefs<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &self.layer
        }
        fn prompt_refs(&self) -> &[&'static str] {
            &self.refs
        }
        fn path(&self) -> &'static Path {
            self.path
        }
    }

    fn file_with(layer: Layer, refs: Vec<&'static str>) -> MockFile {
        MockFile {
            layer,
            refs,
            path: Path::new("01_core/foo.rs"),
        }
    }

    #[test]
    fn two_prompt_headers_in_l1_is_error() {
        let file = file_with(
            Layer::L1,
            vec!["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"],
        );
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V15");
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }

    #[test]
    fn message_lists_all_prompts_and_states_the_rule() {
        let file = file_with(
            Layer::L1,
            vec!["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"],
        );
        let violations = check(&file);
        assert!(violations[0].message.contains("00_nucleo/prompts/a.md"));
        assert!(violations[0].message.contains("00_nucleo/prompts/b.md"));
        assert!(violations[0].message.contains("um ficheiro, um prompt"));
    }

    #[test]
    fn violation_points_to_line_1() {
        let file = file_with(
            Layer::L2,
            vec!["00_nucleo/prompts/a.md", "00_nucleo/prompts/a.md"],
        );
        let violations = check(&file);
        assert_eq!(violations[0].location.line, 1);
    }

    #[test]
    fn three_prompt_headers_produce_single_violation() {
        let file = file_with(Layer::L3, vec!["a.md", "b.md", "c.md"]);
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains('3'));
    }

    #[test]
    fn single_prompt_header_passes() {
        let file = file_with(Layer::L1, vec!["00_nucleo/prompts/a.md"]);
        assert!(check(&file).is_empty());
    }

    #[test]
    fn no_prompt_header_passes() {
        // Ausência de header é território de V1, não de V15.
        let file = file_with(Layer::L1, vec![]);
        assert!(check(&file).is_empty());
    }

    #[test]
    fn multi_prompt_in_l4_is_flagged() {
        let file = file_with(Layer::L4, vec!["a.md", "b.md"]);
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn multi_prompt_outside_l1_l4_passes() {
        // V15 aplica-se apenas a L1–L4.
        for layer in [Layer::Lab, Layer::Unknown, Layer::L0] {
            let file = file_with(layer.clone(), vec!["a.md", "b.md"]);
            assert!(
                check(&file).is_empty(),
                "layer {layer:?} não devia disparar V15"
            );
        }
    }
}
