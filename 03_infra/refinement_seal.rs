//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/segregated-materialization.md
//! @prompt-hash 4f6bc4f5
//! @layer L3
//! @updated 2026-08-24
//!
//! Manifest and receipt I/O for segregated refinement materialization.

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entities::refinement_seal::{OracleKind, VerdictName};

const MAX_ORACLES: usize = 256;

#[derive(Debug)]
pub struct Manifest {
    pub protocol_version: u32,
    pub prompt: PathBuf,
    pub prompt_sha256: String,
    pub baseline_oid: String,
    pub contract: PathBuf,
    pub contract_sha256: String,
    pub contract_producer: String,
    pub implementation_producer: String,
    pub verifier_producer: String,
    pub unknown_policy: String,
    pub oracles: Vec<Oracle>,
}

#[derive(Debug)]
pub struct Oracle {
    pub id: String,
    pub kind: OracleKind,
    pub before_ref: String,
    pub after_ref: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    protocol_version: u32,
    prompt: PathBuf,
    prompt_sha256: String,
    baseline_oid: String,
    contract: PathBuf,
    contract_sha256: String,
    contract_producer: String,
    implementation_producer: String,
    verifier_producer: String,
    unknown_policy: String,
    oracle: Vec<OracleDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OracleDto {
    id: String,
    kind: String,
    before_ref: String,
    after_ref: String,
}

#[derive(Serialize)]
pub struct SealReceipt {
    pub id: String,
    pub kind: &'static str,
    pub before_oid: String,
    pub after_oid: String,
    pub verdict: &'static str,
}

#[derive(Serialize)]
pub struct Seal<'a> {
    pub protocol_version: u32,
    pub manifest_sha256: &'a str,
    pub prompt_sha256: &'a str,
    pub contract_sha256: &'a str,
    pub baseline_oid: &'a str,
    pub contract_producer: &'a str,
    pub implementation_producer: &'a str,
    pub verifier_producer: &'a str,
    pub receipts: &'a [SealReceipt],
    pub counts: SealCounts,
    pub mutation_score: &'static str,
    pub sealed: bool,
}

#[derive(Clone, Copy, Serialize)]
pub struct SealCounts {
    pub positive: usize,
    pub negative: usize,
    pub unknown: usize,
}

#[derive(Serialize)]
struct CanonicalManifest<'a> {
    protocol_version: u32,
    prompt: &'a Path,
    prompt_sha256: &'a str,
    baseline_oid: &'a str,
    contract: &'a Path,
    contract_sha256: &'a str,
    contract_producer: &'a str,
    implementation_producer: &'a str,
    verifier_producer: &'a str,
    unknown_policy: &'a str,
    oracle: Vec<CanonicalOracle<'a>>,
}

#[derive(Serialize)]
struct CanonicalOracle<'a> {
    id: &'a str,
    kind: &'static str,
    before_ref: &'a str,
    after_ref: &'a str,
}

