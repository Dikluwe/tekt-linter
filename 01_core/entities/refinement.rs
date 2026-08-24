//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @prompt-hash 993e71ec
//! @layer L1
//! @updated 2026-08-23

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservableValue {
    Known(String),
    Absent,
    Unknown(UnknownReason),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnknownReason {
    MissingObservable,
    AmbiguousIdentity,
    UnsupportedParser,
    OpaqueConstruction,
    PartialContract,
    BudgetExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureCardinality {
    One,
    Many,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingPolicy {
    Unknown,
    Absent,
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            control if control.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => escaped.push(other),
        }
    }
    escaped.push('"');
    escaped
}

pub fn observable_from_captures(
    mut captures: Vec<String>,
    cardinality: CaptureCardinality,
    missing: MissingPolicy,
) -> ObservableValue {
    captures.sort();
    captures.dedup();
    if captures.is_empty() {
        return match missing {
            MissingPolicy::Unknown => ObservableValue::Unknown(UnknownReason::MissingObservable),
            MissingPolicy::Absent => ObservableValue::Absent,
        };
    }
    match cardinality {
        CaptureCardinality::One if captures.len() == 1 => {
            ObservableValue::Known(captures.remove(0))
        }
        CaptureCardinality::One => ObservableValue::Unknown(UnknownReason::AmbiguousIdentity),
        CaptureCardinality::Many => ObservableValue::Known(format!(
            "[{}]",
            captures
                .iter()
                .map(|value| json_string(value))
                .collect::<Vec<_>>()
                .join(",")
        )),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactFacts {
    pub artifact_id: String,
    pub format_version: u32,
    pub extractor_version: String,
    pub observables: BTreeMap<String, ObservableValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementRelation {
    Preserve {
        source: String,
        target: String,
    },
    MayNormalize {
        source: String,
        target: String,
        accepted_targets: Vec<String>,
    },
    MustNotInvent {
        target: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefinementContract {
    pub id: String,
    pub relations: Vec<RefinementRelation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Witness {
    pub contract_id: String,
    pub relation: String,
    pub source_artifact: String,
    pub target_artifact: String,
    pub source_extractor_version: String,
    pub target_extractor_version: String,
    pub source_observable: Option<String>,
    pub target_observable: String,
    pub source_value: Option<ObservableValue>,
    pub target_value: ObservableValue,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Inconclusive {
    pub contract_id: String,
    pub relation: String,
    pub observable: String,
    pub reason: UnknownReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefinementVerdict {
    Preserved,
    Violated {
        witnesses: Vec<Witness>,
        inconclusive: Vec<Inconclusive>,
    },
    Unknown {
        reasons: Vec<Inconclusive>,
    },
}

fn value(facts: &ArtifactFacts, key: &str) -> ObservableValue {
    facts
        .observables
        .get(key)
        .cloned()
        .unwrap_or(ObservableValue::Unknown(UnknownReason::MissingObservable))
}

fn unknown(value: &ObservableValue) -> Option<UnknownReason> {
    match value {
        ObservableValue::Unknown(reason) => Some(reason.clone()),
        ObservableValue::Known(_) | ObservableValue::Absent => None,
    }
}

fn relation_key(relation: &RefinementRelation) -> (&str, &str, &str) {
    match relation {
        RefinementRelation::Preserve { source, target } => ("preserve", source, target),
        RefinementRelation::MayNormalize { source, target, .. } => {
            ("may-normalize", source, target)
        }
        RefinementRelation::MustNotInvent { target } => ("must-not-invent", "", target),
    }
}

pub fn compare_refinement(
    contract: &RefinementContract,
    source: &ArtifactFacts,
    target: &ArtifactFacts,
) -> RefinementVerdict {
    let mut relations: Vec<&RefinementRelation> = contract.relations.iter().collect();
    relations.sort_by_key(|relation| relation_key(relation));

    let mut witnesses = Vec::new();
    let mut inconclusive = Vec::new();

    for relation in relations {
        let (relation_name, source_key, target_key) = relation_key(relation);
        let source_value = if source_key.is_empty() {
            None
        } else {
            Some(value(source, source_key))
        };
        let target_value = value(target, target_key);

        let unknown_fact = source_value
            .as_ref()
            .and_then(unknown)
            .map(|reason| (source_key, reason))
            .or_else(|| unknown(&target_value).map(|reason| (target_key, reason)));
        if let Some((observable, reason)) = unknown_fact {
            inconclusive.push(Inconclusive {
                contract_id: contract.id.clone(),
                relation: relation_name.to_string(),
                observable: observable.to_string(),
                reason,
            });
            continue;
        }

        let preserved = match relation {
            RefinementRelation::Preserve { .. } => source_value.as_ref() == Some(&target_value),
            RefinementRelation::MayNormalize {
                accepted_targets, ..
            } => {
                source_value.as_ref() == Some(&target_value)
                    || matches!(&target_value, ObservableValue::Known(value) if accepted_targets.contains(value))
            }
            RefinementRelation::MustNotInvent { .. } => target_value == ObservableValue::Absent,
        };

        if !preserved {
            witnesses.push(Witness {
                contract_id: contract.id.clone(),
                relation: relation_name.to_string(),
                source_artifact: source.artifact_id.clone(),
                target_artifact: target.artifact_id.clone(),
                source_extractor_version: source.extractor_version.clone(),
                target_extractor_version: target.extractor_version.clone(),
                source_observable: (!source_key.is_empty()).then(|| source_key.to_string()),
                target_observable: target_key.to_string(),
                source_value,
                target_value,
            });
        }
    }

    witnesses.sort_by(|a, b| {
        (&a.contract_id, &a.relation, &a.target_observable).cmp(&(
            &b.contract_id,
            &b.relation,
            &b.target_observable,
        ))
    });
    inconclusive.sort();

    if !witnesses.is_empty() {
        RefinementVerdict::Violated {
            witnesses,
            inconclusive,
        }
    } else if !inconclusive.is_empty() {
        RefinementVerdict::Unknown {
            reasons: inconclusive,
        }
    } else {
        RefinementVerdict::Preserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(id: &str, values: &[(&str, ObservableValue)]) -> ArtifactFacts {
        ArtifactFacts {
            artifact_id: id.to_string(),
            format_version: 1,
            extractor_version: "test-1".to_string(),
            observables: values
                .iter()
                .map(|(key, value)| ((*key).to_string(), value.clone()))
                .collect(),
        }
    }

    fn preserve(source: &str, target: &str) -> RefinementRelation {
        RefinementRelation::Preserve {
            source: source.to_string(),
            target: target.to_string(),
        }
    }

    #[test]
    fn required_field_is_preserved() {
        let contract = RefinementContract {
            id: "font".to_string(),
            relations: vec![preserve("style.variations", "identity.variations")],
        };
        let source = facts(
            "before",
            &[(
                "style.variations",
                ObservableValue::Known("wght=650".to_string()),
            )],
        );
        let target = facts(
            "after",
            &[(
                "identity.variations",
                ObservableValue::Known("wght=650".to_string()),
            )],
        );
        assert_eq!(
            compare_refinement(&contract, &source, &target),
            RefinementVerdict::Preserved
        );
    }

    #[test]
    fn violation_wins_over_unknown_without_erasing_it() {
        let contract = RefinementContract {
            id: "mixed".to_string(),
            relations: vec![preserve("unknown", "unknown"), preserve("kept", "lost")],
        };
        let source = facts(
            "before",
            &[
                (
                    "unknown",
                    ObservableValue::Unknown(UnknownReason::OpaqueConstruction),
                ),
                ("kept", ObservableValue::Known("yes".to_string())),
            ],
        );
        let target = facts(
            "after",
            &[
                ("unknown", ObservableValue::Known("x".to_string())),
                ("lost", ObservableValue::Known("no".to_string())),
            ],
        );
        match compare_refinement(&contract, &source, &target) {
            RefinementVerdict::Violated {
                witnesses,
                inconclusive,
            } => {
                assert_eq!(witnesses.len(), 1);
                assert_eq!(inconclusive.len(), 1);
            }
            verdict => panic!("unexpected verdict: {verdict:?}"),
        }
    }

    #[test]
    fn missing_observable_is_unknown() {
        let contract = RefinementContract {
            id: "missing".to_string(),
            relations: vec![preserve("field", "field")],
        };
        assert!(matches!(
            compare_refinement(&contract, &facts("a", &[]), &facts("b", &[])),
            RefinementVerdict::Unknown { .. }
        ));
    }

    #[test]
    fn relation_order_does_not_change_result() {
        let mut left = RefinementContract {
            id: "order".to_string(),
            relations: vec![preserve("b", "b"), preserve("a", "a")],
        };
        let values = facts(
            "same",
            &[
                ("a", ObservableValue::Known("1".to_string())),
                ("b", ObservableValue::Known("2".to_string())),
            ],
        );
        let first = compare_refinement(&left, &values, &values);
        left.relations.reverse();
        assert_eq!(first, compare_refinement(&left, &values, &values));
    }

    #[test]
    fn capture_policy_distinguishes_missing_ambiguous_and_many() {
        assert_eq!(
            observable_from_captures(vec![], CaptureCardinality::One, MissingPolicy::Absent),
            ObservableValue::Absent
        );
        assert_eq!(
            observable_from_captures(
                vec!["b".to_string(), "a".to_string()],
                CaptureCardinality::One,
                MissingPolicy::Unknown
            ),
            ObservableValue::Unknown(UnknownReason::AmbiguousIdentity)
        );
        assert_eq!(
            observable_from_captures(
                vec!["b".to_string(), "a".to_string()],
                CaptureCardinality::Many,
                MissingPolicy::Unknown
            ),
            ObservableValue::Known("[\"a\",\"b\"]".to_string())
        );
    }
}
