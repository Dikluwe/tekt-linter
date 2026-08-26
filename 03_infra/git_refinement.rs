//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/git-refinement.md
//! @prompt-hash b1cf6082
//! @layer L3
//! @updated 2026-08-24

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::entities::refinement::{ArtifactFacts, ObservableValue, UnknownReason};
use crate::infra::refinement_extractor::{extract_snapshot_from_content, ObservableSpec};

const MAX_PATHS: usize = 512;
const MAX_BLOB_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 32 * 1024 * 1024;
const GIT_TIMEOUT: Duration = Duration::from_secs(10);
const BLOB_BUDGET_ERROR: &str = "refinement Git blob budget exhausted";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRevisionContent {
    pub oid: String,
    pub paths: BTreeMap<PathBuf, GitPathContent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitPathContent {
    Blob(Vec<u8>),
    Missing,
    Unknown(GitUnknownReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitUnknownReason {
    MissingObject,
    ForbiddenObjectKind,
    BudgetExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitRevisionError {
    InvalidInput,
    MissingRef,
    InvalidFraming,
    Timeout,
    ProcessFailure,
    ContainmentFailure,
}

const GIT_PREFIX: [&str; 16] = [
    "-c",
    "protocol.allow=never",
    "-c",
    "core.hooksPath=",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "credential.helper=",
    "-c",
    "diff.external=",
    "-c",
    "filter.lfs.process=",
    "-c",
    "filter.lfs.smudge=",
    "-c",
    "filter.lfs.clean=",
];

fn valid_oid(bytes: &[u8]) -> bool {
    matches!(bytes.len(), 40 | 64)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn controlled_command(executable: &Path, repository: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(repository)
        .env_clear()
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("LC_ALL", "C")
        .args(GIT_PREFIX);
    command
}

#[cfg(unix)]
fn isolate(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn isolate(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_group(child: &mut Child) -> Result<(), GitRevisionError> {
    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    const SIGKILL: i32 = 9;
    let pid = i32::try_from(child.id()).map_err(|_| GitRevisionError::ContainmentFailure)?;
    // SAFETY: `pid` is the positive PID returned for our own child. The child was
    // placed in a fresh process group whose id is that PID before it executed.
    let result = unsafe { kill(-pid, SIGKILL) };
    if result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(3) {
        Ok(())
    } else {
        Err(GitRevisionError::ContainmentFailure)
    }
}

#[cfg(unix)]
fn group_is_alive(child: &Child) -> Result<bool, GitRevisionError> {
    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    let pid = i32::try_from(child.id()).map_err(|_| GitRevisionError::ContainmentFailure)?;
    // SAFETY: signal zero only probes the process group created for this child.
    let result = unsafe { kill(-pid, 0) };
    if result == 0 {
        Ok(true)
    } else if std::io::Error::last_os_error().raw_os_error() == Some(3) {
        Ok(false)
    } else {
        Err(GitRevisionError::ContainmentFailure)
    }
}

#[cfg(not(unix))]
fn group_is_alive(_child: &Child) -> Result<bool, GitRevisionError> {
    Ok(false)
}

enum ControlledOutput {
    Complete(Output),
    BlobBudgetExhausted,
}

#[cfg(not(unix))]
fn terminate_group(child: &mut Child) -> Result<(), GitRevisionError> {
    child
        .kill()
        .map_err(|_| GitRevisionError::ContainmentFailure)
}

fn run_controlled(
    mut command: Command,
    stdin: Option<Vec<u8>>,
    stdout_cap: usize,
    deadline: Instant,
    inspect_blob_headers: bool,
) -> Result<ControlledOutput, GitRevisionError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    isolate(&mut command);
    let mut child = command
        .spawn()
        .map_err(|_| GitRevisionError::ProcessFailure)?;
    if let Some(input) = stdin {
        let mut pipe = child.stdin.take().ok_or(GitRevisionError::ProcessFailure)?;
        pipe.write_all(&input)
            .map_err(|_| GitRevisionError::ProcessFailure)?;
    }
    drop(child.stdin.take());

    let mut stdout = child
        .stdout
        .take()
        .ok_or(GitRevisionError::ProcessFailure)?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or(GitRevisionError::ProcessFailure)?;
    #[derive(Clone, Copy)]
    enum ReaderEvent {
        InvalidFraming,
        BlobBudgetExhausted,
        ContainmentFailure,
    }
    let (stdout_overflow, overflow_receiver) = mpsc::sync_channel(1);
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 8192];
        let mut exceeded = false;
        let mut header = Vec::new();
        let mut payload_remaining = 0usize;
        let mut payload_newline = false;
        let mut announced_payload_bytes = 0usize;
        loop {
            let count = stdout.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            let available = stdout_cap.saturating_sub(bytes.len());
            bytes.extend_from_slice(&buffer[..count.min(available)]);
            if count > available && !exceeded {
                exceeded = true;
                let _ = stdout_overflow.try_send(ReaderEvent::InvalidFraming);
            }
            if inspect_blob_headers {
                for &byte in &buffer[..count] {
                    if payload_remaining != 0 {
                        payload_remaining -= 1;
                        if payload_remaining == 0 {
                            payload_newline = true;
                        }
                    } else if payload_newline {
                        payload_newline = false;
                        header.clear();
                    } else if byte == b'\n' {
                        if let Ok(line) = std::str::from_utf8(&header) {
                            if let Some(size) = line
                                .rsplit(' ')
                                .next()
                                .and_then(|v| v.parse::<usize>().ok())
                            {
                                if size > MAX_BLOB_BYTES {
                                    let _ =
                                        stdout_overflow.try_send(ReaderEvent::BlobBudgetExhausted);
                                } else {
                                    announced_payload_bytes =
                                        announced_payload_bytes.saturating_add(size);
                                    if announced_payload_bytes > MAX_TOTAL_BYTES {
                                        let _ = stdout_overflow
                                            .try_send(ReaderEvent::BlobBudgetExhausted);
                                    }
                                    payload_remaining = size;
                                    payload_newline = size == 0;
                                }
                            }
                        }
                        header.clear();
                    } else {
                        header.push(byte);
                    }
                }
            }
        }
        std::io::Result::Ok((bytes, exceeded))
    });
    let stderr_reader = thread::spawn(move || {
        let mut observed = false;
        let mut buffer = [0u8; 1024];
        loop {
            let count = stderr.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            if !observed {
                observed = true;
            }
        }
        std::io::Result::Ok(observed)
    });
    let (status, event) = loop {
        if let Ok(event) = overflow_receiver.try_recv() {
            terminate_group(&mut child)?;
            let status = child
                .wait()
                .map_err(|_| GitRevisionError::ContainmentFailure)?;
            break (status, Some(event));
        }
        match child.try_wait() {
            Ok(Some(status)) if group_is_alive(&child)? => {
                terminate_group(&mut child)?;
                break (status, Some(ReaderEvent::ContainmentFailure));
            }
            Ok(Some(status)) => break (status, None),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                terminate_group(&mut child)?;
                child
                    .wait()
                    .map_err(|_| GitRevisionError::ContainmentFailure)?;
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(GitRevisionError::Timeout);
            }
            Err(_) => return Err(GitRevisionError::ProcessFailure),
        }
    };
    let (stdout, stdout_exceeded) = stdout_reader
        .join()
        .map_err(|_| GitRevisionError::ProcessFailure)?
        .map_err(|_| GitRevisionError::ProcessFailure)?;
    let stderr_observed = stderr_reader
        .join()
        .map_err(|_| GitRevisionError::ProcessFailure)?
        .map_err(|_| GitRevisionError::ProcessFailure)?;
    if matches!(event, Some(ReaderEvent::BlobBudgetExhausted)) {
        return Ok(ControlledOutput::BlobBudgetExhausted);
    }
    if matches!(event, Some(ReaderEvent::ContainmentFailure)) {
        return Err(GitRevisionError::ContainmentFailure);
    }
    if event.is_some() {
        return Err(GitRevisionError::InvalidFraming);
    }
    if stdout_exceeded {
        return Err(GitRevisionError::InvalidFraming);
    }
    Ok(ControlledOutput::Complete(Output {
        status,
        stdout,
        stderr: if stderr_observed { vec![1] } else { Vec::new() },
    }))
}

fn validate_public_inputs(
    executable: &Path,
    repository: &Path,
    revision: &OsStr,
    paths: &[PathBuf],
) -> Result<(String, Vec<String>), GitRevisionError> {
    if !executable.is_absolute()
        || !repository.is_absolute()
        || executable.canonicalize().ok().as_deref() != Some(executable)
        || repository.canonicalize().ok().as_deref() != Some(repository)
        || !executable.is_file()
        || paths.len() > MAX_PATHS
    {
        return Err(GitRevisionError::InvalidInput);
    }
    let git_dir = repository.join(".git");
    let objects = git_dir.join("objects");
    let contained = |path: &Path, parent: &Path| {
        std::fs::symlink_metadata(path)
            .ok()
            .is_some_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
            && path
                .canonicalize()
                .ok()
                .is_some_and(|canonical| canonical.starts_with(parent))
    };
    if !contained(&git_dir, repository) || !contained(&objects, &git_dir) {
        return Err(GitRevisionError::ContainmentFailure);
    }
    for directory in [objects.join("info"), objects.join("pack")] {
        match std::fs::symlink_metadata(&directory) {
            Ok(metadata) => {
                let is_confined_directory = metadata.is_dir()
                    && !metadata.file_type().is_symlink()
                    && directory
                        .canonicalize()
                        .ok()
                        .is_some_and(|canonical| canonical.starts_with(&objects));
                if !is_confined_directory {
                    return Err(GitRevisionError::ContainmentFailure);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(GitRevisionError::ContainmentFailure),
        }
    }
    for name in ["alternates", "http-alternates"] {
        let path = objects.join("info").join(name);
        match std::fs::symlink_metadata(&path) {
            Ok(metadata) => {
                let is_confined_empty_file = metadata.is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.len() == 0
                    && path
                        .canonicalize()
                        .ok()
                        .is_some_and(|canonical| canonical.starts_with(&objects));
                if !is_confined_empty_file {
                    return Err(GitRevisionError::ContainmentFailure);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(GitRevisionError::ContainmentFailure),
        }
    }
    let revision = revision.to_str().ok_or(GitRevisionError::InvalidInput)?;
    if revision.is_empty()
        || revision.len() > 255
        || revision
            .bytes()
            .any(|byte| matches!(byte, 0 | b'\n' | b'\r'))
    {
        return Err(GitRevisionError::InvalidInput);
    }
    let mut logical = Vec::with_capacity(paths.len());
    let mut unique = BTreeSet::new();
    for path in paths {
        let path = path.to_str().ok_or(GitRevisionError::InvalidInput)?;
        let components: Vec<&str> = path.split('/').collect();
        if path.is_empty()
            || path.len() > 4096
            || path.starts_with('/')
            || path.ends_with('/')
            || path.bytes().any(|byte| matches!(byte, 0 | b'\n' | b'\r'))
            || components.iter().any(|part| {
                part.is_empty() || *part == "." || *part == ".." || part.starts_with(':')
            })
            || !unique.insert(path.to_string())
        {
            return Err(GitRevisionError::InvalidInput);
        }
        logical.push(path.to_string());
    }
    logical.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok((revision.to_string(), logical))
}

fn validate_object_database(repository: &Path) -> Result<(), GitRevisionError> {
    let objects = repository.join(".git/objects");
    let canonical_objects = objects
        .canonicalize()
        .map_err(|_| GitRevisionError::ContainmentFailure)?;
    if !canonical_objects.starts_with(repository.join(".git")) {
        return Err(GitRevisionError::ContainmentFailure);
    }

    let mut pending = vec![objects];
    while let Some(directory) = pending.pop() {
        let entries =
            std::fs::read_dir(&directory).map_err(|_| GitRevisionError::ContainmentFailure)?;
        for entry in entries {
            let entry = entry.map_err(|_| GitRevisionError::ContainmentFailure)?;
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|_| GitRevisionError::ContainmentFailure)?;
            if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
                return Err(GitRevisionError::ContainmentFailure);
            }
            let canonical = path
                .canonicalize()
                .map_err(|_| GitRevisionError::ContainmentFailure)?;
            if !canonical.starts_with(&canonical_objects) {
                return Err(GitRevisionError::ContainmentFailure);
            }
            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }
    Ok(())
}

pub fn load_revision_with_git(
    git_executable: &Path,
    repository_root: &Path,
    revision: &OsStr,
    logical_paths: &[PathBuf],
) -> Result<GitRevisionContent, GitRevisionError> {
    let deadline = Instant::now() + GIT_TIMEOUT;
    let (revision, logical_paths) =
        validate_public_inputs(git_executable, repository_root, revision, logical_paths)?;
    validate_object_database(repository_root)?;

    let mut rev_parse = controlled_command(git_executable, repository_root);
    rev_parse
        .args(["rev-parse", "--verify", "--end-of-options"])
        .arg(format!("{revision}^{{commit}}"));
    let rev_parse_result = run_controlled(rev_parse, None, 65, deadline, false);
    validate_object_database(repository_root)?;
    let ControlledOutput::Complete(output) = rev_parse_result? else {
        return Err(GitRevisionError::InvalidFraming);
    };
    if !output.status.success() {
        return Err(GitRevisionError::MissingRef);
    }
    if !output.stderr.is_empty()
        || output.stdout.last() != Some(&b'\n')
        || output.stdout[..output.stdout.len().saturating_sub(1)].contains(&b'\n')
    {
        return Err(GitRevisionError::InvalidFraming);
    }
    let oid_bytes = &output.stdout[..output.stdout.len().saturating_sub(1)];
    if !valid_oid(oid_bytes) {
        return Err(GitRevisionError::InvalidFraming);
    }
    let oid =
        String::from_utf8(oid_bytes.to_vec()).map_err(|_| GitRevisionError::InvalidFraming)?;

    let mut ls_tree = controlled_command(git_executable, repository_root);
    ls_tree.args(["ls-tree", "-rz", "--full-tree", &oid, "--"]);
    for path in &logical_paths {
        ls_tree.arg(format!(":(top,literal){path}"));
    }
    validate_object_database(repository_root)?;
    let ls_tree_result = run_controlled(ls_tree, None, 2_200_000, deadline, false);
    validate_object_database(repository_root)?;
    let ControlledOutput::Complete(output) = ls_tree_result? else {
        return Err(GitRevisionError::InvalidFraming);
    };
    if !output.status.success() {
        return Err(GitRevisionError::ProcessFailure);
    }
    if !output.stderr.is_empty() || (!output.stdout.is_empty() && output.stdout.last() != Some(&0))
    {
        return Err(GitRevisionError::InvalidFraming);
    }
    let requested: BTreeSet<&str> = logical_paths.iter().map(String::as_str).collect();
    let mut paths: BTreeMap<PathBuf, GitPathContent> = logical_paths
        .iter()
        .map(|path| (PathBuf::from(path), GitPathContent::Missing))
        .collect();
    let mut blobs_by_path = Vec::new();
    let mut previous: Option<String> = None;
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or(GitRevisionError::InvalidFraming)?;
        let meta =
            std::str::from_utf8(&record[..tab]).map_err(|_| GitRevisionError::InvalidFraming)?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|_| GitRevisionError::InvalidFraming)?;
        let fields: Vec<&str> = meta.split(' ').collect();
        if fields.len() != 3
            || !requested.contains(path)
            || previous.as_deref().is_some_and(|old| old >= path)
        {
            return Err(GitRevisionError::InvalidFraming);
        }
        previous = Some(path.to_string());
        if fields[0].len() != 6
            || !fields[0].bytes().all(|byte| (b'0'..=b'7').contains(&byte))
            || !fields[1].is_ascii()
            || !valid_oid(fields[2].as_bytes())
        {
            return Err(GitRevisionError::InvalidFraming);
        }
        let regular = matches!((fields[0], fields[1]), ("100644" | "100755", "blob"));
        if regular {
            blobs_by_path.push((path.to_string(), fields[2].to_string()));
        } else {
            paths.insert(
                PathBuf::from(path),
                GitPathContent::Unknown(GitUnknownReason::ForbiddenObjectKind),
            );
        }
    }

    let mut ordered_oids = Vec::new();
    let mut seen = BTreeSet::new();
    for (_, blob) in &blobs_by_path {
        if seen.insert(blob.clone()) {
            ordered_oids.push(blob.clone());
        }
    }
    let mut input = Vec::new();
    for blob in &ordered_oids {
        input.extend_from_slice(format!("contents {blob}\n").as_bytes());
    }
    input.extend_from_slice(b"flush\n");
    let mut cat_file = controlled_command(git_executable, repository_root);
    cat_file.args(["cat-file", "--batch-command", "--buffer"]);
    validate_object_database(repository_root)?;
    let output = run_controlled(cat_file, Some(input), 33_600_000, deadline, true);
    validate_object_database(repository_root)?;
    let output = output?;
    if matches!(output, ControlledOutput::BlobBudgetExhausted) {
        for (path, _) in blobs_by_path {
            paths.insert(
                PathBuf::from(path),
                GitPathContent::Unknown(GitUnknownReason::BudgetExhausted),
            );
        }
        return Ok(GitRevisionContent { oid, paths });
    }
    let ControlledOutput::Complete(output) = output else {
        unreachable!("blob budget outcome handled above")
    };
    if !output.status.success() {
        return Err(GitRevisionError::ProcessFailure);
    }
    if !output.stderr.is_empty() {
        return Err(GitRevisionError::InvalidFraming);
    }
    let mut cursor = 0usize;
    let mut blobs = BTreeMap::new();
    let mut budget_exhausted = false;
    for expected in &ordered_oids {
        let relative_newline = output.stdout[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .ok_or(GitRevisionError::InvalidFraming)?;
        let newline = cursor + relative_newline;
        let header = std::str::from_utf8(&output.stdout[cursor..newline])
            .map_err(|_| GitRevisionError::InvalidFraming)?;
        if header == format!("{expected} missing") {
            blobs.insert(expected.clone(), None);
            cursor = newline + 1;
            continue;
        }
        let fields: Vec<&str> = header.split(' ').collect();
        if fields.len() != 3 || fields[0] != expected || fields[1] != "blob" {
            return Err(GitRevisionError::InvalidFraming);
        }
        let size: usize = fields[2]
            .parse()
            .map_err(|_| GitRevisionError::InvalidFraming)?;
        if size.to_string() != fields[2] {
            return Err(GitRevisionError::InvalidFraming);
        }
        if size > MAX_BLOB_BYTES {
            budget_exhausted = true;
            break;
        }
        let start = newline + 1;
        let end = start
            .checked_add(size)
            .ok_or(GitRevisionError::InvalidFraming)?;
        if end >= output.stdout.len() || output.stdout[end] != b'\n' {
            return Err(GitRevisionError::InvalidFraming);
        }
        blobs.insert(expected.clone(), Some(output.stdout[start..end].to_vec()));
        cursor = end + 1;
    }
    if !budget_exhausted && cursor != output.stdout.len() {
        return Err(GitRevisionError::InvalidFraming);
    }
    let total = blobs_by_path
        .iter()
        .filter_map(|(_, oid)| blobs.get(oid).and_then(Option::as_ref).map(Vec::len))
        .sum::<usize>();
    budget_exhausted |= total > MAX_TOTAL_BYTES;
    for (path, blob) in blobs_by_path {
        let value = if budget_exhausted {
            GitPathContent::Unknown(GitUnknownReason::BudgetExhausted)
        } else {
            match blobs.get(&blob) {
                Some(Some(bytes)) => GitPathContent::Blob(bytes.clone()),
                Some(None) => GitPathContent::Unknown(GitUnknownReason::MissingObject),
                None => return Err(GitRevisionError::InvalidFraming),
            }
        };
        paths.insert(PathBuf::from(path), value);
    }
    Ok(GitRevisionContent { oid, paths })
}

pub fn extract_snapshot_from_revision_content(
    revision: &GitRevisionContent,
    specs: &[ObservableSpec],
) -> Result<ArtifactFacts, String> {
    let mut snapshot = extract_snapshot_from_content(&revision.oid, specs, |path| {
        Ok(match revision.paths.get(path) {
            Some(GitPathContent::Blob(bytes)) => Some(bytes.clone()),
            Some(GitPathContent::Missing | GitPathContent::Unknown(_)) | None => None,
        })
    })?;
    for spec in specs {
        let reason = match revision.paths.get(&spec.file) {
            Some(GitPathContent::Unknown(GitUnknownReason::MissingObject)) => {
                Some(UnknownReason::OpaqueConstruction)
            }
            Some(GitPathContent::Unknown(GitUnknownReason::ForbiddenObjectKind)) => {
                Some(UnknownReason::UnsupportedParser)
            }
            Some(GitPathContent::Unknown(GitUnknownReason::BudgetExhausted)) => {
                Some(UnknownReason::BudgetExhausted)
            }
            Some(GitPathContent::Blob(_) | GitPathContent::Missing) | None => None,
        };
        if let Some(reason) = reason {
            snapshot
                .observables
                .insert(spec.key.clone(), ObservableValue::Unknown(reason));
        }
    }
    Ok(snapshot)
}

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
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env("GIT_PAGER", "cat");
    command
}

const SELF_CONTAINED_ERROR: &str = "refinement mode requires a self-contained object database";

pub fn require_self_contained_object_database(repository: &Path) -> Result<(), String> {
    let repository = repository.canonicalize().map_err(|error| {
        format!("{SELF_CONTAINED_ERROR}; cannot resolve repository root: {error}")
    })?;
    let git_dir = repository.join(".git");
    let git_metadata = std::fs::symlink_metadata(&git_dir).map_err(|error| {
        format!(
            "{SELF_CONTAINED_ERROR}; cannot inspect {}: {error}",
            git_dir.display()
        )
    })?;
    if git_metadata.file_type().is_symlink() || !git_metadata.is_dir() {
        return Err(format!(
            "{SELF_CONTAINED_ERROR}; linked worktrees, bare repositories and gitdir indirection are not supported"
        ));
    }
    let canonical_git_dir = git_dir.canonicalize().map_err(|error| {
        format!(
            "{SELF_CONTAINED_ERROR}; cannot resolve {}: {error}",
            git_dir.display()
        )
    })?;
    if !canonical_git_dir.starts_with(&repository) {
        return Err(format!(
            "{SELF_CONTAINED_ERROR}; Git metadata escapes repository root"
        ));
    }
    if git_dir.join("commondir").exists() {
        return Err(format!(
            "{SELF_CONTAINED_ERROR}; linked common directories are not supported"
        ));
    }
    let objects = git_dir.join("objects");
    let metadata = std::fs::symlink_metadata(&objects).map_err(|error| {
        format!(
            "{SELF_CONTAINED_ERROR}; cannot inspect {}: {error}",
            objects.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{SELF_CONTAINED_ERROR}; Git object metadata must be a local directory"
        ));
    }
    let canonical_objects = objects.canonicalize().map_err(|error| {
        format!(
            "{SELF_CONTAINED_ERROR}; cannot resolve {}: {error}",
            objects.display()
        )
    })?;
    if !canonical_objects.starts_with(&canonical_git_dir) {
        return Err(format!(
            "{SELF_CONTAINED_ERROR}; Git object metadata escapes repository root"
        ));
    }
    let info = objects.join("info");
    match std::fs::symlink_metadata(&info) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(format!(
                    "{SELF_CONTAINED_ERROR}; Git object metadata must be a local directory"
                ));
            }
            let canonical = info.canonicalize().map_err(|error| {
                format!(
                    "{SELF_CONTAINED_ERROR}; cannot resolve {}: {error}",
                    info.display()
                )
            })?;
            if !canonical.starts_with(&canonical_objects) {
                return Err(format!(
                    "{SELF_CONTAINED_ERROR}; Git object metadata escapes repository root"
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "{SELF_CONTAINED_ERROR}; cannot inspect {}: {error}",
                info.display()
            ))
        }
    }
    for name in ["alternates", "http-alternates"] {
        let path = git_dir.join("objects/info").join(name);
        match std::fs::read(&path) {
            Ok(bytes) if !bytes.is_empty() => {
                return Err(format!(
                    "{SELF_CONTAINED_ERROR}; {} is not empty",
                    path.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{SELF_CONTAINED_ERROR}; cannot inspect {}: {error}",
                    path.display()
                ))
            }
        }
    }
    let config_path = git_dir.join("config");
    let config = std::fs::read_to_string(&config_path).map_err(|error| {
        format!(
            "{SELF_CONTAINED_ERROR}; cannot inspect {}: {error}",
            config_path.display()
        )
    })?;
    for line in config.lines() {
        let key = line
            .split_once('=')
            .map(|(key, _)| key)
            .unwrap_or(line)
            .trim();
        let normalized: String = key
            .chars()
            .filter(|ch| !ch.is_whitespace() && *ch != '-' && *ch != '_')
            .flat_map(char::to_lowercase)
            .collect();
        if normalized.contains("alternateobject") || normalized.contains("objectdirectory") {
            return Err(format!(
                "{SELF_CONTAINED_ERROR}; external object database configuration is not supported"
            ));
        }
    }
    Ok(())
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

    #[test]
    fn self_contained_preflight_distinguishes_empty_and_nonempty_alternates() {
        let repository = tempfile::tempdir().unwrap();
        let info = repository.path().join(".git/objects/info");
        std::fs::create_dir_all(&info).unwrap();
        std::fs::write(
            repository.path().join(".git/config"),
            "[core]\n\tbare = false\n",
        )
        .unwrap();
        std::fs::write(info.join("alternates"), b"").unwrap();
        require_self_contained_object_database(repository.path()).unwrap();
        std::fs::write(info.join("alternates"), b"/external/objects\n").unwrap();
        let error = require_self_contained_object_database(repository.path()).unwrap_err();
        assert!(error.contains("self-contained object database"));
    }

    #[cfg(unix)]
    #[test]
    fn self_contained_preflight_rejects_git_directory_symlink() {
        use std::os::unix::fs::symlink;
        let repository = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(external.path().join("objects/info")).unwrap();
        std::fs::write(external.path().join("config"), "[core]\n\tbare = false\n").unwrap();
        symlink(external.path(), repository.path().join(".git")).unwrap();
        let error = require_self_contained_object_database(repository.path()).unwrap_err();
        assert!(error.contains("self-contained object database"));
    }
}
