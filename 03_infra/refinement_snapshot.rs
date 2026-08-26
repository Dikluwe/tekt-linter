//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/refinement-snapshot.md
//! @prompt-hash 21de06b7
//! @layer L3
//! @updated 2026-08-25

use crate::entities::refinement::{
    ArtifactFacts, ObservableValue, RefinementContract, RefinementRelation, UnknownReason,
};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_STRING: usize = 64 * 1024;
const MAX_ITEMS: usize = 4096;
const MAX_ACCEPTED_TOTAL: usize = 16_384;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotDto {
    format_version: u32,
    artifact_id: String,
    extractor_version: String,
    observables: BTreeMap<String, ObservableDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservableDto {
    state: String,
    value: Option<String>,
    reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContractDto {
    id: String,
    #[serde(rename = "observable")]
    _observables: Option<Vec<toml::Value>>,
    relation: Vec<RelationDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationDto {
    kind: String,
    source: Option<String>,
    target: String,
    accepted_targets: Option<Vec<String>>,
}

struct DuplicateSafe;
impl<'de> Deserialize<'de> for DuplicateSafe {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(DuplicateVisitor)
    }
}
struct DuplicateVisitor;
impl<'de> Visitor<'de> for DuplicateVisitor {
    type Value = DuplicateSafe;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JSON without duplicate keys")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > MAX_STRING {
                return Err(de::Error::custom("limit: string exceeds 64 KiB"));
            }
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate key `{key}`")));
            }
            map.next_value::<DuplicateSafe>()?;
        }
        Ok(DuplicateSafe)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        while seq.next_element::<DuplicateSafe>()?.is_some() {}
        Ok(DuplicateSafe)
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        if value.len() > MAX_STRING {
            return Err(E::custom("limit: string exceeds 64 KiB"));
        }
        Ok(DuplicateSafe)
    }
    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        self.visit_str(&value)
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(DuplicateSafe)
    }
}

fn bounded<'a>(bytes: &'a [u8], source: &str) -> Result<&'a str, String> {
    if bytes.len() > MAX_BYTES {
        return Err(format!("limit: {source}: input exceeds 4 MiB"));
    }
    std::str::from_utf8(bytes).map_err(|_| format!("invalid-utf8: {source}"))
}

fn read_regular(path: &Path) -> Result<Vec<u8>, String> {
    let source = path.display().to_string();
    let mut prefix = std::path::PathBuf::new();
    for component in path.components() {
        prefix.push(component.as_os_str());
        let component_meta =
            fs::symlink_metadata(&prefix).map_err(|e| format!("io: {source}: {e}"))?;
        if component_meta.file_type().is_symlink() {
            return Err(format!(
                "io: {source}: expected path without symlink components"
            ));
        }
    }
    let meta = fs::symlink_metadata(path).map_err(|e| format!("io: {source}: {e}"))?;
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err(format!("io: {source}: expected regular non-symlink file"));
    }
    if meta.len() > MAX_BYTES as u64 {
        return Err(format!("limit: {source}: input exceeds 4 MiB"));
    }
    let file = File::open(path).map_err(|e| format!("io: {source}: {e}"))?;
    let before = file.metadata().map_err(|e| format!("io: {source}: {e}"))?;
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&file)
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("io: {source}: {e}"))?;
    if bytes.len() > MAX_BYTES {
        return Err(format!("limit: {source}: input exceeds 4 MiB"));
    }
    let after = file.metadata().map_err(|e| format!("io: {source}: {e}"))?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        return Err(format!("concurrent-modification: {source}"));
    }
    Ok(bytes)
}

fn reason(value: &str, source: &str) -> Result<UnknownReason, String> {
    match value {
        "missing-observable" => Ok(UnknownReason::MissingObservable),
        "ambiguous-identity" => Ok(UnknownReason::AmbiguousIdentity),
        "unsupported-parser" => Ok(UnknownReason::UnsupportedParser),
        "opaque-construction" => Ok(UnknownReason::OpaqueConstruction),
        "partial-contract" => Ok(UnknownReason::PartialContract),
        "budget-exhausted" => Ok(UnknownReason::BudgetExhausted),
        _ => Err(format!("schema: {source}: unsupported unknown reason")),
    }
}

pub fn load_snapshot(path: &Path) -> Result<ArtifactFacts, String> {
    let bytes = read_regular(path)?;
    load_snapshot_from_bytes(&bytes, &path.display().to_string())
}

