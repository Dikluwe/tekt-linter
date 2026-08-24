//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/decision-ownership.md
//! @prompt-hash e05a4cf8
//! @layer L1
//! @updated 2026-08-23

use crate::entities::rule_traits::{HasSemanticObservations, SemanticObservationKind};
use crate::entities::violation::{Location, Violation, ViolationLevel};
use std::borrow::Cow;

pub fn check<'a, T: HasSemanticObservations<'a>>(
    file: &T,
    level: ViolationLevel,
) -> Vec<Violation<'a>> {
    file.semantic_observations()
        .iter()
        .filter_map(|observation| {
            let modality = match observation.kind {
                SemanticObservationKind::DuplicateDecisionOwner => "duplicate-owner",
                SemanticObservationKind::DecisionProxyReentry => "proxy-reentry",
                SemanticObservationKind::CanonicalizerReentry => "canonicalizer-reentry",
                SemanticObservationKind::DirectDecisionReimplementation => {
                    "direct-reimplementation"
                }
                SemanticObservationKind::ContextNeutralArgument
                | SemanticObservationKind::ContextErasingProjection
                | SemanticObservationKind::NeutralProjectionDestination => return None,
            };
            Some(Violation {
                rule_id: "V25".to_string(),
                level: level.clone(),
                message: format!(
                    "DecisionOwnership `{}` ({modality}): {}",
                    observation.contract_id, observation.detail
                ),
                location: Location {
                    path: Cow::Borrowed(file.path()),
                    line: observation.line,
                    column: observation.column,
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Language;
    use crate::entities::rule_traits::{
        HasSemanticObservations, SemanticObservation, SemanticObservationKind,
    };
    use std::path::Path;

    struct Subject {
        observations: Vec<SemanticObservation>,
    }
    impl HasSemanticObservations<'static> for Subject {
        fn semantic_observations(&self) -> &[SemanticObservation] {
            &self.observations
        }
        fn path(&self) -> &'static Path {
            Path::new("03_infra/shaper.rs")
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    #[test]
    fn reports_all_three_ownership_modalities() {
        let kinds = [
            SemanticObservationKind::DuplicateDecisionOwner,
            SemanticObservationKind::DecisionProxyReentry,
            SemanticObservationKind::CanonicalizerReentry,
        ];
        let subject = Subject {
            observations: kinds
                .into_iter()
                .enumerate()
                .map(|(i, kind)| SemanticObservation {
                    contract_id: "math".into(),
                    kind,
                    detail: "ownership".into(),
                    line: i + 1,
                    column: 0,
                })
                .collect(),
        };
        let violations = check(&subject, ViolationLevel::Warning);
        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.rule_id == "V25"));
    }
}
