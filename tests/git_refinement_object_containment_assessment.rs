//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @prompt-hash d1231d0f
//! @layer L3
//! @updated 2026-08-25

#![cfg(unix)]

use crystalline_lint::infra::git_refinement::{
    load_revision_with_git, GitPathContent, GitRevisionError,
};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
const LOGICAL_PATH: &str = "evidence.txt";
const SENTINEL: &[u8] = b"P0101-B2-external-object-sentinel\n";

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "crystalline-p0101-b2-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated fixture root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Repository {
    _fixture: TempDir,
    root: PathBuf,
    external: PathBuf,
    git: PathBuf,
    blob_oid: String,
}

impl Repository {
    fn loose(label: &str) -> Self {
        let fixture = TempDir::new(label);
        let root = fixture.path().join("repository");
        let external = fixture.path().join("external");
        fs::create_dir(&root).expect("create repository root");
        fs::create_dir(&external).expect("create external sentinel root");

        let git = real_git();
        git_ok(&git, &root, &["init", "-q"]);
        git_ok(&git, &root, &["config", "user.name", "P0101 fixture"]);
        git_ok(
            &git,
            &root,
            &["config", "user.email", "p0101@example.invalid"],
        );
        fs::write(root.join(LOGICAL_PATH), SENTINEL).expect("write fixture content");
        git_ok(&git, &root, &["add", LOGICAL_PATH]);
        git_ok(&git, &root, &["commit", "-qm", "fixture"]);
        let blob_oid = git_stdout(&git, &root, &["rev-parse", "HEAD:evidence.txt"]);

        Self {
            _fixture: fixture,
            root,
            external,
            git,
            blob_oid,
        }
    }

    fn loose_object(&self) -> PathBuf {
        self.root
            .join(".git/objects")
            .join(&self.blob_oid[..2])
            .join(&self.blob_oid[2..])
    }

    fn load(&self, executable: &Path) -> Result<Vec<u8>, GitRevisionError> {
        let revision = load_revision_with_git(
            executable,
            &self.root,
            OsStr::new("HEAD"),
            &[PathBuf::from(LOGICAL_PATH)],
        )?;
        match revision.paths.get(Path::new(LOGICAL_PATH)) {
            Some(GitPathContent::Blob(bytes)) => Ok(bytes.clone()),
            other => panic!("expected regular blob, got {other:?}"),
        }
    }

    fn pack(&self) {
        git_ok(&self.git, &self.root, &["gc", "--prune=now", "-q"]);
    }
}

#[test]
fn regular_internal_loose_object_is_accepted() {
    let repository = Repository::loose("regular-loose");
    assert_eq!(repository.load(&repository.git), Ok(SENTINEL.to_vec()));
}

#[test]
fn requested_loose_object_symlink_to_external_bytes_fails_closed() {
    let repository = Repository::loose("loose-object-link");
    replace_file_with_external_symlink(
        &repository.loose_object(),
        &repository.external.join("external-loose-object"),
    );

    assert_eq!(
        repository.load(&repository.git),
        Err(GitRevisionError::ContainmentFailure),
        "external object bytes must never be published"
    );
}

#[test]
fn requested_loose_object_fanout_symlink_fails_closed() {
    let repository = Repository::loose("loose-fanout-link");
    let object = repository.loose_object();
    let fanout = object.parent().expect("fanout directory");
    let external_fanout = repository.external.join("external-fanout");
    fs::rename(fanout, &external_fanout).expect("move fanout outside object database");
    symlink(&external_fanout, fanout).expect("replace fanout with external symlink");

    assert_eq!(
        repository.load(&repository.git),
        Err(GitRevisionError::ContainmentFailure)
    );
}

#[test]
fn regular_internal_pack_and_index_are_accepted() {
    let repository = Repository::loose("regular-pack");
    repository.pack();
    assert_eq!(repository.load(&repository.git), Ok(SENTINEL.to_vec()));
}

#[test]
fn accessible_pack_symlink_to_external_bytes_fails_closed() {
    assert_pack_component_symlink_is_rejected("pack");
}

#[test]
fn accessible_index_symlink_to_external_bytes_fails_closed() {
    assert_pack_component_symlink_is_rejected("idx");
}

#[test]
fn concurrent_regular_to_symlink_swap_after_preflight_fails_closed() {
    let repository = Repository::loose("concurrent-swap");
    let object = repository.loose_object();
    let external_object = repository.external.join("swapped-object");
    fs::copy(&object, &external_object).expect("copy valid object outside repository");
    let marker = repository.external.join("swap-complete");
    let wrapper = repository._fixture.path().join("git-swap-wrapper");
    let script = format!(
        "#!/bin/sh\nset -eu\nif [ ! -e '{}' ]; then\n  rm -- '{}'\n  ln -s -- '{}' '{}'\n  : > '{}'\nfi\nexec '{}' \"$@\"\n",
        shell_path(&marker),
        shell_path(&object),
        shell_path(&external_object),
        shell_path(&object),
        shell_path(&marker),
        shell_path(&repository.git),
    );
    fs::write(&wrapper, script).expect("write synchronized Git wrapper");
    let mut permissions = fs::metadata(&wrapper)
        .expect("wrapper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&wrapper, permissions).expect("make wrapper executable");

    assert_eq!(
        repository.load(&wrapper),
        Err(GitRevisionError::ContainmentFailure),
        "a swap performed by the first child, after adapter preflight, must fail closed"
    );
    assert!(
        marker.exists(),
        "fixture must prove that the race was exercised"
    );
    assert!(
        fs::symlink_metadata(&object)
            .expect("swapped object metadata")
            .file_type()
            .is_symlink(),
        "fixture must leave direct evidence of the regular-to-symlink swap"
    );
}

fn assert_pack_component_symlink_is_rejected(extension: &str) {
    let repository = Repository::loose(&format!("{extension}-link"));
    repository.pack();
    let pack_dir = repository.root.join(".git/objects/pack");
    let component = fs::read_dir(&pack_dir)
        .expect("read pack directory")
        .map(|entry| entry.expect("pack entry").path())
        .find(|path| path.extension() == Some(OsStr::new(extension)))
        .expect("accessible packed component");
    replace_file_with_external_symlink(
        &component,
        &repository.external.join(format!("external.{extension}")),
    );

    assert_eq!(
        repository.load(&repository.git),
        Err(GitRevisionError::ContainmentFailure),
        "Git must not consume an external .{extension} through the object database"
    );
}

fn replace_file_with_external_symlink(internal: &Path, external: &Path) {
    fs::copy(internal, external).expect("copy valid bytes to external sentinel location");
    fs::remove_file(internal).expect("remove internal regular component");
    symlink(external, internal).expect("install external symlink");
}

fn real_git() -> PathBuf {
    ["/usr/bin/git", "/bin/git"]
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.is_absolute() && path.is_file())
        .expect("P0101 B2 requires a real Git executable at an absolute path")
}

fn git_ok(git: &Path, cwd: &Path, args: &[&str]) {
    let output = Command::new(git)
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run fixture Git");
    assert!(
        output.status.success(),
        "fixture Git failed for {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(git: &Path, cwd: &Path, args: &[&str]) -> String {
    let output = Command::new(git)
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run fixture Git");
    assert!(output.status.success(), "fixture Git failed for {args:?}");
    String::from_utf8(output.stdout)
        .expect("fixture Git stdout must be UTF-8")
        .trim()
        .to_owned()
}

fn shell_path(path: &Path) -> String {
    let value = path.to_str().expect("fixture paths must be UTF-8");
    assert!(!value.contains('\''), "fixture path must be shell-quotable");
    value.to_owned()
}
