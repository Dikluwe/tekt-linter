//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/wildcard-saturation.md
//! @prompt-hash 361ede53
//! @layer L1
//! @updated 2026-08-14

use std::borrow::Cow;

use crate::entities::layer::Language;
use crate::entities::rule_traits::{HasDecisionArms, ScrutineeForm};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V20 — DeepPatternNesting (ADR-0016).
///
/// Regra-métrica (nível Info): reporta aninhamento profundo de padrões (`pattern_depth > 2`)
/// fora de contextos regulares de tabela (e.g. tuplas comparativas).
pub fn check<'a, T: HasDecisionArms<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();

    for expr in file.decision_exprs() {
        // Tabela regular (tuplas ou braços homogéneos) é isenta de aninhamento
        if expr.scrutinee_form == ScrutineeForm::Tuple || is_regular_table_context(expr) {
            continue;
        }

        for arm in &expr.arms {
            if arm.pattern_depth > 2 {
                violations.push(Violation {
                    rule_id: "V20".to_string(),
                    level: ViolationLevel::Info,
                    message: format!(
                        "Profundidade de padrão {} > 2 em `{}` fora de contexto-tabela",
                        arm.pattern_depth, arm.pattern_snippet
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


fn is_regular_table_context(expr: &crate::entities::rule_traits::DecisionExpr) -> bool {
    if expr.arms.len() < 3 {
        return false;
    }
    let tuple_pattern_count = expr
        .arms
        .iter()
        .filter(|a| a.pattern_snippet.starts_with('(') || a.is_catchall)
        .count();
    if tuple_pattern_count * 10 >= expr.arms.len() * 8 {
        return true;
    }
    false
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
        fn layer(&self) -> &Layer { &Layer::L1 }
        fn decision_exprs(&self) -> &[DecisionExpr<'static>] { &self.exprs }
        fn path(&self) -> &'static Path { self.path }
        fn language(&self) -> &Language { &self.language }
    }

    #[test]
    fn v20_reports_deep_pattern_nesting_as_info() {
        let arm = DecisionArm {
            pattern_snippet: "Some(Color::Rgb(r, g, b))",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 3,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 40,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "color_opt",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![arm],
            line: 39,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/color.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].rule_id, "V20");
        assert_eq!(viols[0].level, ViolationLevel::Info);
        assert!(viols[0].message.contains("Profundidade de padrão 3 > 2"));
    }

    #[test]
    fn v20_exempts_tuple_table_context() {
        let arm = DecisionArm {
            pattern_snippet: "(Some(a), Some(b))",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 3,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 40,
            column: 12,
        };
        let expr = DecisionExpr {
            snippet_scrutinee: "(a, b)",
            scrutinee_form: ScrutineeForm::Tuple,
            arms: vec![arm],
            line: 39,
            column: 8,
        };
        let file = MockFile {
            path: Path::new("01_core/compare.rs"),
            language: Language::Rust,
            exprs: vec![expr],
        };
        let viols = check(&file);
        assert!(viols.is_empty());
    }
}
