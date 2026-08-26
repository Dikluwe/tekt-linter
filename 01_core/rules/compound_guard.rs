//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/compound-guard.md
//! @prompt-hash acc0543e
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;

use crate::entities::layer::Language;
use crate::entities::rule_traits::HasDecisionArms;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V17 — CompoundGuard (ADR-0016).
///
/// Detecta guards em braços de decisão contendo operadores booleanos compostos (`&&`, `||`).
/// Decisões com guard composto devem ser desdobradas ou movidas para o corpo.
pub fn check<'a, T: HasDecisionArms<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();

    for expr in file.decision_exprs() {
        for arm in &expr.arms {
            if arm.has_guard && arm.guard_is_compound {
                violations.push(Violation {
                    rule_id: "V17".to_string(),
                    level: ViolationLevel::Warning,
                    message: format!(
                        "Guard composto com operadores lógicos em braço de decisão: `{}` — simplifique ou desdobre a condição",
                        arm.pattern_snippet
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
    fn v17_detects_compound_guard() {
        let arm = DecisionArm {
            pattern_snippet: "Some(x)",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: true,
            guard_is_compound: true,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            mergeability: None,
            line: 15,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "val",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm],
            line: 14,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/eval.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V17");
        assert_eq!(viols[0].level, ViolationLevel::Warning);
    }
}
