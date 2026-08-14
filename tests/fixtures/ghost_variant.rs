//! Teste de mutação (ADR-0016 §7.1 / wildcard-saturation.md §7.1).
//!
//! Demonstra que a regra V16 aponta ao mecanismo sintáctico de saturação silenciosa,
//! não a um texto estático: um enum de domínio com fallback `_ => Unit::Percent` é
//! violado tanto na forma base quanto após a adição de uma variante fantasma (`GhostVariant`),
//! mantendo o diagnóstico exactamente no mesmo braço.

use std::path::Path;
use std::collections::HashMap;

use crystalline_lint::entities::layer::Language;
use crystalline_lint::entities::rule_traits::{
    BodyForm, DecisionArm, DecisionExpr, HasDecisionArms, ScrutineeForm,
};
use crystalline_lint::rules::wildcard_saturation;

struct MutationFixture {
    path: &'static Path,
    language: Language,
    exprs: Vec<DecisionExpr<'static>>,
}

impl HasDecisionArms<'static> for MutationFixture {
    fn layer(&self) -> &crystalline_lint::entities::layer::Layer {
        &crystalline_lint::entities::layer::Layer::L1
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
fn ghost_variant_mutation_preserves_v16_violation() {
    let exceptions = HashMap::new();

    // 1. Caso Base: Enum Unit com Pt, Mm e wildcard catch-all saturando para Percent
    let base_arms = vec![
        DecisionArm {
            pattern_snippet: "Unit::Pt",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 10,
            column: 8,
        },
        DecisionArm {
            pattern_snippet: "Unit::Mm",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 11,
            column: 8,
        },
        DecisionArm {
            pattern_snippet: "_",
            is_catchall: true,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::EnumPath,
            body_snippet: "Unit::Percent",
            line: 12,
            column: 8,
        },
    ];

    let base_file = MutationFixture {
        path: Path::new("01_core/src/unit.rs"),
        language: Language::Rust,
        exprs: vec![DecisionExpr {
            snippet_scrutinee: "unit",
            scrutinee_form: ScrutineeForm::Path,
            arms: base_arms,
            line: 9,
            column: 4,
        }],
    };

    let base_violations = wildcard_saturation::check(&base_file, &exceptions);
    assert_eq!(base_violations.len(), 1, "Base deve conter exactamente 1 violação V16");
    assert_eq!(base_violations[0].location.line, 12);
    assert!(base_violations[0].message.contains("Unit::Percent"));

    // 2. Mutação: Adição de uma variante fantasma Unit::GhostVariant (sem braço explícito no match)
    // O compilador aceita silenciosamente devido ao wildcard, mas a regra V16 continua mordendo o braço 12.
    let mutated_arms = vec![
        DecisionArm {
            pattern_snippet: "Unit::Pt",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 10,
            column: 8,
        },
        DecisionArm {
            pattern_snippet: "Unit::Mm",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["Unit"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::LiteralNeutral,
            body_snippet: "0",
            line: 11,
            column: 8,
        },
        DecisionArm {
            pattern_snippet: "_",
            is_catchall: true,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec![],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::EnumPath,
            body_snippet: "Unit::Percent",
            line: 12,
            column: 8,
        },
    ];

    let mutated_file = MutationFixture {
        path: Path::new("01_core/src/unit.rs"),
        language: Language::Rust,
        exprs: vec![DecisionExpr {
            snippet_scrutinee: "unit",
            scrutinee_form: ScrutineeForm::Path,
            arms: mutated_arms,
            line: 9,
            column: 4,
        }],
    };

    let mutated_violations = wildcard_saturation::check(&mutated_file, &exceptions);
    assert_eq!(mutated_violations.len(), 1, "Mutação deve manter a violação V16");
    assert_eq!(mutated_violations[0].location.line, 12);
    assert_eq!(mutated_violations[0].message, base_violations[0].message);
}
