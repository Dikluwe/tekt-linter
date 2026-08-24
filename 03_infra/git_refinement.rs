//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @prompt-hash cc8920e0
//! @layer L3
//! @updated 2026-08-24

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::entities::refinement::{ArtifactFacts, ObservableValue, UnknownReason};
use crate::infra::refinement_extractor::{extract_snapshot_from_content, ObservableSpec};

const MAX_PATHS: usize = 512;
const MAX_BLOB_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;
const GIT_TIMEOUT: Duration = Duration::from_secs(10);
const BLOB_BUDGET_ERROR: &str = "refinement Git blob budget exhausted";

#[derive(Clone, Debug)]
enum TreeEntry {
    Missing,
    Blob(String),
    Unsupported,
}

fn git_command(repository: &Path) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository)
        .args([
            "-c",
            "protocol.allow=never",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_PAGER", "cat");
    command
}

fn checked(output: Output, operation: &str) -> Result<Vec<u8>, String> {
    if output.status.success() {
        return Ok(output.stdout);
    }
    let detail = String::from_utf8_lossy(&output.stderr);
    Err(format!("git {operation} failed: {}", detail.trim()))
}

fn collect_output(child: Child, operation: &str) -> Result<Output, String> {
    collect_output_with_timeout(child, operation, GIT_TIMEOUT)
}

fn collect_output_with_timeout(
    mut child: Child,
    operation: &str,
    timeout: Duration,
) -> Result<Output, String> {
    let mut stdout = child.stdout.take().ok_or("cannot capture git stdout")?;
    let mut stderr = child.stderr.take().ok_or("cannot capture git stderr")?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let started = Instant::now();
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("git {operation} exceeded 10 second budget"));
            }
            Err(error) => return Err(format!("cannot wait for git {operation}: {error}")),
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "git stdout reader panicked".to_string())?
        .map_err(|error| format!("cannot read git stdout: {error}"))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "git stderr reader panicked".to_string())?
        .map_err(|error| format!("cannot read git stderr: {error}"))?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn run(mut command: Command, operation: &str) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command
        .spawn()
        .map_err(|error| format!("cannot start git {operation}: {error}"))?;
    collect_output(child, operation)
}

pub fn resolve_commit(repository: &Path, reference: &str) -> Result<String, String> {
    if reference.contains('\0') || reference.trim().is_empty() {
        return Err("Git ref must not be empty or contain NUL".to_string());
    }
    let expression = format!("{reference}^{{commit}}");
    let mut command = git_command(repository);
    command
        .args(["rev-parse", "--verify", "--end-of-options"])
        .arg(expression);
    let output = run(command, "rev-parse")?;
    let oid = String::from_utf8(checked(output, "rev-parse")?)
        .map_err(|_| "git returned a non-UTF-8 object ID".to_string())?;
    let oid = oid.trim();
    if oid.is_empty() || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("git returned an invalid object ID".to_string());
    }
    Ok(oid.to_ascii_lowercase())
}

fn tree_entries(
    repository: &Path,
    oid: &str,
    specs: &[ObservableSpec],
) -> Result<BTreeMap<PathBuf, TreeEntry>, String> {
    let paths: BTreeSet<&Path> = specs.iter().map(|spec| spec.file.as_path()).collect();
    if paths.len() > MAX_PATHS {
        return Err(format!(
            "refinement path budget exceeded: {} > {MAX_PATHS}",
            paths.len()
        ));
    }
    let mut command = git_command(repository);
    command.args(["ls-tree", "-rz", oid, "--"]);
    for path in &paths {
        command.arg(path);
    }
    let output = checked(run(command, "ls-tree")?, "ls-tree")?;
    let mut entries: BTreeMap<PathBuf, TreeEntry> = paths
        .into_iter()
        .map(|path| (path.to_path_buf(), TreeEntry::Missing))
        .collect();
    for record in output
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or("invalid git ls-tree framing")?;
        let metadata =
            std::str::from_utf8(&record[..tab]).map_err(|_| "non-UTF-8 ls-tree metadata")?;
        let path =
            std::str::from_utf8(&record[tab + 1..]).map_err(|_| "non-UTF-8 observable path")?;
        let fields: Vec<&str> = metadata.split_whitespace().collect();
        if fields.len() != 3 {
            return Err("invalid git ls-tree metadata".to_string());
        }
        let value = match (fields[0], fields[1]) {
            ("100644" | "100755", "blob") => TreeEntry::Blob(fields[2].to_string()),
            ("120000", "blob") | ("160000", "commit") => TreeEntry::Unsupported,
            _ => return Err(format!("unsupported Git tree entry for `{path}`")),
        };
        entries.insert(PathBuf::from(path), value);
    }
    Ok(entries)
}

