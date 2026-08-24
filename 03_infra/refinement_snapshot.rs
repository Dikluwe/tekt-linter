//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @prompt-hash e10c5722
//! @layer L3
//! @updated 2026-08-23

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::entities::refinement::{
    ArtifactFacts, ObservableValue, RefinementContract, RefinementRelation, UnknownReason,
};

#[derive(Deserialize)]
struct SnapshotDto {
    format_version: u32,
    artifact_id: String,
    extractor_version: String,
    observables: BTreeMap<String, ObservableDto>,
}

#[derive(Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
enum ObservableDto {
    Known { value: String },
    Absent,
    Unknown { reason: UnknownReasonDto },
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum UnknownReasonDto {
    MissingObservable,
    AmbiguousIdentity,
    UnsupportedParser,
    OpaqueConstruction,
    PartialContract,
    BudgetExhausted,
}

#[derive(Deserialize)]
struct ContractDto {
    id: String,
    relation: Vec<RelationDto>,
}

#[derive(Deserialize)]
struct RelationDto {
    kind: String,
    source: Option<String>,
    target: String,
    #[serde(default)]
    accepted_targets: Vec<String>,
}

fn reason(dto: UnknownReasonDto) -> UnknownReason {
    match dto {
        UnknownReasonDto::MissingObservable => UnknownReason::MissingObservable,
        UnknownReasonDto::AmbiguousIdentity => UnknownReason::AmbiguousIdentity,
        UnknownReasonDto::UnsupportedParser => UnknownReason::UnsupportedParser,
        UnknownReasonDto::OpaqueConstruction => UnknownReason::OpaqueConstruction,
        UnknownReasonDto::PartialContract => UnknownReason::PartialContract,
        UnknownReasonDto::BudgetExhausted => UnknownReason::BudgetExhausted,
    }
}

pub fn load_snapshot(path: &Path) -> Result<ArtifactFacts, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("cannot read snapshot {}: {error}", path.display()))?;
    let dto: SnapshotDto = serde_json::from_str(&content)
        .map_err(|error| format!("invalid snapshot {}: {error}", path.display()))?;
    if dto.format_version != 1 {
        return Err(format!(
            "unsupported snapshot format_version {} in {}; expected 1",
            dto.format_version,
            path.display()
        ));
    }
    if dto.artifact_id.trim().is_empty() || dto.extractor_version.trim().is_empty() {
        return Err(format!(
            "snapshot {} requires non-empty artifact_id and extractor_version",
            path.display()
        ));
    }
    let observables = dto
        .observables
        .into_iter()
        .map(|(key, value)| {
            let value = match value {
                ObservableDto::Known { value } => ObservableValue::Known(value),
                ObservableDto::Absent => ObservableValue::Absent,
                ObservableDto::Unknown { reason: dto } => ObservableValue::Unknown(reason(dto)),
            };
            (key, value)
        })
        .collect();
    Ok(ArtifactFacts {
        artifact_id: dto.artifact_id,
        format_version: dto.format_version,
        extractor_version: dto.extractor_version,
        observables,
    })
}

pub fn load_contract(path: &Path) -> Result<RefinementContract, String> {
    let content = fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read refinement contract {}: {error}",
            path.display()
        )
    })?;
    let dto: ContractDto = toml::from_str(&content)
        .map_err(|error| format!("invalid refinement contract {}: {error}", path.display()))?;
    if dto.id.trim().is_empty() || dto.relation.is_empty() {
        return Err(format!(
            "refinement contract {} requires id and at least one [[relation]]",
            path.display()
        ));
    }
    let mut relations = Vec::with_capacity(dto.relation.len());
    for relation in dto.relation {
        if relation.target.trim().is_empty() {
            return Err("refinement relation requires non-empty target".to_string());
        }
        let source = || {
            relation
                .source
                .clone()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| format!("{} relation requires source", relation.kind))
        };
        let compiled = match relation.kind.as_str() {
            "preserve" => RefinementRelation::Preserve {
                source: source()?,
                target: relation.target,
            },
            "may-normalize" => {
                if relation.accepted_targets.is_empty() {
                    return Err("may-normalize relation requires accepted_targets".to_string());
                }
                RefinementRelation::MayNormalize {
                    source: source()?,
                    target: relation.target,
                    accepted_targets: relation.accepted_targets,
                }
            }
            "must-not-invent" => RefinementRelation::MustNotInvent {
                target: relation.target,
            },
            other => return Err(format!("unsupported refinement relation kind `{other}`")),
        };
        relations.push(compiled);
    }
    Ok(RefinementContract {
        id: dto.id,
        relations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_snapshot_and_contract() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot = dir.path().join("facts.json");
        let contract = dir.path().join("contract.toml");
        fs::write(
            &snapshot,
            r#"{"format_version":1,"artifact_id":"before","extractor_version":"manual-1","observables":{"field":{"state":"known","value":"x"}}}"#,
        )
        .unwrap();
        fs::write(
            &contract,
            "id = \"example\"\n[[relation]]\nkind = \"preserve\"\nsource = \"field\"\ntarget = \"field\"\n",
        )
        .unwrap();
        assert_eq!(load_snapshot(&snapshot).unwrap().artifact_id, "before");
        assert_eq!(load_contract(&contract).unwrap().relations.len(), 1);
    }

    #[test]
    fn rejects_unknown_snapshot_version() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot = dir.path().join("facts.json");
        fs::write(
            &snapshot,
            r#"{"format_version":2,"artifact_id":"a","extractor_version":"x","observables":{}}"#,
        )
        .unwrap();
        assert!(load_snapshot(&snapshot).unwrap_err().contains("expected 1"));
    }
}
