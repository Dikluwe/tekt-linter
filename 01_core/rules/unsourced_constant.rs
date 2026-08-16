//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/unsourced-constant.md
//! @prompt-hash 0b319e67
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::entities::layer::Language;
use crate::entities::rule_traits::{CitationKind, HasConstants};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// Configuração de escopo, fontes contextuais e sumidouros para V21.
#[derive(Debug, Clone)]
pub struct V21RuleConfig {
    pub context_vars: Vec<String>,
    pub geometric_sinks: Vec<String>,
    pub format_syntax_modules: Vec<String>,
    pub scope_modules: Vec<String>,
    pub scope_types: Vec<String>,
    pub trivial_literals: HashSet<String>,
    pub strict_modules: Vec<String>,
}

impl Default for V21RuleConfig {
    fn default() -> Self {
        let context_vars = vec![
            "size".to_string(),
            "style".to_string(),
            "em".to_string(),
            "font".to_string(),
            "weight".to_string(),
            "ascent".to_string(),
            "descent".to_string(),
            "width".to_string(),
            "height".to_string(),
            "depth".to_string(),
            "frame".to_string(),
            "region".to_string(),
            "page".to_string(),
            "margin".to_string(),
            "padding".to_string(),
            "container".to_string(),
        ];
        let geometric_sinks = vec![
            "cursor_y".to_string(),
            "cursor_x".to_string(),
            "cursor".to_string(),
            "gap".to_string(),
            "inset".to_string(),
            "offset".to_string(),
            "pos".to_string(),
            "x".to_string(),
            "y".to_string(),
            "width".to_string(),
            "height".to_string(),
            "thickness".to_string(),
            "ascent".to_string(),
            "descent".to_string(),
            "length".to_string(),
            "pt".to_string(),
            "em".to_string(),
            "ratio".to_string(),
            "abs".to_string(),
            "point".to_string(),
            "size".to_string(),
            "frame".to_string(),
        ];
        let format_syntax_modules = vec![
            "export/pdf".to_string(),
            "export/svg".to_string(),
        ];
        let scope_modules = vec![
            "layout/".to_string(),
            "export/".to_string(),
            "math/".to_string(),
            "shaper".to_string(),
            "geom".to_string(),
        ];
        let scope_types = vec![
            "Frame".to_string(),
            "FrameItem".to_string(),
            "Length".to_string(),
            "Point".to_string(),
            "Size".to_string(),
            "Transform".to_string(),
            "Color".to_string(),
            "Paint".to_string(),
        ];
        let mut trivial_literals = HashSet::new();
        for lit in &["0", "1", "-1", "2", "100", "0.0", "1.0", r#"""#] {
            trivial_literals.insert(lit.to_string());
        }
        Self {
            context_vars,
            geometric_sinks,
            format_syntax_modules,
            scope_modules,
            scope_types,
            trivial_literals,
            strict_modules: Vec::new(),
        }
    }
}

/// V21 — HardcodedContextualValue (Passo 0066 / ADR-0016 Rev. 1).
///
/// Caça escalares contextuais tratados como fixos: literal numérico operando de `*`/`/`
/// com uma variável de fonte contextual cujo resultado alimenta um sumidouro geométrico.
pub fn check<'a, T: HasConstants<'a>>(
    file: &T,
    config: &V21RuleConfig,
    project_root: Option<&Path>,
) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();
    let path_str = file.path().to_string_lossy();

    // 1. Exclusão de módulos de sintaxe fixa de formato (PDF operators, SVG tags, etc.)
    if config
        .format_syntax_modules
        .iter()
        .any(|m| path_str.contains(m))
    {
        return vec![];
    }

    let is_strict = config
        .strict_modules
        .iter()
        .any(|s| path_str.contains(s) || path_str.starts_with(s));

    for constant in file.constants() {
        // Exclusões: testes/fixtures e tabelas de tradução
        if constant.is_test_origin || constant.is_in_data_table {
            continue;
        }

        // Allowlist de triviais
        if is_trivial_literal(constant.snippet, &config.trivial_literals) {
            continue;
        }

        // Predicado estrito de V21:
        // (a) Deve ser operação de multiplicação ou divisão com variável
        if !constant.is_in_binary_scaling {
            continue;
        }

        // (b) A variável parceira deve ser uma fonte contextual
        let var_match = match &constant.context_var {
            Some(v) => {
                let v_lower = v.to_lowercase();
                config.context_vars.iter().any(|c| v_lower.contains(c))
            }
            None => false,
        };
        if !var_match {
            continue;
        }

        // (c) O resultado deve alimentar um sumidouro geométrico
        let sink_match = match &constant.geometric_sink {
            Some(s) => {
                let s_lower = s.to_lowercase();
                config.geometric_sinks.iter().any(|g| s_lower.contains(g))
            }
            None => false,
        };
        if !sink_match {
            continue;
        }

        // Checagem de proveniência / citação
        match &constant.citation {
            None => {
                let level = if is_strict {
                    ViolationLevel::Error
                } else {
                    ViolationLevel::Warning
                };
                let var_name = constant.context_var.as_deref().unwrap_or("var");
                let sink_name = constant.geometric_sink.as_deref().unwrap_or("sink");
                violations.push(Violation {
                    rule_id: "V21".to_string(),
                    level,
                    message: format!(
                        "Escalar contextual fixo: literal `{}` escala `{}` para `{}` sem proveniência citada (adicione `// ref:`, `// spec:` ou `// rationale:`)",
                        constant.snippet, var_name, sink_name
                    ),
                    location: Location {
                        path: Cow::Borrowed(file.path()),
                        line: constant.line,
                        column: constant.column,
                    },
                });
            }
            Some(citation) => {
                if let CitationKind::Ref { path: ref_path, line: ref_line } = citation.kind {
                    if is_ref_citation_stale(ref_path, ref_line, project_root) {
                        violations.push(Violation {
                            rule_id: "V21".to_string(),
                            level: ViolationLevel::Warning,
                            message: format!(
                                "Citação obsoleta: 'ref: {}:{}' aponta para arquivo ou linha inexistente/vazia",
                                ref_path, ref_line
                            ),
                            location: Location {
                                path: Cow::Borrowed(file.path()),
                                line: constant.line,
                                column: constant.column,
                            },
                        });
                    }
                }
            }
        }
    }

    violations
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

