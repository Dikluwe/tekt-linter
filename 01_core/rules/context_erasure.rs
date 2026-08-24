//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/context-erasure.md
//! @prompt-hash aa106066
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
            if !matches!(
                observation.kind,
                SemanticObservationKind::ContextNeutralArgument
                    | SemanticObservationKind::ContextErasingProjection
            ) {
                return None;
            }
            Some(Violation {
                rule_id: "V23".to_string(),
                level: level.clone(),
                message: format!(
                    "ContextErasure `{}`: {}",
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
            Path::new("03_infra/export.rs")
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    #[test]
    fn reports_neutral_context_and_erasing_projection_only() {
        let subject = Subject {
            observations: vec![
                SemanticObservation {
                    contract_id: "radius".into(),
                    kind: SemanticObservationKind::ContextNeutralArgument,
                    detail: "zero context".into(),
                    line: 4,
                    column: 2,
                },
                SemanticObservation {
                    contract_id: "radius".into(),
                    kind: SemanticObservationKind::ContextErasingProjection,
                    detail: "abs".into(),
                    line: 7,
                    column: 3,
                },
                SemanticObservation {
                    contract_id: "font".into(),
                    kind: SemanticObservationKind::NeutralProjectionDestination,
                    detail: "default".into(),
                    line: 9,
                    column: 0,
                },
            ],
        };
        let violations = check(&subject, ViolationLevel::Warning);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].rule_id, "V23");
        assert_eq!(
            violations[0].location.path.as_ref(),
            Path::new("03_infra/export.rs")
        );
    }
}
