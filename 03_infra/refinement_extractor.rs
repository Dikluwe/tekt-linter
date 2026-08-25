//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/refinement-extractor.md
//! @prompt-hash 3f2b944e
//! @layer L3
//! @updated 2026-08-24

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tree_sitter::{Parser, Query, QueryCursor};

use crate::entities::refinement::{
    observable_from_captures, ArtifactFacts, CaptureCardinality, MissingPolicy, ObservableValue,
    UnknownReason,
};

pub const EXTRACTOR_VERSION: &str = "crystalline-rust-query-v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservableSpec {
    pub key: String,
    pub file: PathBuf,
    pub query: String,
    pub capture: String,
    pub cardinality: CaptureCardinality,
    pub on_missing: MissingPolicy,
}

#[derive(Deserialize)]
struct ExtractionContractDto {
    #[serde(default)]
    observable: Vec<ObservableDto>,
}

#[derive(Deserialize)]
struct ObservableDto {
    key: String,
    language: String,
    file: PathBuf,
    query: String,
    capture: String,
    cardinality: String,
    on_missing: String,
}

#[derive(Serialize)]
struct SnapshotOut<'a> {
    format_version: u32,
    artifact_id: &'a str,
    extractor_version: &'a str,
    observables: BTreeMap<&'a str, ObservableOut<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
enum ObservableOut<'a> {
    Known { value: &'a str },
    Absent,
    Unknown { reason: &'a str },
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

pub fn load_observable_specs(path: &Path) -> Result<Vec<ObservableSpec>, String> {
    let content = fs::read_to_string(path).map_err(|error| {
        format!(
            "cannot read refinement contract {}: {error}",
            path.display()
        )
    })?;
    load_observable_specs_from_bytes(content.as_bytes(), &path.display().to_string())
}

pub fn load_observable_specs_from_bytes(
    content: &[u8],
    source: &str,
) -> Result<Vec<ObservableSpec>, String> {
    let content = std::str::from_utf8(content)
        .map_err(|_| format!("refinement contract {source} is not UTF-8"))?;
    let dto: ExtractionContractDto = toml::from_str(content)
        .map_err(|error| format!("invalid refinement contract {source}: {error}"))?;
    if dto.observable.is_empty() {
        return Err(format!(
            "refinement contract {} requires at least one [[observable]] for snapshot",
            source
        ));
    }
    let mut keys = HashSet::new();
    dto.observable
        .into_iter()
        .map(|observable| {
            if observable.language != "rust" {
                return Err(format!(
                    "observable `{}` uses unsupported language `{}`; expected rust",
                    observable.key, observable.language
                ));
            }
            if observable.key.trim().is_empty()
                || observable.capture.trim().is_empty()
                || observable.query.trim().is_empty()
            {
                return Err("observable requires non-empty key, query and capture".to_string());
            }
            if !keys.insert(observable.key.clone()) {
                return Err(format!("duplicate observable key `{}`", observable.key));
            }
            if !safe_relative_path(&observable.file) {
                return Err(format!(
                    "observable `{}` has unsafe file path `{}`",
                    observable.key,
                    observable.file.display()
                ));
            }
            let cardinality = match observable.cardinality.as_str() {
                "one" => CaptureCardinality::One,
                "many" => CaptureCardinality::Many,
                other => return Err(format!("unsupported cardinality `{other}`")),
            };
            let on_missing = match observable.on_missing.as_str() {
                "unknown" => MissingPolicy::Unknown,
                "absent" => MissingPolicy::Absent,
                other => return Err(format!("unsupported on_missing policy `{other}`")),
            };
            Ok(ObservableSpec {
                key: observable.key,
                file: observable.file,
                query: observable.query,
                capture: observable.capture,
                cardinality,
                on_missing,
            })
        })
        .collect()
}

fn normalize_syntax(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_source(source: &str, spec: &ObservableSpec) -> Result<ObservableValue, String> {
    let language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|error| format!("cannot initialize Rust parser: {error}"))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| format!("Rust parser returned no tree for {}", spec.file.display()))?;
    if tree.root_node().has_error() {
        return Ok(ObservableValue::Unknown(UnknownReason::OpaqueConstruction));
    }
    let query = Query::new(&language, &spec.query)
        .map_err(|error| format!("invalid query for observable `{}`: {error}", spec.key))?;
    let capture_index = query
        .capture_names()
        .iter()
        .position(|name| *name == spec.capture)
        .ok_or_else(|| {
            format!(
                "query for observable `{}` does not declare capture `@{}`",
                spec.key, spec.capture
            )
        })? as u32;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    let mut captures = Vec::new();
    while let Some(query_match) = matches.next() {
        for capture in query_match.captures {
            if capture.index == capture_index {
                let text = &source[capture.node.byte_range()];
                captures.push(normalize_syntax(text));
            }
        }
    }
    Ok(observable_from_captures(
        captures,
        spec.cardinality,
        spec.on_missing,
    ))
}

fn extract_one(root: &Path, spec: &ObservableSpec) -> Result<ObservableValue, String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve project root {}: {error}", root.display()))?;
    let requested = root.join(&spec.file);
    let path = match requested.canonicalize() {
        Ok(path) => path,
        Err(_) => return Ok(ObservableValue::Unknown(UnknownReason::MissingObservable)),
    };
    if !path.starts_with(&canonical_root) {
        return Err(format!(
            "observable `{}` resolves outside project root: {}",
            spec.key,
            path.display()
        ));
    }
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(_) => return Ok(ObservableValue::Unknown(UnknownReason::MissingObservable)),
    };
    extract_source(&source, spec)
}

