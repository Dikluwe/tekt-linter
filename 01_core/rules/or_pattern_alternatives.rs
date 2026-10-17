//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/or-pattern-alternatives.md
//! @prompt-hash 28ecbcd3
//! @layer L1
//! @updated 2026-10-16

use std::borrow::Cow;

use crate::entities::layer::Language;
use crate::entities::rule_traits::HasDecisionArms;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// Explicação compartilhada entre a nota individual e o relatório consolidado.
pub const REVIEW_NOTE: &str = "Este braço agrupa alternativas. Executar o corpo não comprova exercitar cada alternativa; verificar os testes pertinentes.";

/// V19 — OrPatternAlternatives (ADR-0016).
///
/// Regra-métrica (nível Info): informa alternativas escritas nos grupos do padrão
/// e presença de guard, sem inferir cobertura de testes ou recomendar separar corpos.
pub fn check<'a, T: HasDecisionArms<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();

    for expr in file.decision_exprs() {
        for arm in &expr.arms {
            if arm.or_alternatives > 1 {
                let count = if arm.or_alternatives == u16::MAX {
                    format!("pelo menos {}", arm.or_alternatives)
                } else {
                    arm.or_alternatives.to_string()
                };
                let guard = if arm.has_guard { "presente" } else { "ausente" };
                violations.push(Violation {
                    rule_id: "V19".to_string(),
                    level: ViolationLevel::Info,
                    message: format!(
                        "{} Alternativas escritas nos grupos: {}; guard: {}; padrão: `{}`.",
                        REVIEW_NOTE, count, guard, arm.pattern_snippet
                    ),
                    location: Location {
                        path: Cow::Borrowed(file.path()),
                        line: arm.line,
                        column: arm.column,
                    },
                });
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::rule_traits::{BodyForm, DecisionArm, DecisionExpr, ScrutineeForm};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        language: Language,
        exprs: Vec<DecisionExpr<'static>>,
    }

    impl HasDecisionArms<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn decision_exprs(&self) -> &[DecisionExpr<'static>] {
            &self.exprs
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &self.language
        }
    }

    #[test]
    fn v19_reports_or_alternatives_as_info() {
        let arm = DecisionArm {
            pattern_snippet: "A | B | C",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 3,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            mergeability: None,
            line: 30,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "mode",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm],
            line: 29,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/mode.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V19");
        assert_eq!(viols[0].level, ViolationLevel::Info);
        assert_eq!(viols[0].message, "Este braço agrupa alternativas. Executar o corpo não comprova exercitar cada alternativa; verificar os testes pertinentes. Alternativas escritas nos grupos: 3; guard: ausente; padrão: `A | B | C`.");
    }
}