fn is_ref_citation_stale(ref_path: &str, ref_line: usize, project_root: Option<&Path>) -> bool {
    if ref_line == 0 {
        return true;
    }
    let full_path = match project_root {
        Some(root) => root.join(ref_path),
        None => Path::new(ref_path).to_path_buf(),
    };
    if !full_path.exists() || !full_path.is_file() {
        return true;
    }
    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(_) => return true,
    };
    let lines: Vec<&str> = content.lines().collect();
    if ref_line > lines.len() {
        return true;
    }
    lines[ref_line - 1].trim().is_empty()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::rule_traits::{ConstantKind, SourceConstant};

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
    fn compound_assignment_deep_field_triggers_v21() {
        let file = MockFile {
            path: Path::new("layout/src/divider.rs"),
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.6",
                line: 42,
                column: 12,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: true,
                context_var: Some("layouter.style.size".to_string()),
                geometric_sink: Some("layouter.regions.current.cursor_y".to_string()),
                is_in_data_table: false,
            }],
        };

        let viols = check(&file, &V21RuleConfig::default(), None);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V21");
        assert!(viols[0].message.contains("0.6"));
        assert!(viols[0].message.contains("layouter.style.size"));
        assert!(viols[0].message.contains("layouter.regions.current.cursor_y"));
    }

    #[test]
    fn hardcoded_scaling_in_geometric_sink_triggers_warning() {
        let file = MockFile {
            path: Path::new("layout/src/raw.rs"),
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.9",
                line: 15,
                column: 8,
                citation: None,
                is_test_origin: false,
                function_return_type: Some("Length"),
                is_in_binary_scaling: true,
                context_var: Some("size".to_string()),
                geometric_sink: Some("gap".to_string()),
                is_in_data_table: false,
            }],
        };

        let viols = check(&file, &V21RuleConfig::default(), None);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V21");
        assert_eq!(viols[0].level, ViolationLevel::Warning);
        assert!(viols[0].message.contains("0.9"));
        assert!(viols[0].message.contains("size"));
        assert!(viols[0].message.contains("gap"));
    }

    #[test]
    fn isolated_literal_outside_scaling_does_not_trigger() {
        let file = MockFile {
            path: Path::new("layout/src/raw.rs"),
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "12.5",
                line: 20,
                column: 8,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: Some("gap".to_string()),
                is_in_data_table: false,
            }],
        };

        let viols = check(&file, &V21RuleConfig::default(), None);
        assert!(viols.is_empty(), "literal isolado fora de multiplicacao contextual nao dispara V21");
    }

    #[test]
    fn data_table_is_exempt() {
        let file = MockFile {
            path: Path::new("layout/src/table.rs"),
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.9",
                line: 30,
                column: 8,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: true,
                context_var: Some("size".to_string()),
                geometric_sink: Some("gap".to_string()),
                is_in_data_table: true,
            }],
        };

        let viols = check(&file, &V21RuleConfig::default(), None);
        assert!(viols.is_empty(), "tabela de dados e isenta de V21");
    }

    #[test]
    fn format_syntax_module_is_exempt() {
        let file = MockFile {
            path: Path::new("export/pdf/stream.rs"),
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.9",
                line: 10,
                column: 8,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: true,
                context_var: Some("width".to_string()),
                geometric_sink: Some("pos".to_string()),
                is_in_data_table: false,
            }],
        };

        let viols = check(&file, &V21RuleConfig::default(), None);
        assert!(viols.is_empty(), "modulo de sintaxe de formato e isento de V21");
    }
}