pub fn extract_snapshot_from_content<F>(
    artifact_id: &str,
    specs: &[ObservableSpec],
    mut read: F,
) -> Result<ArtifactFacts, String>
where
    F: FnMut(&Path) -> Result<Option<Vec<u8>>, String>,
{
    if artifact_id.trim().is_empty() {
        return Err("artifact-id must not be empty".to_string());
    }
    let mut observables = BTreeMap::new();
    for spec in specs {
        let value = match read(&spec.file)? {
            None => ObservableValue::Unknown(UnknownReason::MissingObservable),
            Some(bytes) => match std::str::from_utf8(&bytes) {
                Ok(source) => extract_source(source, spec)?,
                Err(_) => ObservableValue::Unknown(UnknownReason::OpaqueConstruction),
            },
        };
        observables.insert(spec.key.clone(), value);
    }
    Ok(ArtifactFacts {
        artifact_id: artifact_id.to_string(),
        format_version: 1,
        extractor_version: EXTRACTOR_VERSION.to_string(),
        observables,
    })
}

pub fn extract_snapshot(
    root: &Path,
    artifact_id: &str,
    specs: &[ObservableSpec],
) -> Result<ArtifactFacts, String> {
    if artifact_id.trim().is_empty() {
        return Err("artifact-id must not be empty".to_string());
    }
    let mut observables = BTreeMap::new();
    for spec in specs {
        observables.insert(spec.key.clone(), extract_one(root, spec)?);
    }
    Ok(ArtifactFacts {
        artifact_id: artifact_id.to_string(),
        format_version: 1,
        extractor_version: EXTRACTOR_VERSION.to_string(),
        observables,
    })
}

fn reason_text(reason: &UnknownReason) -> &'static str {
    match reason {
        UnknownReason::MissingObservable => "missing-observable",
        UnknownReason::AmbiguousIdentity => "ambiguous-identity",
        UnknownReason::UnsupportedParser => "unsupported-parser",
        UnknownReason::OpaqueConstruction => "opaque-construction",
        UnknownReason::PartialContract => "partial-contract",
        UnknownReason::BudgetExhausted => "budget-exhausted",
    }
}

pub fn serialize_snapshot(snapshot: &ArtifactFacts) -> Result<String, String> {
    let observables = snapshot
        .observables
        .iter()
        .map(|(key, value)| {
            let value = match value {
                ObservableValue::Known(value) => ObservableOut::Known { value },
                ObservableValue::Absent => ObservableOut::Absent,
                ObservableValue::Unknown(reason) => ObservableOut::Unknown {
                    reason: reason_text(reason),
                },
            };
            (key.as_str(), value)
        })
        .collect();
    serde_json::to_string_pretty(&SnapshotOut {
        format_version: snapshot.format_version,
        artifact_id: &snapshot.artifact_id,
        extractor_version: &snapshot.extractor_version,
        observables,
    })
    .map(|json| format!("{json}\n"))
    .map_err(|error| format!("cannot serialize refinement snapshot: {error}"))
}

