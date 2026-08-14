//! Teste de calibração de MessageProducer (064-wildcard-calibration.md §4.3).
//!
//! Valida que um braço catch-all que produz mensagem de erro ruidosa
//! (e.g. `_ => format!("cannot apply {op:?} to {a} and {b}")`) é considerado
//! barreira ruidosa (`MessageProducer`), sendo portanto ISENTO de V16.

use std::collections::HashMap;
use std::path::Path;

use crystalline_lint::entities::layer::Language;
use crystalline_lint::entities::rule_traits::{
    BodyForm, DecisionArm, DecisionExpr, HasDecisionArms, ScrutineeForm,
};
use crystalline_lint::rules::wildcard_saturation;

struct MessageProducerFixture {
    path: &'static Path,
    language: Language,
    exprs: Vec<DecisionExpr<'static>>,
}

impl HasDecisionArms<'static> for MessageProducerFixture {
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
fn error_message_arm_is_exempt_from_v16() {
    let exceptions = HashMap::new();

    let arms = vec![
        DecisionArm {
            pattern_snippet: "BinaryOp::Add",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["BinaryOp"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::Call,
            body_snippet: "eval_add(a, b)",
            line: 20,
            column: 8,
        },
        DecisionArm {
            pattern_snippet: "BinaryOp::Sub",
            is_catchall: false,
            bound_ident_used_in_body: false,
            qualified_prefixes: vec!["BinaryOp"],
            has_guard: false,
            guard_is_compound: false,
            pattern_is_range: false,
            pattern_depth: 1,
            or_alternatives: 1,
            body_form: BodyForm::Call,
            body_snippet: "eval_sub(a, b)",
            line: 21,
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
            body_form: BodyForm::MessageProducer,
            body_snippet: "format!(\"cannot apply {op:?} to {a} and {b}\")",
            line: 22,
            column: 8,
        },
    ];

    let file = MessageProducerFixture {
        path: Path::new("01_core/src/eval/operators.rs"),
        language: Language::Rust,
        exprs: vec![DecisionExpr {
            snippet_scrutinee: "op",
            scrutinee_form: ScrutineeForm::Path,
            arms,
            line: 19,
            column: 4,
        }],
    };

    let viols = wildcard_saturation::check(&file, &exceptions);
    assert!(
        viols.is_empty(),
        "Braço com MessageProducer (format! de erro) deve ser isento de V16"
    );
}
