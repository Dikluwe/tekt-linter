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
use crate::entities::rule_traits::{source_term_for, CitationKind, HasConstants};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// Configuração de escopo e filtros para a regra V21.
#[derive(Debug, Clone)]
pub struct V21RuleConfig {
    pub scope_modules: Vec<String>,
    pub scope_types: Vec<String>,
    pub trivial_literals: HashSet<String>,
    pub strict_modules: Vec<String>,
}

impl Default for V21RuleConfig {
    fn default() -> Self {
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
        for lit in &["0", "1", "-1", "2", "100", "0.0", "1.0", "\"\"" ] {
            trivial_literals.insert(lit.to_string());
        }
        Self {
            scope_modules,
            scope_types,
            trivial_literals,
            strict_modules: Vec::new(),
        }
    }
}

/// V21 — UnsourcedConstant (ADR-0016 Rev. 1).
///
/// Verifica se constantes e literais em módulos geométricos/exportação possuem
/// proveniência citada (`// ref:`, `// spec:`, `// rationale:`).
pub fn check<'a, T: HasConstants<'a>>(
    file: &T,
    config: &V21RuleConfig,
    project_root: Option<&Path>,
) -> Vec<Violation<'a>> {
    // Escopo inicial: Rust
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();
    let path_str = file.path().to_string_lossy();

    // 1. Verificar se o próprio path do arquivo está no escopo de módulos
    let file_in_module_scope = config
        .scope_modules
        .iter()
        .any(|m| path_str.contains(m));

    let is_strict = config
        .strict_modules
        .iter()
        .any(|s| path_str.contains(s) || path_str.starts_with(s));

    for constant in file.constants() {
        // Funções e módulos de teste são sempre isentos
        if constant.is_test_origin {
            continue;
        }

        // 2. Verificar escopo (duas camadas: path do módulo OU tipo de retorno da função)
        let in_function_type_scope = match constant.function_return_type {
            Some(ret) => config.scope_types.iter().any(|t| ret.contains(t)),
            None => false,
        };

        if !file_in_module_scope && !in_function_type_scope {
            continue;
        }

        // 3. Allowlist de triviais (anti-ruído obrigatória)
        if is_trivial_literal(constant.snippet, &config.trivial_literals) {
            continue;
        }

        // 4. Checagem de proveniência / citação
        match &constant.citation {
            None => {
                let term = source_term_for(file.language(), &constant.kind);
                let level = if is_strict {
                    ViolationLevel::Warning
                } else {
                    ViolationLevel::Info
                };
                violations.push(Violation {
                    rule_id: "V21".to_string(),
                    level,
                    message: format!(
                        "{} `{}` carece de proveniência citada (adicione `// ref:`, `// spec:` ou `// rationale:`)",
                        term, constant.snippet
                    ),
                    location: Location {
                        path: Cow::Borrowed(file.path()),
                        line: constant.line,
                        column: constant.column,
                    },
                });
            }
            Some(citation) => {
                // Anti-apodrecimento para `// ref: <caminho>:<linha>`
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

/// Heurística de identificação de literais triviais anti-ruído.
fn is_trivial_literal(snippet: &str, trivial_set: &HashSet<String>) -> bool {
    let trimmed = snippet.trim();
    if trivial_set.contains(trimmed) {
        return true;
    }

    // String vazia ou de 1 caractere (ex: "", ",", "\n", " ", "x")
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        // Comprimento decodificado de 0 ou 1 caractere (incluindo escapes como \n, \t)
        if inner.is_empty() || inner.chars().count() == 1 || inner == "\n" || inner == "\t" || inner == "\r" || inner == "\"" {
            return true;
        }
    }

    false
}

/// Verifica se uma referência `ref: <caminho>:<linha>` aponta para linha não-vazia existente.
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

    let target_line = lines[ref_line - 1].trim();
    target_line.is_empty()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::rule_traits::{Citation, CitationKind, ConstantKind, SourceConstant};
    use std::io::Write;
    use tempfile::NamedTempFile;

    struct MockFile {
        path: &'static Path,
        language: Language,
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
            &self.language
        }
    }

    #[test]
    fn non_rust_files_return_empty() {
        let file = MockFile {
            path: Path::new("export/svg.ts"),
            language: Language::TypeScript,
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "42.0",
                line: 10,
                column: 4,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert!(violations.is_empty());
    }

    #[test]
    fn file_outside_scope_and_type_is_exempt() {
        let file = MockFile {
            path: Path::new("01_core/entities/layer.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "42.0",
                line: 10,
                column: 4,
                citation: None,
                is_test_origin: false,
                function_return_type: Some("usize"),
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert!(violations.is_empty());
    }

    #[test]
    fn const_in_module_scope_without_citation_triggers_info() {
        let file = MockFile {
            path: Path::new("typst-layout/src/frame.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::ItemDefinition,
                snippet: "const MIN_LEADING: Length = 12.5",
                line: 15,
                column: 0,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V21");
        assert_eq!(violations[0].level, ViolationLevel::Info);
        assert!(violations[0].message.contains("MIN_LEADING"));
    }

    #[test]
    fn const_with_valid_rationale_is_clean() {
        let file = MockFile {
            path: Path::new("export/pdf.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::ItemDefinition,
                snippet: "const DPI: f64 = 72.0",
                line: 12,
                column: 0,
                citation: Some(Citation {
                    kind: CitationKind::Rationale("padrão PDF 72 pontos por polegada"),
                    raw: "// rationale: padrão PDF 72 pontos por polegada",
                    line: 11,
                }),
                is_test_origin: false,
                function_return_type: None,
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert!(violations.is_empty());
    }

    #[test]
    fn stale_ref_citation_triggers_warning() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(temp, "").unwrap(); // Linha 1 vazia
        let temp_path = temp.path().to_str().unwrap().to_string();

        let file = MockFile {
            path: Path::new("geom/rect.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "14.4",
                line: 20,
                column: 8,
                citation: Some(Citation {
                    kind: CitationKind::Ref {
                        path: Box::leak(temp_path.into_boxed_str()),
                        line: 1,
                    },
                    raw: "// ref: ...",
                    line: 19,
                }),
                is_test_origin: false,
                function_return_type: None,
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].level, ViolationLevel::Warning);
        assert!(violations[0].message.contains("Citação obsoleta"));
    }

    #[test]
    fn trivial_literals_do_not_trigger() {
        let file = MockFile {
            path: Path::new("layout/box.rs"),
            language: Language::Rust,
            constants: vec![
                SourceConstant {
                    kind: ConstantKind::FunctionNumberLiteral,
                    snippet: "0",
                    line: 1,
                    column: 0,
                    citation: None,
                    is_test_origin: false,
                    function_return_type: None,
                },
                SourceConstant {
                    kind: ConstantKind::FunctionNumberLiteral,
                    snippet: "1.0",
                    line: 2,
                    column: 0,
                    citation: None,
                    is_test_origin: false,
                    function_return_type: None,
                },
                SourceConstant {
                    kind: ConstantKind::FunctionStringLiteral,
                    snippet: "\"\"",
                    line: 3,
                    column: 0,
                    citation: None,
                    is_test_origin: false,
                    function_return_type: None,
                },
            ],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_origin_is_exempt() {
        let file = MockFile {
            path: Path::new("layout/box.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "999.99",
                line: 50,
                column: 8,
                citation: None,
                is_test_origin: true,
                function_return_type: None,
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert!(violations.is_empty());
    }

    #[test]
    fn function_returning_scope_type_is_analyzed() {
        let file = MockFile {
            path: Path::new("src/builder.rs"),
            language: Language::Rust,
            constants: vec![SourceConstant {
                kind: ConstantKind::NegativeLiteral,
                snippet: "-0.5",
                line: 30,
                column: 12,
                citation: None,
                is_test_origin: false,
                function_return_type: Some("Result<Point, Error>"),
            }],
        };
        let violations = check(&file, &V21RuleConfig::default(), None);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V21");
    }
}