pub fn write_snapshot_atomic(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("invalid output path {}", path.display()))?;
    let temporary = parent.join(format!(".{file_name}.tmp"));
    fs::write(&temporary, content)
        .map_err(|error| format!("cannot write temporary snapshot: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| {
        format!(
            "cannot publish snapshot {} from {}: {error}",
            path.display(),
            temporary.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(
        query: &str,
        cardinality: CaptureCardinality,
        missing: MissingPolicy,
    ) -> ObservableSpec {
        ObservableSpec {
            key: "functions".to_string(),
            file: PathBuf::from("sample.rs"),
            query: query.to_string(),
            capture: "value".to_string(),
            cardinality,
            on_missing: missing,
        }
    }

    #[test]
    fn extracts_one_many_absent_unknown_ambiguous_and_opaque() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("sample.rs"), "fn beta() {} fn alpha() {}").unwrap();
        let query = "(function_item name: (identifier) @value)";
        let many = extract_one(
            dir.path(),
            &spec(query, CaptureCardinality::Many, MissingPolicy::Unknown),
        )
        .unwrap();
        assert_eq!(
            many,
            ObservableValue::Known("[\"alpha\",\"beta\"]".to_string())
        );
        assert_eq!(
            extract_one(
                dir.path(),
                &spec(query, CaptureCardinality::One, MissingPolicy::Unknown)
            )
            .unwrap(),
            ObservableValue::Unknown(UnknownReason::AmbiguousIdentity)
        );
        let no_match = "(struct_item name: (type_identifier) @value)";
        assert_eq!(
            extract_one(
                dir.path(),
                &spec(no_match, CaptureCardinality::One, MissingPolicy::Absent)
            )
            .unwrap(),
            ObservableValue::Absent
        );
        assert_eq!(
            extract_one(
                dir.path(),
                &spec(no_match, CaptureCardinality::One, MissingPolicy::Unknown)
            )
            .unwrap(),
            ObservableValue::Unknown(UnknownReason::MissingObservable)
        );
        fs::write(dir.path().join("sample.rs"), "fn broken(").unwrap();
        assert_eq!(
            extract_one(
                dir.path(),
                &spec(query, CaptureCardinality::One, MissingPolicy::Unknown)
            )
            .unwrap(),
            ObservableValue::Unknown(UnknownReason::OpaqueConstruction)
        );
    }

    #[test]
    fn serialization_is_byte_deterministic() {
        let snapshot = ArtifactFacts {
            artifact_id: "self".to_string(),
            format_version: 1,
            extractor_version: EXTRACTOR_VERSION.to_string(),
            observables: BTreeMap::from([
                ("b".to_string(), ObservableValue::Absent),
                ("a".to_string(), ObservableValue::Known("x".to_string())),
            ]),
        };
        assert_eq!(serialize_snapshot(&snapshot), serialize_snapshot(&snapshot));
    }

    #[test]
    fn rejects_parent_path() {
        let dir = tempfile::tempdir().unwrap();
        let contract = dir.path().join("contract.toml");
        fs::write(
            &contract,
            "[[observable]]\nkey='x'\nlanguage='rust'\nfile='../escape.rs'\nquery='(identifier) @value'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n",
        )
        .unwrap();
        assert!(load_observable_specs(&contract)
            .unwrap_err()
            .contains("unsafe"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        fs::write(outside.path(), "fn escaped() {}").unwrap();
        symlink(outside.path(), root.path().join("sample.rs")).unwrap();
        let error = extract_one(
            root.path(),
            &spec(
                "(function_item name: (identifier) @value)",
                CaptureCardinality::One,
                MissingPolicy::Unknown,
            ),
        )
        .unwrap_err();
        assert!(error.contains("outside project root"));
    }
}
