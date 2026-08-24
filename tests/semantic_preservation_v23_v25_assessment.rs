//! Gate segregado do Assessment 0015 (V23--V25).
//! Expectativas derivadas somente dos quatro L0 hash-pinned.

use std::path::Path;

use crystalline_lint::entities::layer::Language;
use crystalline_lint::entities::rule_traits::{
    HasSemanticObservations, SemanticObservation, SemanticObservationKind,
};
use crystalline_lint::entities::violation::{Violation, ViolationLevel};
use crystalline_lint::rules::{context_erasure, decision_ownership, semantic_field_loss};

struct PublicFile {
    path: &'static Path,
    language: Language,
    observations: Vec<SemanticObservation>,
}

impl<'a> HasSemanticObservations<'a> for PublicFile {
    fn semantic_observations(&self) -> &[SemanticObservation] {
        &self.observations
    }
    fn path(&self) -> &'a Path {
        self.path
    }
    fn language(&self) -> &Language {
        &self.language
    }
}

fn obs(
    kind: SemanticObservationKind,
    contract: &str,
    detail: &str,
    line: usize,
    column: usize,
) -> SemanticObservation {
    SemanticObservation {
        contract_id: contract.into(),
        kind,
        detail: detail.into(),
        line,
        column,
    }
}

fn file(language: Language, observations: Vec<SemanticObservation>) -> PublicFile {
    PublicFile {
        path: Path::new("unicode/λ-gate.rs"),
        language,
        observations,
    }
}

fn complete_input() -> Vec<SemanticObservation> {
    use SemanticObservationKind::*;
    vec![
        obs(ContextNeutralArgument, "c23-a", "neutral λ", 1, 2),
        obs(ContextErasingProjection, "c23-b", "erase 🙂", 3, 4),
        obs(NeutralProjectionDestination, "c24", "field loss", 5, 6),
        obs(DuplicateDecisionOwner, "c25-a", "owners", 7, 8),
        obs(DecisionProxyReentry, "c25-b", "proxy", 9, 10),
        obs(CanonicalizerReentry, "c25-c", "canon", 11, 12),
        obs(DirectDecisionReimplementation, "c25-d", "direct", 13, 14),
    ]
}

fn evidence(v: &Violation<'_>) -> (String, ViolationLevel, String, String, usize, usize) {
    (
        v.rule_id.clone(),
        v.level.clone(),
        v.message.clone(),
        v.location.path.display().to_string(),
        v.location.line,
        v.location.column,
    )
}

fn assert_diagnostic_evidence(
    violation: &Violation<'_>,
    rule_id: &str,
    contract_id: &str,
    detail: &str,
    line: usize,
    column: usize,
) {
    assert_eq!(violation.rule_id, rule_id);
    assert!(violation.message.contains(contract_id));
    assert!(violation.message.contains(detail));
    assert_eq!(violation.location.path, Path::new("unicode/λ-gate.rs"));
    assert_eq!(
        (violation.location.line, violation.location.column),
        (line, column)
    );
}

#[test]
fn complete_seven_by_three_matrix_and_v25_modalities() {
    let f = file(Language::Rust, complete_input());
    let v23 = context_erasure::check(&f, ViolationLevel::Warning);
    let v24 = semantic_field_loss::check(&f, ViolationLevel::Warning);
    let v25 = decision_ownership::check(&f, ViolationLevel::Warning);
    assert_eq!(v23.len(), 2);
    assert_eq!(v24.len(), 1);
    assert_eq!(v25.len(), 4);
    for (v, contract, detail, line, column) in [
        (&v23[0], "c23-a", "neutral λ", 1, 2),
        (&v23[1], "c23-b", "erase 🙂", 3, 4),
    ] {
        assert_diagnostic_evidence(v, "V23", contract, detail, line, column);
    }
    assert_diagnostic_evidence(&v24[0], "V24", "c24", "field loss", 5, 6);
    for (v, contract, detail, mode) in [
        (&v25[0], "c25-a", "owners", "duplicate-owner"),
        (&v25[1], "c25-b", "proxy", "proxy-reentry"),
        (&v25[2], "c25-c", "canon", "canonicalizer-reentry"),
        (&v25[3], "c25-d", "direct", "direct-reimplementation"),
    ] {
        assert!(v.message.contains(contract));
        assert!(v.message.contains(detail));
        assert!(v.message.contains(mode));
    }
    for (v, contract, detail, line, column) in [
        (&v25[0], "c25-a", "owners", 7, 8),
        (&v25[1], "c25-b", "proxy", 9, 10),
        (&v25[2], "c25-c", "canon", 11, 12),
        (&v25[3], "c25-d", "direct", 13, 14),
    ] {
        assert_diagnostic_evidence(v, "V25", contract, detail, line, column);
    }
}

