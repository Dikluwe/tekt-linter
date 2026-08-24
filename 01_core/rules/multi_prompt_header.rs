//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/multi-prompt-header.md
//! @prompt-hash 868d3a92
//! @layer L1
//! @updated 2026-07-23

use std::borrow::Cow;

use crate::entities::layer::Layer;
use crate::entities::rule_traits::HasPromptRefs;
use crate::entities::violation::{Location, Violation, ViolationLevel};

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
        location: Location { path: Cow::Borrowed(file.path()), line: 1, column: 0 },
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
        MockFile { layer, refs, path: Path::new("01_core/foo.rs") }
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
        let file = file_with(
            Layer::L3,
            vec!["a.md", "b.md", "c.md"],
        );
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
            assert!(check(&file).is_empty(), "layer {layer:?} não devia disparar V15");
        }
    }
}
