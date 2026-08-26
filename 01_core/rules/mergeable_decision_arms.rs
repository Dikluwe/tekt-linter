//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/mergeable-decision-arms.md
//! @prompt-hash fd318ed9
//! @layer L1
//! @updated 2026-08-26

use std::borrow::Cow;

use crate::entities::layer::Language;
use crate::entities::rule_traits::{BodyForm, DecisionArm, HasDecisionArms};
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V27 — MergeableDecisionArms.
///
/// Detecta grupos maximais de braços adjacentes cuja consolidação sintática pode ser
/// provada pela IR. `None` em `mergeability` é Unknown e nunca vira achado.
pub fn check<'a, T: HasDecisionArms<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.language() != Language::Rust {
        return vec![];
    }

    let mut violations = Vec::new();
    for expr in file.decision_exprs() {
        let mut first = 0;
        while first + 1 < expr.arms.len() {
            let mut end = first + 1;
            while end < expr.arms.len() && mergeable(&expr.arms[first], &expr.arms[end]) {
                end += 1;
            }

            if end > first + 1 {
                let patterns = expr.arms[first..end]
                    .iter()
                    .map(|arm| arm.pattern_snippet)
                    .collect::<Vec<_>>()
                    .join(" | ");
                let lines = expr.arms[first..end]
                    .iter()
                    .map(|arm| arm.line.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let repeated = &expr.arms[first + 1];
                violations.push(Violation {
                    rule_id: "V27".to_string(),
                    level: ViolationLevel::Info,
                    message: format!(
                        "Braços nas linhas {lines} possuem consequência estruturalmente idêntica; verifique possível copiar/colar ou una os padrões como `{patterns}`"
                    ),
                    location: Location {
                        path: Cow::Borrowed(file.path()),
                        line: repeated.line,
                        column: repeated.column,
                    },
                });
                first = end;
            } else {
                first += 1;
            }
        }
    }
    violations
}

fn mergeable(left: &DecisionArm<'_>, right: &DecisionArm<'_>) -> bool {
    if left.is_catchall
        || right.is_catchall
        || left.pattern_is_range
        || right.pattern_is_range
        || left.body_form == BodyForm::EmptyBlock
        || right.body_form == BodyForm::EmptyBlock
    {
        return false;
    }

    let (Some(left_evidence), Some(right_evidence)) = (&left.mergeability, &right.mergeability)
    else {
        return false;
    };

    !left_evidence.has_macro
        && !right_evidence.has_macro
        && !left_evidence.has_conditional_attribute
        && !right_evidence.has_conditional_attribute
        && !left_evidence.is_placeholder
        && !right_evidence.is_placeholder
        // A AST sintática não prova que bindings homônimos têm o mesmo tipo,
        // requisito obrigatório de um or-pattern Rust. Sem autoridade de tipos,
        // qualquer binding permanece Unknown.
        && left_evidence.bindings.is_empty()
        && right_evidence.bindings.is_empty()
        && left_evidence.guard_structure == right_evidence.guard_structure
        && left_evidence.body_structure == right_evidence.body_structure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::rule_traits::{
        BindingMode, DecisionArmMergeability, DecisionExpr, PatternBinding, ScrutineeForm,
    };
    use std::path::Path;

    struct MockFile {
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
            Path::new("01_core/sample.rs")
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    fn arm(pattern: &'static str, body: &str, line: usize) -> DecisionArm<'static> {
        DecisionArm {
            pattern_snippet: pattern,
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::Call,
            body_snippet: "handle()",
            mergeability: Some(DecisionArmMergeability {
                body_structure: body.to_string(),
                guard_structure: None,
                bindings: vec![],
                has_macro: false,
                has_conditional_attribute: false,
                is_placeholder: false,
            }),
            line,
            column: 8,
        }
    }

    fn file(arms: Vec<DecisionArm<'static>>) -> MockFile {
        MockFile {
            exprs: vec![DecisionExpr {
                snippet_scrutinee: "kind",
                scrutinee_form: ScrutineeForm::Path,
                arms,
                line: 1,
                column: 0,
            }],
        }
    }

    #[test]
    fn emits_one_maximal_group_for_adjacent_equal_bodies() {
        let violations = check(&file(vec![
            arm("A", "call(handle)", 2),
            arm("B", "call(handle)", 3),
            arm("C", "call(handle)", 4),
        ]));
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V27");
        assert_eq!(violations[0].level, ViolationLevel::Info);
        assert_eq!(violations[0].location.line, 3);
        assert!(violations[0].message.contains("`A | B | C`"));
    }

    #[test]
    fn unequal_body_splits_groups_and_does_not_bridge_non_adjacent_arms() {
        let violations = check(&file(vec![
            arm("A", "same", 2),
            arm("Middle", "different", 3),
            arm("B", "same", 4),
        ]));
        assert!(violations.is_empty());
    }

    #[test]
    fn guard_binding_macro_cfg_placeholder_and_unknown_are_fail_closed() {
        let base = arm("A(x)", "use($0)", 2);
        let mut cases = Vec::new();

        let mut guard = arm("B(x)", "use($0)", 3);
        guard.mergeability.as_mut().unwrap().guard_structure = Some("guard".into());
        cases.push(guard);

        let mut binding = arm("B(x)", "use($0)", 3);
        binding.mergeability.as_mut().unwrap().bindings = vec![PatternBinding {
            name: "x",
            mode: BindingMode::Ref,
            mutable: false,
        }];
        cases.push(binding);

        let mut macro_arm = arm("B", "use($0)", 3);
        macro_arm.mergeability.as_mut().unwrap().has_macro = true;
        cases.push(macro_arm);

        let mut cfg = arm("B", "use($0)", 3);
        cfg.mergeability.as_mut().unwrap().has_conditional_attribute = true;
        cases.push(cfg);

        let mut placeholder = arm("B", "use($0)", 3);
        placeholder.mergeability.as_mut().unwrap().is_placeholder = true;
        cases.push(placeholder);

        let mut unknown = arm("B", "use($0)", 3);
        unknown.mergeability = None;
        cases.push(unknown);

        for candidate in cases {
            assert!(check(&file(vec![base.clone(), candidate])).is_empty());
        }
    }

    #[test]
    fn syntactically_compatible_bindings_remain_unknown_without_type_authority() {
        let binding = PatternBinding {
            name: "x",
            mode: BindingMode::Move,
            mutable: false,
        };
        let mut a = arm("A(x)", "use($0)", 2);
        a.mergeability.as_mut().unwrap().bindings = vec![binding.clone()];
        let mut b = arm("B(x)", "use($0)", 3);
        b.mergeability.as_mut().unwrap().bindings = vec![binding];
        assert!(check(&file(vec![a, b])).is_empty());
    }
}