#[test]
fn every_public_level_is_preserved_exactly() {
    for level in [
        ViolationLevel::Info,
        ViolationLevel::Warning,
        ViolationLevel::Error,
        ViolationLevel::Fatal,
    ] {
        let f = file(
            Language::Rust,
            vec![
                obs(
                    SemanticObservationKind::ContextNeutralArgument,
                    "a",
                    "d",
                    1,
                    1,
                ),
                obs(
                    SemanticObservationKind::NeutralProjectionDestination,
                    "b",
                    "d",
                    2,
                    2,
                ),
                obs(
                    SemanticObservationKind::DuplicateDecisionOwner,
                    "c",
                    "d",
                    3,
                    3,
                ),
            ],
        );
        assert_eq!(context_erasure::check(&f, level.clone())[0].level, level);
        assert_eq!(
            semantic_field_loss::check(&f, level.clone())[0].level,
            level
        );
        assert_eq!(decision_ownership::check(&f, level.clone())[0].level, level);
    }
}

#[test]
fn evidence_order_multiplicity_permutation_unicode_empty_and_extremes() {
    use SemanticObservationKind::ContextNeutralArgument as K;
    let a = obs(K, "contrato λ", "detalhe 🙂", 0, usize::MAX);
    let b = obs(K, "", "", usize::MAX, 0);
    let f1 = file(Language::Rust, vec![a.clone(), b.clone(), a.clone()]);
    let f2 = file(Language::Rust, vec![b, a]);
    let x = context_erasure::check(&f1, ViolationLevel::Error);
    let y = context_erasure::check(&f2, ViolationLevel::Error);
    assert_eq!(x.len(), 3);
    assert_eq!(y.len(), 2);
    assert_eq!((x[0].location.line, x[0].location.column), (0, usize::MAX));
    assert_eq!((x[1].location.line, x[1].location.column), (usize::MAX, 0));
    assert_eq!((x[2].location.line, x[2].location.column), (0, usize::MAX));
    assert_eq!((y[0].location.line, y[0].location.column), (usize::MAX, 0));
    assert_eq!((y[1].location.line, y[1].location.column), (0, usize::MAX));
    assert!(x[0].message.contains("contrato λ"));
    assert!(x[0].message.contains("detalhe 🙂"));
    assert_eq!(x[0].location.path, Path::new("unicode/λ-gate.rs"));
}

#[test]
fn language_and_ignored_fields_are_integrally_inert() {
    let eligible = obs(
        SemanticObservationKind::NeutralProjectionDestination,
        "id",
        "detail",
        4,
        5,
    );
    let rust = file(
        Language::Rust,
        vec![
            obs(
                SemanticObservationKind::ContextNeutralArgument,
                "ignored-a",
                "x",
                1,
                2,
            ),
            eligible.clone(),
        ],
    );
    let python = file(
        Language::Python,
        vec![
            obs(
                SemanticObservationKind::DirectDecisionReimplementation,
                "ignored-b",
                "🙂",
                usize::MAX,
                0,
            ),
            eligible,
        ],
    );
    let a = semantic_field_loss::check(&rust, ViolationLevel::Info);
    let b = semantic_field_loss::check(&python, ViolationLevel::Info);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(evidence(&a[0]), evidence(&b[0]));
}

#[test]
fn no_cross_activation_and_empty_is_total_for_all_rules() {
    let empty = file(Language::Rust, vec![]);
    assert!(context_erasure::check(&empty, ViolationLevel::Info).is_empty());
    assert!(semantic_field_loss::check(&empty, ViolationLevel::Warning).is_empty());
    assert!(decision_ownership::check(&empty, ViolationLevel::Fatal).is_empty());
    for (kind, expected) in [
        (SemanticObservationKind::ContextNeutralArgument, (1, 0, 0)),
        (SemanticObservationKind::ContextErasingProjection, (1, 0, 0)),
        (
            SemanticObservationKind::NeutralProjectionDestination,
            (0, 1, 0),
        ),
        (SemanticObservationKind::DuplicateDecisionOwner, (0, 0, 1)),
        (SemanticObservationKind::DecisionProxyReentry, (0, 0, 1)),
        (SemanticObservationKind::CanonicalizerReentry, (0, 0, 1)),
        (
            SemanticObservationKind::DirectDecisionReimplementation,
            (0, 0, 1),
        ),
    ] {
        let f = file(Language::Unknown, vec![obs(kind, "", "", 0, usize::MAX)]);
        assert_eq!(
            (
                context_erasure::check(&f, ViolationLevel::Info).len(),
                semantic_field_loss::check(&f, ViolationLevel::Info).len(),
                decision_ownership::check(&f, ViolationLevel::Info).len()
            ),
            expected
        );
    }
}