pub fn load_snapshot_from_bytes(bytes: &[u8], source: &str) -> Result<ArtifactFacts, String> {
    let content = bounded(bytes, source)?;
    if let Err(error) = serde_json::from_str::<DuplicateSafe>(content) {
        let class = if error.to_string().contains("limit: string exceeds 64 KiB") {
            "limit"
        } else if error.classify() == serde_json::error::Category::Data {
            "schema"
        } else {
            "json-syntax"
        };
        return Err(format!("{class}: {source}: {error}"));
    }
    let dto: SnapshotDto =
        serde_json::from_str(content).map_err(|e| format!("schema: {source}: {e}"))?;
    if dto.format_version != 1 {
        return Err(format!(
            "unsupported-version: {source}: {}",
            dto.format_version
        ));
    }
    if dto.artifact_id.trim().is_empty() || dto.extractor_version.trim().is_empty() {
        return Err(format!("schema: {source}: non-empty metadata required"));
    }
    if dto.artifact_id.len() > MAX_STRING || dto.extractor_version.len() > MAX_STRING {
        return Err(format!("limit: {source}: string exceeds 64 KiB"));
    }
    if dto.observables.len() > MAX_ITEMS {
        return Err(format!("limit: {source}: too many observables"));
    }
    let mut observables = BTreeMap::new();
    for (key, value) in dto.observables {
        if key.trim().is_empty() {
            return Err(format!("schema: {source}: empty observable key"));
        }
        if key.len() > MAX_STRING {
            return Err(format!("limit: {source}: string exceeds 64 KiB"));
        }
        if value.state.len() > MAX_STRING
            || value.value.as_ref().is_some_and(|v| v.len() > MAX_STRING)
            || value.reason.as_ref().is_some_and(|r| r.len() > MAX_STRING)
        {
            return Err(format!("limit: {source}: string exceeds 64 KiB"));
        }
        let value = match (value.state.as_str(), value.value, value.reason) {
            ("known", Some(value), None) => ObservableValue::Known(value),
            ("absent", None, None) => ObservableValue::Absent,
            ("unknown", None, Some(reason_value)) => {
                ObservableValue::Unknown(reason(&reason_value, source)?)
            }
            _ => return Err(format!("schema: {source}: invalid observable state fields")),
        };
        observables.insert(key, value);
    }
    Ok(ArtifactFacts {
        artifact_id: dto.artifact_id,
        format_version: 1,
        extractor_version: dto.extractor_version,
        observables,
    })
}

pub fn load_contract(path: &Path) -> Result<RefinementContract, String> {
    let bytes = read_regular(path)?;
    load_contract_from_bytes(&bytes, &path.display().to_string())
}

pub fn load_contract_from_bytes(bytes: &[u8], source: &str) -> Result<RefinementContract, String> {
    let content = bounded(bytes, source)?;
    let value: toml::Value = toml::from_str(content).map_err(|e| {
        let text = e.to_string();
        let class = if text.contains("duplicate key") {
            "schema"
        } else {
            "toml-syntax"
        };
        format!("{class}: {source}: {e}")
    })?;
    ensure_toml_string_limits(&value, source)?;
    let dto: ContractDto = value
        .try_into()
        .map_err(|e| format!("schema: {source}: {e}"))?;
    if dto.id.trim().is_empty() || dto.relation.is_empty() {
        return Err(format!("schema: {source}: id and relation required"));
    }
    if dto.id.len() > MAX_STRING || dto.relation.len() > MAX_ITEMS {
        return Err(format!("limit: {source}: contract limit exceeded"));
    }
    let mut relations = Vec::with_capacity(dto.relation.len());
    let mut structural = HashSet::new();
    let mut pairs = HashSet::new();
    let mut protected = HashSet::new();
    let mut ordinary = HashSet::new();
    let mut accepted_total = 0usize;
    for r in dto.relation {
        if r.target.trim().is_empty() {
            return Err(format!("schema: {source}: target required"));
        }
        if r.target.len() > MAX_STRING || r.kind.len() > MAX_STRING {
            return Err(format!("limit: {source}: string exceeds 64 KiB"));
        }
        let compiled = match r.kind.as_str() {
            "preserve" => {
                if r.accepted_targets.is_some() {
                    return Err(format!(
                        "schema: {source}: preserve forbids accepted_targets"
                    ));
                }
                let s = r
                    .source
                    .filter(|v| !v.trim().is_empty())
                    .ok_or_else(|| format!("schema: {source}: preserve requires source"))?;
                if s.len() > MAX_STRING {
                    return Err(format!("limit: {source}: string exceeds 64 KiB"));
                }
                if !pairs.insert((s.clone(), r.target.clone())) {
                    return Err(format!("schema: {source}: conflicting relation"));
                }
                ordinary.insert(r.target.clone());
                RefinementRelation::Preserve {
                    source: s,
                    target: r.target,
                }
            }
            "may-normalize" => {
                let s = r
                    .source
                    .filter(|v| !v.trim().is_empty())
                    .ok_or_else(|| format!("schema: {source}: may-normalize requires source"))?;
                if s.len() > MAX_STRING {
                    return Err(format!("limit: {source}: string exceeds 64 KiB"));
                }
                let values = r
                    .accepted_targets
                    .ok_or_else(|| format!("schema: {source}: accepted_targets required"))?;
                if values.is_empty() {
                    return Err(format!("schema: {source}: accepted_targets required"));
                }
                accepted_total += values.len();
                if values.len() > MAX_ITEMS || accepted_total > MAX_ACCEPTED_TOTAL {
                    return Err(format!("limit: {source}: too many accepted_targets"));
                }
                let mut unique = HashSet::new();
                for v in &values {
                    if v.trim().is_empty() {
                        return Err(format!("schema: {source}: empty accepted target"));
                    }
                    if v.len() > MAX_STRING {
                        return Err(format!("limit: {source}: string exceeds 64 KiB"));
                    }
                    if !unique.insert(v.clone()) {
                        return Err(format!("schema: {source}: duplicate accepted target"));
                    }
                }
                if !pairs.insert((s.clone(), r.target.clone())) {
                    return Err(format!("schema: {source}: conflicting relation"));
                }
                ordinary.insert(r.target.clone());
                RefinementRelation::MayNormalize {
                    source: s,
                    target: r.target,
                    accepted_targets: values,
                }
            }
            "must-not-invent" => {
                if r.source.is_some() || r.accepted_targets.is_some() {
                    return Err(format!("schema: {source}: forbidden relation fields"));
                }
                protected.insert(r.target.clone());
                RefinementRelation::MustNotInvent { target: r.target }
            }
            _ => return Err(format!("schema: {source}: unsupported relation kind")),
        };
        if !structural.insert(format!("{compiled:?}")) {
            return Err(format!("schema: {source}: duplicate relation"));
        }
        relations.push(compiled);
    }
    if protected.iter().any(|t| ordinary.contains(t)) {
        return Err(format!("schema: {source}: conflicting target relations"));
    }
    Ok(RefinementContract {
        id: dto.id,
        relations,
    })
}

