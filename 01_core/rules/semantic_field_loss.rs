//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/semantic-field-loss.md
//! @prompt-hash d356e01a
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
            if observation.kind != SemanticObservationKind::NeutralProjectionDestination {
                return None;
            }
            Some(Violation {
                rule_id: "V24".to_string(),
                level: level.clone(),
                message: format!(
                    "SemanticFieldLoss `{}`: {}",
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
            Path::new("03_infra/font_metrics.rs")
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    #[test]
    fn reports_only_neutral_projection_destination() {
        let subject = Subject {
            observations: vec![
                SemanticObservation {
                    contract_id: "font-id".into(),
                    kind: SemanticObservationKind::NeutralProjectionDestination,
                    detail: "variations default".into(),
                    line: 12,
                    column: 8,
                },
                SemanticObservation {
                    contract_id: "radius".into(),
                    kind: SemanticObservationKind::ContextNeutralArgument,
                    detail: "zero".into(),
                    line: 3,
                    column: 0,
                },
            ],
        };
        let violations = check(&subject, ViolationLevel::Error);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V24");
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }
}
