//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/unsourced-constant.md
//! @prompt-hash aa2a1a88
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use crate::entities::layer::Language;
use crate::entities::rule_traits::HasConstants;
use crate::entities::violation::{Location, Violation, ViolationLevel};
use crate::rules::unsourced_constant::V21RuleConfig;

/// V22 — ProvenanceInventory (Passo 0066 / ADR-0016 Rev. 1).
///
/// Vigilância agregada por módulo: reporta o rácio `(literais com proveniência) / (total de literais)`
/// consolidado por módulo sem gerar ruído por ocorrência individual.
pub fn check_inventory<'a, T: HasConstants<'a>>(
    files: &[T],
    config: &V21RuleConfig,
) -> Vec<Violation<'a>> {
    // Agregação por módulo: module_name -> (cited, total, first_file_path)
    let mut module_stats: BTreeMap<String, (usize, usize, &'a Path)> = BTreeMap::new();

    for file in files {
        if *file.language() != Language::Rust {
            continue;
        }

        let path = file.path();
        let path_str = path.to_string_lossy();

        // Exclusão de módulos de sintaxe de formato
        if config
            .format_syntax_modules
            .iter()
            .any(|m| path_str.contains(m))
        {
            continue;
        }

        let module_name = extract_module_name(path);

        let mut file_total = 0;
        let mut file_cited = 0;

        for constant in file.constants() {
            if constant.is_test_origin || constant.is_in_data_table {
                continue;
            }
            if is_trivial_literal(constant.snippet, &config.trivial_literals) {
                continue;
            }

            file_total += 1;
            if constant.citation.is_some() {
                file_cited += 1;
            }
        }

        if file_total > 0 {
            let entry = module_stats.entry(module_name).or_insert((0, 0, path));
            entry.0 += file_cited;
            entry.1 += file_total;
            if path < entry.2 {
                entry.2 = path;
            }
        }
    }

    let mut violations = Vec::new();
    for (module_name, (cited, total, first_path)) in module_stats {
        let percentage = (cited as f64 / total as f64) * 100.0;
        violations.push(Violation {
            rule_id: "V22".to_string(),
            level: ViolationLevel::Info,
            message: format!(
                "Módulo '{}': inventário de proveniência = {}/{} ({:.1}%)",
                module_name, cited, total, percentage
            ),
            location: Location {
                path: Cow::Borrowed(first_path),
                line: 1,
                column: 0,
            },
        });
    }

    violations
}

/// Extrai o nome canônico do módulo a partir do caminho do arquivo.
/// Agrupa por subdiretório principal (ex: "01_core/rules", "03_infra", "layout", "export/pdf").
fn extract_module_name(path: &Path) -> String {
    let comps: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .filter(|c| c != "." && c != "..")
        .collect();

    if comps.is_empty() {
        return "root".to_string();
    }

    // Se começa com L0-L4 (ex: 01_core/rules/foo.rs -> "01_core/rules")
    if comps[0].starts_with("01_")
        || comps[0].starts_with("02_")
        || comps[0].starts_with("03_")
        || comps[0].starts_with("04_")
    {
        if comps.len() >= 3
            && (comps[1] == "entities" || comps[1] == "contracts" || comps[1] == "rules")
        {
            return format!("{}/{}", comps[0], comps[1]);
        }
        return comps[0].clone();
    }

    // Se é estilo workspace (ex: typst-layout/src/flow.rs -> "typst-layout")
    if comps.len() >= 2 && comps[1] == "src" {
        return comps[0].clone();
    }

    // Fallback: diretório pai
    if let Some(parent) = path.parent() {
        let p_str = parent.to_string_lossy().replace('\\', "/");
        if !p_str.is_empty() && p_str != "." {
            return p_str;
        }
    }

    comps[0].clone()
}

fn is_trivial_literal(snippet: &str, trivial_set: &HashSet<String>) -> bool {
    let trimmed = snippet.trim();
    if trivial_set.contains(trimmed) {
        return true;
    }
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        if inner.is_empty()
            || inner.chars().count() == 1
            || inner == r"\n"
            || inner == r"\t"
            || inner == r"\r"
            || inner == r#"\""#
        {
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

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }

    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    #[test]
    fn provenance_inventory_aggregates_multiple_files_per_module() {
        let files = vec![
            MockFile {
                path: Path::new("01_core/rules/a.rs"),
                constants: vec![SourceConstant {
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
                }],
            },
            MockFile {
                path: Path::new("01_core/rules/b.rs"),
                constants: vec![SourceConstant {
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
                }],
            },
        ];

        let viols = check_inventory(&files, &V21RuleConfig::default());
        assert_eq!(
            viols.len(),
            1,
            "Agrega os 2 arquivos em 1 entrada para o módulo 01_core/rules"
        );
        assert_eq!(viols[0].rule_id, "V22");
        assert_eq!(viols[0].level, ViolationLevel::Info);
        assert!(viols[0].message.contains("01_core/rules"));
        assert!(viols[0].message.contains("1/2"));
        assert!(viols[0].message.contains("50.0%"));
    }

    #[test]
    fn inventory_location_is_minimum_path_under_permutation() {
        let constant = || SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "10.5",
            line: 1,
            column: 0,
            citation: None,
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: false,
            context_var: None,
            geometric_sink: None,
            is_in_data_table: false,
        };
        let files = vec![
            MockFile {
                path: Path::new("01_core/rules/z.rs"),
                constants: vec![constant()],
            },
            MockFile {
                path: Path::new("01_core/rules/a.rs"),
                constants: vec![constant()],
            },
        ];
        let forward = check_inventory(&files, &V21RuleConfig::default());
        let reverse = check_inventory(
            &files.into_iter().rev().collect::<Vec<_>>(),
            &V21RuleConfig::default(),
        );
        assert_eq!(
            forward[0].location.path.as_ref(),
            Path::new("01_core/rules/a.rs")
        );
        assert_eq!(forward[0].location.path, reverse[0].location.path);
    }
}