pub fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub fn confined_read(root: &Path, relative_path: &Path) -> Result<Vec<u8>, String> {
    if !relative(relative_path) {
        return Err(format!(
            "unsafe repository-relative path `{}`",
            relative_path.display()
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve repository root: {e}"))?;
    let path = root.join(relative_path).canonicalize().map_err(|e| {
        format!(
            "cannot resolve repository input {}: {e}",
            relative_path.display()
        )
    })?;
    if !path.starts_with(&root) {
        return Err(format!(
            "repository input escapes root: {}",
            relative_path.display()
        ));
    }
    fs::read(&path).map_err(|e| {
        format!(
            "cannot read repository input {}: {e}",
            relative_path.display()
        )
    })
}

pub fn read_manifest(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| {
        format!(
            "cannot read refinement seal manifest {}: {e}",
            path.display()
        )
    })
}

pub fn load_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let dto: ManifestDto =
        toml::from_str(std::str::from_utf8(bytes).map_err(|_| "manifest is not UTF-8")?)
            .map_err(|e| format!("invalid refinement seal manifest: {e}"))?;
    if dto.protocol_version != 1 {
        return Err(format!(
            "unsupported protocol_version {}; expected 1",
            dto.protocol_version
        ));
    }
    if dto.unknown_policy != "block" {
        return Err("unknown_policy must be `block`".to_string());
    }
    for (name, value) in [
        ("contract_producer", &dto.contract_producer),
        ("implementation_producer", &dto.implementation_producer),
        ("verifier_producer", &dto.verifier_producer),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{name} must not be empty"));
        }
    }
    let producers: HashSet<&str> = [
        dto.contract_producer.as_str(),
        dto.implementation_producer.as_str(),
        dto.verifier_producer.as_str(),
    ]
    .into_iter()
    .collect();
    if producers.len() != 3 {
        return Err("refinement seal producers must be distinct".to_string());
    }
    if !relative(&dto.prompt) || !relative(&dto.contract) {
        return Err("prompt and contract paths must be safe repository-relative paths".to_string());
    }
    if !valid_hash(&dto.prompt_sha256) || !valid_hash(&dto.contract_sha256) {
        return Err("SHA-256 values must be 64 lowercase hexadecimal characters".to_string());
    }
    if dto.oracle.is_empty() {
        return Err("manifest requires at least one [[oracle]]".to_string());
    }
    if dto.oracle.len() > MAX_ORACLES {
        return Err(format!(
            "oracle budget exceeded: {} > {MAX_ORACLES}",
            dto.oracle.len()
        ));
    }
    let mut ids = HashSet::new();
    let mut oracles = Vec::with_capacity(dto.oracle.len());
    for oracle in dto.oracle {
        if oracle.id.trim().is_empty() {
            return Err("oracle id must not be empty".to_string());
        }
        if !ids.insert(oracle.id.clone()) {
            return Err(format!("duplicate oracle id `{}`", oracle.id));
        }
        if oracle.before_ref.trim().is_empty() || oracle.after_ref.trim().is_empty() {
            return Err(format!(
                "oracle `{}` requires before_ref and after_ref",
                oracle.id
            ));
        }
        let kind = match oracle.kind.as_str() {
            "positive" => OracleKind::Positive,
            "negative" => OracleKind::Negative,
            "unknown" => OracleKind::Unknown,
            other => return Err(format!("unsupported oracle kind `{other}`")),
        };
        oracles.push(Oracle {
            id: oracle.id,
            kind,
            before_ref: oracle.before_ref,
            after_ref: oracle.after_ref,
        });
    }
    for required in [
        OracleKind::Positive,
        OracleKind::Negative,
        OracleKind::Unknown,
    ] {
        if !oracles.iter().any(|oracle| oracle.kind == required) {
            return Err(format!(
                "manifest requires at least one {} oracle",
                required.as_str()
            ));
        }
    }
    Ok(Manifest {
        protocol_version: dto.protocol_version,
        prompt: dto.prompt,
        prompt_sha256: dto.prompt_sha256,
        baseline_oid: dto.baseline_oid,
        contract: dto.contract,
        contract_sha256: dto.contract_sha256,
        contract_producer: dto.contract_producer,
        implementation_producer: dto.implementation_producer,
        verifier_producer: dto.verifier_producer,
        unknown_policy: dto.unknown_policy,
        oracles,
    })
}

