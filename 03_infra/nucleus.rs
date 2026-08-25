//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/nucleus-artifact.md
//! @prompt-hash 00000000
//! @layer L3

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::infra::prompt_io::{read_confined, without_meta_line};

const MAX_FILE: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NucleusLevel {
    Must,
    MustNot,
    May,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NucleusClaim {
    pub id: String,
    pub level: NucleusLevel,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NucleusDependency {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NucleusDocument {
    pub tekt: u32,
    pub kind: String,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub depends: Vec<NucleusDependency>,
    pub claims: Vec<NucleusClaim>,
}

fn valid_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn is_logical_nucleus_path(value: &str) -> bool {
    value.starts_with("00_nucleo/prompts/_nuclei/")
        && value.ends_with(".toml")
        && !value.ends_with(".tekt.toml")
        && !value.contains('\\')
        && !value.contains("//")
        && !value
            .split('/')
            .any(|part| part == "." || part == ".." || part.is_empty())
}

pub fn parse_nucleus(bytes: &[u8]) -> Result<NucleusDocument, String> {
    if bytes.len() > MAX_FILE {
        return Err("nucleus exceeds 1 MiB".into());
    }
    if bytes.contains(&0) {
        return Err("NUL is forbidden".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let document: NucleusDocument = toml::from_str(text).map_err(|e| e.to_string())?;
    if document.tekt != 1 || document.kind != "nucleus" {
        return Err("unsupported tekt version or kind".into());
    }
    if !valid_id(&document.id) || document.title.is_empty() || document.title.len() > 160 {
        return Err("invalid nucleus identity/title".into());
    }
    if document.claims.is_empty() || document.claims.len() > 1024 || document.depends.len() > 256 {
        return Err("invalid nucleus cardinality".into());
    }
    let mut claims = BTreeSet::new();
    for claim in &document.claims {
        if !valid_id(&claim.id)
            || !claims.insert(&claim.id)
            || claim.statement.is_empty()
            || claim.statement.len() > 2048
        {
            return Err("invalid or duplicate claim".into());
        }
    }
    let mut deps = BTreeSet::new();
    for dependency in &document.depends {
        if !is_logical_nucleus_path(&dependency.path)
            || !valid_hash(&dependency.sha256)
            || !deps.insert(&dependency.path)
        {
            return Err("invalid or duplicate nucleus dependency".into());
        }
    }
    Ok(document)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashDependency {
    pub path: String,
    pub digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptNucleusRef {
    pub path: String,
    pub sha256: String,
}

pub fn parse_prompt_nucleus_refs(bytes: &[u8]) -> Result<Vec<PromptNucleusRef>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let lines: Vec<&str> = text.lines().collect();
    let starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| (*line == "Núcleos Tekt:").then_some(i))
        .collect();
    if starts.len() > 1 {
        return Err("multiple Núcleos Tekt blocks".into());
    }
    let Some(start) = starts.first().copied() else {
        return Ok(Vec::new());
    };
    if lines[..start].iter().any(|line| line.starts_with("## ")) {
        return Err("Núcleos Tekt block must precede the first section".into());
    }
    let mut refs = Vec::new();
    for line in &lines[start + 1..] {
        let Some(value) = line.strip_prefix("- ") else {
            break;
        };
        let Some((path, hash)) = value.split_once(" sha256:") else {
            return Err("malformed nucleus reference".into());
        };
        if !is_logical_nucleus_path(path) || !valid_hash(hash) {
            return Err("invalid nucleus reference".into());
        }
        refs.push(PromptNucleusRef {
            path: path.into(),
            sha256: hash.into(),
        });
    }
    if refs.is_empty() {
        return Err("empty Núcleos Tekt block".into());
    }
    if refs
        .windows(2)
        .any(|window| window[0].path.as_bytes() >= window[1].path.as_bytes())
    {
        return Err("nucleus references must be unique and byte-sorted".into());
    }
    Ok(refs)
}

fn framed(mut hasher: Sha256, domain: &[u8], dependencies: &[HashDependency]) -> Sha256 {
    hasher.update([0]);
    hasher.update(domain);
    hasher.update([0]);
    let mut ordered = dependencies.to_vec();
    ordered.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    for dependency in ordered {
        hasher.update((dependency.path.len() as u64).to_be_bytes());
        hasher.update(dependency.path.as_bytes());
        hasher.update([0x20]);
        hasher.update(dependency.digest);
    }
    hasher
}

pub fn effective_nucleus_hash(bytes: &[u8], dependencies: &[HashDependency]) -> [u8; 32] {
    framed(
        Sha256::new_with_prefix(bytes),
        b"TEKT-NUCLEUS-DEPS-V1",
        dependencies,
    )
    .finalize()
    .into()
}

pub fn effective_prompt_hash(
    bytes: &[u8],
    dependencies: &[HashDependency],
) -> Result<String, String> {
    let cleaned = without_meta_line(bytes, b"Hash do C\xC3\xB3digo: ", false)?;
    let digest: [u8; 32] = if dependencies.is_empty() {
        Sha256::digest(cleaned).into()
    } else {
        framed(
            Sha256::new_with_prefix(cleaned),
            b"TEKT-PROMPT-NUCLEI-V1",
            dependencies,
        )
        .finalize()
        .into()
    };
    Ok(hex::encode(digest)[..8].to_owned())
}

fn load_effective(
    root: &Path,
    logical: &str,
    visiting: &mut BTreeSet<String>,
    cache: &mut BTreeMap<String, [u8; 32]>,
) -> Result<[u8; 32], String> {
    if let Some(digest) = cache.get(logical) {
        return Ok(*digest);
    }
    if !visiting.insert(logical.to_owned()) {
        return Err(format!("nucleus cycle at {logical}"));
    }
    if visiting.len() > 256 {
        return Err("nucleus dependency depth exceeds 256".into());
    }
    let bytes = read_confined(root, Path::new(logical), MAX_FILE)
        .map_err(|error| format!("{logical}: {error}"))?;
    let document = parse_nucleus(&bytes)?;
    let mut dependencies = Vec::new();
    for dependency in &document.depends {
        let digest = load_effective(root, &dependency.path, visiting, cache)?;
        if hex::encode(digest) != dependency.sha256 {
            return Err(format!("stale nucleus dependency pin {}", dependency.path));
        }
        dependencies.push(HashDependency {
            path: dependency.path.clone(),
            digest,
        });
    }
    visiting.remove(logical);
    let digest = effective_nucleus_hash(&bytes, &dependencies);
    cache.insert(logical.to_owned(), digest);
    Ok(digest)
}

pub fn effective_prompt_hash_at(root: &Path, prompt_path: &str) -> Result<String, String> {
    let bytes = read_confined(root, Path::new(prompt_path), 10 * 1024 * 1024)
        .map_err(|error| error.to_string())?;
    let refs = parse_prompt_nucleus_refs(&bytes)?;
    let mut cache = BTreeMap::new();
    let mut dependencies = Vec::new();
    for reference in refs {
        dependencies.push(HashDependency {
            path: reference.path.clone(),
            digest: load_effective(root, &reference.path, &mut BTreeSet::new(), &mut cache)?,
        });
    }
    effective_prompt_hash(&bytes, &dependencies)
}

pub fn refresh_prompt_nucleus_pins(
    root: &Path,
    bytes: &[u8],
) -> Result<(Vec<u8>, Vec<HashDependency>), String> {
    let references = parse_prompt_nucleus_refs(bytes)?;
    let mut text = std::str::from_utf8(bytes)
        .map_err(|error| error.to_string())?
        .to_owned();
    let mut cache = BTreeMap::new();
    let mut dependencies = Vec::new();
    for reference in references {
        let digest = load_effective(root, &reference.path, &mut BTreeSet::new(), &mut cache)?;
        let old = format!("- {} sha256:{}", reference.path, reference.sha256);
        let new = format!("- {} sha256:{}", reference.path, hex::encode(digest));
        if !text.contains(&old) {
            return Err(format!("nucleus reference disappeared: {}", reference.path));
        }
        text = text.replacen(&old, &new, 1);
        dependencies.push(HashDependency {
            path: reference.path,
            digest,
        });
    }
    Ok((text.into_bytes(), dependencies))
}

#[derive(Debug, Default)]
pub struct NucleusAudit {
    pub entries: Vec<crate::rules::nucleus_integrity::NucleusGraphEntry>,
    pub usages: Vec<crate::rules::nucleus_integrity::PromptNucleusUsage>,
    pub issues: Vec<(PathBuf, String)>,
}

pub fn audit_project(root: &Path) -> NucleusAudit {
    let mut audit = NucleusAudit::default();
    let prompts_root = root.join("00_nucleo/prompts");
    if !prompts_root.is_dir() {
        return audit;
    }
    let mut cache = BTreeMap::new();
    for result in walkdir::WalkDir::new(root.join("00_nucleo"))
        .follow_links(false)
        .into_iter()
    {
        let entry = match result {
            Ok(entry) => entry,
            Err(error) => {
                audit.issues.push((
                    error.path().unwrap_or(root).to_path_buf(),
                    format!("nucleus inventory walk failed: {error}"),
                ));
                continue;
            }
        };
        let path = entry.path();
        if entry.file_type().is_symlink()
            && matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("toml") | Some("tekt") | Some("md")
            )
        {
            audit.issues.push((
                path.to_path_buf(),
                "symlink forbidden in nucleus inventory".into(),
            ));
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let logical = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        match path.extension().and_then(|v| v.to_str()) {
            Some("tekt") => {
                audit.issues.push((
                    path.to_path_buf(),
                    "legacy .tekt nucleus extension is unsupported; use .toml".into(),
                ));
            }
            Some("toml") if logical.starts_with("00_nucleo/prompts/_nuclei/") => {
                if !is_logical_nucleus_path(&logical) {
                    audit.issues.push((
                        path.to_path_buf(),
                        "invalid nucleus .toml path in canonical namespace".into(),
                    ));
                    continue;
                }
                match std::fs::read(path)
                    .map_err(|e| e.to_string())
                    .and_then(|b| parse_nucleus(&b))
                {
                    Ok(document) => {
                        audit
                            .entries
                            .push(crate::rules::nucleus_integrity::NucleusGraphEntry {
                                path: logical,
                                dependencies: document
                                    .depends
                                    .into_iter()
                                    .map(|d| d.path)
                                    .collect(),
                            })
                    }
                    Err(reason) => audit.issues.push((path.to_path_buf(), reason)),
                }
            }
            Some("md") if path.starts_with(&prompts_root) => match std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| parse_prompt_nucleus_refs(&b))
            {
                Ok(refs) => {
                    for reference in refs {
                        audit
                            .usages
                            .push(crate::rules::nucleus_integrity::PromptNucleusUsage {
                                prompt: logical.clone(),
                                nucleus: reference.path.clone(),
                            });
                        match load_effective(
                            root,
                            &reference.path,
                            &mut BTreeSet::new(),
                            &mut cache,
                        ) {
                            Ok(digest) if hex::encode(digest) == reference.sha256 => {}
                            Ok(_) => audit.issues.push((
                                path.to_path_buf(),
                                format!("stale nucleus pin {}", reference.path),
                            )),
                            Err(reason) => audit.issues.push((path.to_path_buf(), reason)),
                        }
                    }
                }
                Err(reason) => audit.issues.push((path.to_path_buf(), reason)),
            },
            _ => {}
        }
    }
    if audit.entries.len() > 16_384 {
        audit
            .issues
            .push((prompts_root, "nucleus graph exceeds 16384 nodes".into()));
    }
    audit.entries.sort_by(|a, b| a.path.cmp(&b.path));
    audit
        .usages
        .sort_by(|a, b| a.prompt.cmp(&b.prompt).then(a.nucleus.cmp(&b.nucleus)));
    audit.issues.sort();
    audit
}