fn ensure_toml_string_limits(value: &toml::Value, source: &str) -> Result<(), String> {
    match value {
        toml::Value::String(value) if value.len() > MAX_STRING => {
            Err(format!("limit: {source}: string exceeds 64 KiB"))
        }
        toml::Value::Array(values) => {
            for value in values {
                ensure_toml_string_limits(value, source)?;
            }
            Ok(())
        }
        toml::Value::Table(table) => {
            for (key, value) in table {
                if key.len() > MAX_STRING {
                    return Err(format!("limit: {source}: string exceeds 64 KiB"));
                }
                ensure_toml_string_limits(value, source)?;
            }
            Ok(())
        }
        toml::Value::String(_) => Ok(()),
        toml::Value::Integer(_) => Ok(()),
        toml::Value::Float(_) => Ok(()),
        toml::Value::Boolean(_) => Ok(()),
        toml::Value::Datetime(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke() {
        let s =
            br#"{"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{}}"#;
        assert!(load_snapshot_from_bytes(s, "m").is_ok());
        let c = b"id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\n";
        assert!(load_contract_from_bytes(c, "m").is_ok());
    }

    #[test]
    fn oversized_observable_discriminants_are_limits() {
        for field in ["state", "reason", "prohibited-value"] {
            let oversized = "x".repeat(MAX_STRING + 1);
            let observable = match field {
                "state" => serde_json::json!({"state": oversized}),
                "reason" => serde_json::json!({"state": "unknown", "reason": oversized}),
                _ => serde_json::json!({"state": "absent", "value": oversized}),
            };
            let snapshot = serde_json::json!({
                "format_version": 1,
                "artifact_id": "a",
                "extractor_version": "e",
                "observables": {"x": observable}
            });
            let error = load_snapshot_from_bytes(snapshot.to_string().as_bytes(), "m").unwrap_err();
            assert!(error.starts_with("limit:"), "{field}: {error}");
        }
    }

    #[test]
    fn oversized_strings_precede_closed_schema_validation() {
        let oversized = "x".repeat(MAX_STRING + 1);
        let snapshot = serde_json::json!({
            "format_version": 1,
            "artifact_id": "a",
            "extractor_version": "e",
            "observables": {},
            "unknown": oversized
        });
        assert!(
            load_snapshot_from_bytes(snapshot.to_string().as_bytes(), "m")
                .unwrap_err()
                .starts_with("limit:")
        );

        let contract = format!(
            "id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\nsource='{oversized}'\n"
        );
        assert!(load_contract_from_bytes(contract.as_bytes(), "m")
            .unwrap_err()
            .starts_with("limit:"));
    }
}