pub fn semantic_manifest_sha256(manifest: &Manifest) -> Result<String, String> {
    let mut oracles: Vec<&Oracle> = manifest.oracles.iter().collect();
    oracles.sort_by(|left, right| left.id.cmp(&right.id));
    let canonical = CanonicalManifest {
        protocol_version: manifest.protocol_version,
        prompt: &manifest.prompt,
        prompt_sha256: &manifest.prompt_sha256,
        baseline_oid: &manifest.baseline_oid,
        contract: &manifest.contract,
        contract_sha256: &manifest.contract_sha256,
        contract_producer: &manifest.contract_producer,
        implementation_producer: &manifest.implementation_producer,
        verifier_producer: &manifest.verifier_producer,
        unknown_policy: &manifest.unknown_policy,
        oracle: oracles
            .into_iter()
            .map(|oracle| CanonicalOracle {
                id: &oracle.id,
                kind: oracle.kind.as_str(),
                before_ref: &oracle.before_ref,
                after_ref: &oracle.after_ref,
            })
            .collect(),
    };
    serde_json::to_vec(&canonical)
        .map(|bytes| sha256(&bytes))
        .map_err(|e| format!("cannot canonicalize refinement seal manifest: {e}"))
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

pub fn serialize_seal(seal: &Seal<'_>) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(seal)
        .map(|mut bytes| {
            bytes.push(b'\n');
            bytes
        })
        .map_err(|e| format!("cannot serialize refinement seal: {e}"))
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| format!("invalid output path {}", path.display()))?;
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("cannot create temporary seal: {e}"))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("cannot write temporary seal: {e}"))?;
        fs::rename(&temporary, path)
            .map_err(|e| format!("cannot publish seal {}: {e}", path.display()))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn receipt(
    id: String,
    kind: OracleKind,
    before_oid: String,
    after_oid: String,
    verdict: VerdictName,
) -> SealReceipt {
    SealReceipt {
        id,
        kind: kind.as_str(),
        before_oid,
        after_oid,
        verdict: verdict.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_manifest_hash_ignores_toml_and_oracle_order() {
        let header = r#"
protocol_version = 1
prompt = "prompt.md"
prompt_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
baseline_oid = "1111111111111111111111111111111111111111"
contract = "contract.toml"
contract_sha256 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
contract_producer = "contract"
implementation_producer = "implementation"
verifier_producer = "verifier"
unknown_policy = "block"
"#;
        let first = format!(
            "{header}\n[[oracle]]\nid = \"b\"\nkind = \"negative\"\nbefore_ref = \"1\"\nafter_ref = \"2\"\n\n[[oracle]]\nid = \"a\"\nkind = \"positive\"\nbefore_ref = \"3\"\nafter_ref = \"4\"\n\n[[oracle]]\nid = \"c\"\nkind = \"unknown\"\nbefore_ref = \"5\"\nafter_ref = \"6\"\n"
        );
        let second = format!(
            "{header}\n[[oracle]]\nafter_ref = \"6\"\nbefore_ref = \"5\"\nkind = \"unknown\"\nid = \"c\"\n\n[[oracle]]\nafter_ref = \"4\"\nbefore_ref = \"3\"\nkind = \"positive\"\nid = \"a\"\n\n[[oracle]]\nafter_ref = \"2\"\nbefore_ref = \"1\"\nkind = \"negative\"\nid = \"b\"\n"
        );
        let first = load_manifest(first.as_bytes()).unwrap();
        let second = load_manifest(second.as_bytes()).unwrap();
        assert_eq!(
            semantic_manifest_sha256(&first).unwrap(),
            semantic_manifest_sha256(&second).unwrap()
        );
    }

    #[test]
    fn manifest_requires_every_oracle_category() {
        let incomplete = r#"
protocol_version = 1
prompt = "prompt.md"
prompt_sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
baseline_oid = "1111111111111111111111111111111111111111"
contract = "contract.toml"
contract_sha256 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
contract_producer = "contract"
implementation_producer = "implementation"
verifier_producer = "verifier"
unknown_policy = "block"
[[oracle]]
id = "positive"
kind = "positive"
before_ref = "1"
after_ref = "2"
[[oracle]]
id = "negative"
kind = "negative"
before_ref = "1"
after_ref = "2"
"#;
        assert!(load_manifest(incomplete.as_bytes())
            .unwrap_err()
            .contains("unknown oracle"));
    }

    #[test]
    fn seal_serializes_nested_counts() {
        let seal = Seal {
            protocol_version: 1,
            manifest_sha256: "m",
            prompt_sha256: "p",
            contract_sha256: "c",
            baseline_oid: "b",
            contract_producer: "one",
            implementation_producer: "two",
            verifier_producer: "three",
            receipts: &[],
            counts: SealCounts {
                positive: 1,
                negative: 2,
                unknown: 3,
            },
            mutation_score: "1.0",
            sealed: true,
        };
        let json: serde_json::Value =
            serde_json::from_slice(&serialize_seal(&seal).unwrap()).unwrap();
        assert_eq!(json["counts"]["positive"], 1);
        assert!(json.get("positive_count").is_none());
    }
}
