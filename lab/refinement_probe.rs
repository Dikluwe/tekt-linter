//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @layer Lab
//! @updated 2026-08-23
//!
//! Experimento descartável: refinamento direcional de fatos normalizados.
//!
//! Não pertence ao produto, não é importável por L1–L4 e não define API pública.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Known(&'static str),
    Missing,
    Unknown(&'static str),
}

type Facts = BTreeMap<&'static str, Value>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Relation {
    Preserve {
        source: &'static str,
        target: &'static str,
    },
    MayNormalize {
        source: &'static str,
        target: &'static str,
        accepted_targets: Vec<&'static str>,
    },
    MustNotInvent {
        target: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Contract {
    id: &'static str,
    relations: Vec<Relation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Witness {
    contract_id: &'static str,
    relation: &'static str,
    source_key: Option<&'static str>,
    target_key: &'static str,
    source_value: Option<Value>,
    target_value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    Preserved,
    Violated(Witness),
    Unknown {
        contract_id: &'static str,
        relation: &'static str,
        reason: &'static str,
    },
}

fn fact(facts: &Facts, key: &'static str) -> Value {
    facts.get(key).cloned().unwrap_or(Value::Missing)
}

fn unknown_reason(value: &Value) -> Option<&'static str> {
    match value {
        Value::Unknown(reason) => Some(reason),
        Value::Known(_) | Value::Missing => None,
    }
}

fn check(contract: &Contract, source: &Facts, target: &Facts) -> Verdict {
    for relation in &contract.relations {
        match relation {
            Relation::Preserve {
                source: source_key,
                target: target_key,
            } => {
                let source_value = fact(source, source_key);
                let target_value = fact(target, target_key);
                if let Some(reason) = unknown_reason(&source_value).or(unknown_reason(&target_value)) {
                    return Verdict::Unknown {
                        contract_id: contract.id,
                        relation: "preserve",
                        reason,
                    };
                }
                if source_value != target_value {
                    return Verdict::Violated(Witness {
                        contract_id: contract.id,
                        relation: "preserve",
                        source_key: Some(source_key),
                        target_key,
                        source_value: Some(source_value),
                        target_value,
                    });
                }
            }
            Relation::MayNormalize {
                source: source_key,
                target: target_key,
                accepted_targets,
            } => {
                let source_value = fact(source, source_key);
                let target_value = fact(target, target_key);
                if let Some(reason) = unknown_reason(&source_value).or(unknown_reason(&target_value)) {
                    return Verdict::Unknown {
                        contract_id: contract.id,
                        relation: "may-normalize",
                        reason,
                    };
                }
                let accepted = source_value == target_value
                    || matches!(&target_value, Value::Known(value) if accepted_targets.contains(value));
                if !accepted {
                    return Verdict::Violated(Witness {
                        contract_id: contract.id,
                        relation: "may-normalize",
                        source_key: Some(source_key),
                        target_key,
                        source_value: Some(source_value),
                        target_value,
                    });
                }
            }
            Relation::MustNotInvent { target: target_key } => {
                let target_value = fact(target, target_key);
                if let Some(reason) = unknown_reason(&target_value) {
                    return Verdict::Unknown {
                        contract_id: contract.id,
                        relation: "must-not-invent",
                        reason,
                    };
                }
                if target_value != Value::Missing {
                    return Verdict::Violated(Witness {
                        contract_id: contract.id,
                        relation: "must-not-invent",
                        source_key: None,
                        target_key,
                        source_value: None,
                        target_value,
                    });
                }
            }
        }
    }
    Verdict::Preserved
}

fn facts(entries: &[(&'static str, Value)]) -> Facts {
    entries.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preserve_variations() -> Contract {
        Contract {
            id: "font-identity",
            relations: vec![Relation::Preserve {
                source: "style.variations",
                target: "identity.variations",
            }],
        }
    }

    #[test]
    fn preserved_when_required_field_survives() {
        let source = facts(&[("style.variations", Value::Known("wght=650"))]);
        let target = facts(&[("identity.variations", Value::Known("wght=650"))]);
        assert_eq!(check(&preserve_variations(), &source, &target), Verdict::Preserved);
    }

    #[test]
    fn violated_with_reproducible_witness_when_field_is_erased() {
        let source = facts(&[("style.variations", Value::Known("wght=650"))]);
        let target = facts(&[("identity.variations", Value::Known("default"))]);
        assert_eq!(
            check(&preserve_variations(), &source, &target),
            Verdict::Violated(Witness {
                contract_id: "font-identity",
                relation: "preserve",
                source_key: Some("style.variations"),
                target_key: "identity.variations",
                source_value: Some(Value::Known("wght=650")),
                target_value: Value::Known("default"),
            })
        );
    }

    #[test]
    fn unknown_evidence_is_not_reported_as_preserved() {
        let source = facts(&[("style.variations", Value::Unknown("macro-opaque"))]);
        let target = facts(&[("identity.variations", Value::Known("default"))]);
        assert_eq!(
            check(&preserve_variations(), &source, &target),
            Verdict::Unknown {
                contract_id: "font-identity",
                relation: "preserve",
                reason: "macro-opaque",
            }
        );
    }

    #[test]
    fn refinement_is_directional() {
        let source = facts(&[("style.variations", Value::Known("wght=650"))]);
        let target = facts(&[("identity.variations", Value::Known("default"))]);
        assert!(matches!(check(&preserve_variations(), &source, &target), Verdict::Violated(_)));

        let reverse_contract = Contract {
            id: "reverse-font-identity",
            relations: vec![Relation::MayNormalize {
                source: "identity.variations",
                target: "style.variations",
                accepted_targets: vec!["wght=650"],
            }],
        };
        assert_eq!(check(&reverse_contract, &target, &source), Verdict::Preserved);
    }

    #[test]
    fn declared_normalization_is_preserved() {
        let contract = Contract {
            id: "weight-normalization",
            relations: vec![Relation::MayNormalize {
                source: "style.weight",
                target: "identity.weight",
                accepted_targets: vec!["700"],
            }],
        };
        let source = facts(&[("style.weight", Value::Known("bold"))]);
        let target = facts(&[("identity.weight", Value::Known("700"))]);
        assert_eq!(check(&contract, &source, &target), Verdict::Preserved);
    }

    #[test]
    fn undeclared_normalization_is_a_violation() {
        let source = facts(&[("style.variations", Value::Known("wght=650"))]);
        let target = facts(&[("identity.variations", Value::Known("normalized"))]);
        assert!(matches!(check(&preserve_variations(), &source, &target), Verdict::Violated(_)));
    }

    #[test]
    fn contextual_value_cannot_become_erased() {
        let contract = Contract {
            id: "rounded-radius",
            relations: vec![Relation::Preserve {
                source: "radius.state",
                target: "rendered-radius.state",
            }],
        };
        let source = facts(&[("radius.state", Value::Known("contextual"))]);
        let target = facts(&[("rendered-radius.state", Value::Known("erased"))]);
        assert!(matches!(check(&contract, &source, &target), Verdict::Violated(_)));
    }

    #[test]
    fn target_cannot_invent_a_second_decision_owner() {
        let contract = Contract {
            id: "math-style-authority",
            relations: vec![Relation::MustNotInvent {
                target: "decision.proxy-owner",
            }],
        };
        let source = facts(&[]);
        let target = facts(&[("decision.proxy-owner", Value::Known("family.contains(math)"))]);
        assert!(matches!(check(&contract, &source, &target), Verdict::Violated(_)));
    }

    #[test]
    fn absent_forbidden_fact_is_preserved() {
        let contract = Contract {
            id: "math-style-authority",
            relations: vec![Relation::MustNotInvent {
                target: "decision.proxy-owner",
            }],
        };
        assert_eq!(check(&contract, &facts(&[]), &facts(&[])), Verdict::Preserved);
    }
}
