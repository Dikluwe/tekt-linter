//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/wildcard-saturation.md
//! @prompt-hash a5de3b49
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;

use crate::entities::layer::Language;
use crate::entities::rule_traits::HasDecisionArms;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V18 — RangePatternInMatch (ADR-0016).
///
/// Detecta padrões de range numérico/caractere em expressões match de domínio.
/// Módulos com semântica intrínseca de parsing ou formatação numérica (`lexer`, `numbering`, `parser`)
/// são isentos.
pub fn check<'a, T: HasDecisionArms<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let path_str = file.path().to_string_lossy();
    if is_exempt_module(&path_str) {
        return vec![];
    }

    let mut violations = Vec::new();

    for expr in file.decision_exprs() {
        for arm in &expr.arms {
            if arm.pattern_is_range {
                violations.push(Violation {
                    rule_id: "V18".to_string(),
                    level: ViolationLevel::Warning,
                    message: format!(
                        "Padrão de range `{}` em match de domínio fora de módulo de lexing/numeração",
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

fn is_exempt_module(path: &str) -> bool {
    path.split(['/', '\\']).any(|component| {
        let stem = component
            .rsplit_once('.')
            .map_or(component, |(stem, _)| stem);
        matches!(stem, "lexer" | "numbering" | "syntax")
    })
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
    fn v18_detects_range_pattern_in_domain() {
        let arm = DecisionArm {
            pattern_snippet: "0..=9",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: true,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 20,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "code",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm],
            line: 19,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/domain.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V18");
        assert_eq!(viols[0].level, ViolationLevel::Warning);
    }

    #[test]
    fn v18_exempts_lexer_module() {
        let arm = DecisionArm {
            pattern_snippet: "'a'..='z'",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: true,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 20,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "ch",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm],
            line: 19,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/lexer.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert!(viols.is_empty());
    }
}