fn read_blobs(
    repository: &Path,
    oids: &BTreeSet<String>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if oids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut child = git_command(repository)
        .args(["cat-file", "--batch-command", "--buffer"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot start git cat-file: {error}"))?;
    {
        let input = child
            .stdin
            .as_mut()
            .ok_or("cannot open git cat-file stdin")?;
        for oid in oids {
            writeln!(input, "contents {oid}")
                .map_err(|error| format!("cannot write git protocol: {error}"))?;
        }
        writeln!(input, "flush").map_err(|error| format!("cannot flush git protocol: {error}"))?;
    }
    drop(child.stdin.take());
    let output = collect_output(child, "cat-file")?;
    let bytes = checked(output, "cat-file")?;
    let mut cursor = 0usize;
    let mut total = 0usize;
    let mut blobs = BTreeMap::new();
    for expected in oids {
        let newline = bytes[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
            .ok_or("truncated git cat-file header")?;
        let header = std::str::from_utf8(&bytes[cursor..newline])
            .map_err(|_| "non-UTF-8 git cat-file header")?;
        let fields: Vec<&str> = header.split_whitespace().collect();
        if fields.len() != 3 || fields[0] != expected || fields[1] != "blob" {
            return Err(format!("unexpected git cat-file response `{header}`"));
        }
        let size: usize = fields[2].parse().map_err(|_| "invalid git blob size")?;
        if size > MAX_BLOB_BYTES || total.saturating_add(size) > MAX_TOTAL_BYTES {
            return Err(BLOB_BUDGET_ERROR.to_string());
        }
        let start = newline + 1;
        let end = start.checked_add(size).ok_or("git blob size overflow")?;
        if end >= bytes.len() || bytes[end] != b'\n' {
            return Err("truncated git blob response".to_string());
        }
        blobs.insert(expected.clone(), bytes[start..end].to_vec());
        total += size;
        cursor = end + 1;
    }
    Ok(blobs)
}

pub fn extract_revision_snapshot(
    repository: &Path,
    oid: &str,
    specs: &[ObservableSpec],
) -> Result<ArtifactFacts, String> {
    let entries = tree_entries(repository, oid, specs)?;
    let oids: BTreeSet<String> = entries
        .values()
        .filter_map(|entry| match entry {
            TreeEntry::Blob(oid) => Some(oid.clone()),
            TreeEntry::Missing | TreeEntry::Unsupported => None,
        })
        .collect();
    let blobs = match read_blobs(repository, &oids) {
        Ok(blobs) => blobs,
        Err(error) if error == BLOB_BUDGET_ERROR => {
            let mut snapshot = extract_snapshot_from_content(oid, specs, |_| Ok(None))?;
            for spec in specs {
                if matches!(entries.get(&spec.file), Some(TreeEntry::Blob(_))) {
                    snapshot.observables.insert(
                        spec.key.clone(),
                        ObservableValue::Unknown(UnknownReason::BudgetExhausted),
                    );
                }
            }
            return Ok(snapshot);
        }
        Err(error) => return Err(error),
    };
    let mut snapshot = extract_snapshot_from_content(oid, specs, |path| {
        Ok(entries
            .get(path)
            .and_then(|entry| match entry {
                TreeEntry::Blob(oid) => Some(oid),
                TreeEntry::Missing | TreeEntry::Unsupported => None,
            })
            .and_then(|blob| blobs.get(blob))
            .cloned())
    })?;
    for spec in specs {
        if matches!(entries.get(&spec.file), Some(TreeEntry::Unsupported)) {
            snapshot.observables.insert(
                spec.key.clone(),
                ObservableValue::Unknown(UnknownReason::UnsupportedParser),
            );
        }
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn timeout_kills_a_blocked_process() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 5"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = command.spawn().unwrap();
        let started = Instant::now();
        let error =
            collect_output_with_timeout(child, "fixture", Duration::from_millis(30)).unwrap_err();
        assert!(error.contains("exceeded"));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn option_like_ref_is_not_treated_as_an_option() {
        let error = resolve_commit(Path::new("."), "--help").unwrap_err();
        assert!(error.contains("rev-parse failed"));
    }
}
