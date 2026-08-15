//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/unsourced-constant.md
//! @prompt-hash 0b319e67
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;
use std::collections::HashSet;

use crate::entities::layer::Language;
use crate::entities::rule_traits::HasConstants;
use crate::entities::violation::{Location, Violation, ViolationLevel};
use crate::rules::unsourced_constant::V21RuleConfig;

/// V22 — ProvenanceInventory (Passo 0066 / ADR-0016 Rev. 1).
///
/// Vigilância agregada por módulo: reporta o rácio `(literais com proveniência) / (total de literais)`
/// por módulo sem gerar ruído por ocorrência individual.
pub fn check<'a, T: HasConstants<'a>>(
    file: &T,
    config: &V21RuleConfig,
) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let path_str = file.path().to_string_lossy();

    // Exclusão de módulos de sintaxe de formato
    if config
        .format_syntax_modules
        .iter()
        .any(|m| path_str.contains(m))
    {
        return vec![];
    }

    let mut total_literals = 0;
    let mut cited_literals = 0;

    for constant in file.constants() {
        if constant.is_test_origin || constant.is_in_data_table {
            continue;
        }
        if is_trivial_literal(constant.snippet, &config.trivial_literals) {
            continue;
        }

        total_literals += 1;
        if constant.citation.is_some() {
            cited_literals += 1;
        }
    }

    if total_literals == 0 {
        return vec![];
    }

    let percentage = (cited_literals as f64 / total_literals as f64) * 100.0;
    vec![Violation {
        rule_id: "V22".to_string(),
        level: ViolationLevel::Info,
        message: format!(
            "Módulo '{}': inventário de proveniência = {}/{} ({:.1}%)",
            path_str, cited_literals, total_literals, percentage
        ),
        location: Location {
            path: Cow::Borrowed(file.path()),
            line: 1,
            column: 0,
        },
    }]
}

fn is_trivial_literal(snippet: &str, trivial_set: &HashSet<String>) -> bool {
    let trimmed = snippet.trim();
    if trivial_set.contains(trimmed) {
        return true;
    }
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        if inner.is_empty() || inner.chars().count() == 1 || inner == r"\n" || inner == r"\t" || inner == r"\r" || inner == r#"\""# {
            return true;
        }
    }
    false
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::rule_traits::{Citation, CitationKind, ConstantKind, SourceConstant};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }

    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer { &Layer::L1 }
        fn constants(&self) -> &[SourceConstant<'static>] { &self.constants }
        fn path(&self) -> &'static Path { self.path }
        fn language(&self) -> &Language { &Language::Rust }
    }

    #[test]
    fn provenance_inventory_aggregates_correct_ratio() {
        let file = MockFile {
            path: Path::new("layout/src/flow.rs"),
            constants: vec![
                SourceConstant {
                    kind: ConstantKind::FunctionNumberLiteral,
                    snippet: "10.5",
                    line: 10,
                    column: 0,
                    citation: Some(Citation {
                        kind: CitationKind::Rationale("teste"),
                        raw: "// rationale: teste",
                        line: 9,
                    }),
                    is_test_origin: false,
                    function_return_type: None,
                    is_in_binary_scaling: false,
                    context_var: None,
                    geometric_sink: None,
                    is_in_data_table: false,
                },
                SourceConstant {
                    kind: ConstantKind::FunctionNumberLiteral,
                    snippet: "20.5",
                    line: 12,
                    column: 0,
                    citation: None,
                    is_test_origin: false,
                    function_return_type: None,
                    is_in_binary_scaling: false,
                    context_var: None,
                    geometric_sink: None,
                    is_in_data_table: false,
                },
            ],
        };

        let viols = check(&file, &V21RuleConfig::default());
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V22");
        assert_eq!(viols[0].level, ViolationLevel::Info);
        assert!(viols[0].message.contains("1/2"));
        assert!(viols[0].message.contains("50.0%"));
    }
}
