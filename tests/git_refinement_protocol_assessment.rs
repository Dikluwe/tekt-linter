#![cfg(unix)]

use crystalline_lint::infra::git_refinement::{
    load_revision_with_git, GitPathContent, GitRevisionError, GitUnknownReason,
};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
const PREFIX: [&str; 16] = [
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

struct Harness {
    root: PathBuf,
    git: PathBuf,
}

impl Harness {
    fn new(scenario: &str) -> Self {
        let id = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("crystalline-p0100-b1-{}-{id}", std::process::id()));
        fs::create_dir_all(root.join(".git/objects/info")).unwrap();
        fs::create_dir_all(root.join(".git/objects/pack")).unwrap();
        fs::write(root.join(format!("scenario_{scenario}")), b"").unwrap();
        let git = root.join("hostile-git");
        fs::copy(
            Path::new("tests/fixtures/git_refinement_protocol/hostile_git.sh"),
            &git,
        )
        .unwrap();
        let mut permissions = fs::metadata(&git).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&git, permissions).unwrap();
        Self { root, git }
    }

    fn load(
        &self,
        revision: &str,
        paths: &[&str],
    ) -> Result<crystalline_lint::infra::git_refinement::GitRevisionContent, GitRevisionError> {
        load_revision_with_git(
            &self.git,
            &self.root,
            OsStr::new(revision),
            &paths.iter().map(PathBuf::from).collect::<Vec<_>>(),
        )
    }

    fn log(&self) -> String {
        fs::read_to_string(self.root.join("protocol.log")).unwrap()
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn argv(prefix_tail: &str) -> String {
    format!(
        "ARGV {} {prefix_tail}",
        PREFIX.map(|arg| format!("<{arg}>")).join(" ")
    )
}

#[test]
fn resolves_once_then_uses_only_the_opaque_oid() {
    let h = Harness::new("oid64");
    let result = h.load("--hostile-ref", &["-odd.rs"]).unwrap();
    let oid = "2222222222222222222222222222222222222222222222222222222222222222";
    assert_eq!(result.oid, oid);
    assert_eq!(
        result.paths[Path::new("-odd.rs")],
        GitPathContent::Blob(b"abc".to_vec())
    );

    let log = h.log();
    assert_eq!(log.matches("<rev-parse>").count(), 1);
    assert!(log.contains(&argv(
        "<rev-parse> <--verify> <--end-of-options> <--hostile-ref^{commit}>"
    )));
    assert!(log.contains(&argv(&format!(
        "<ls-tree> <-rz> <--full-tree> <{oid}> <--> <:(top,literal)-odd.rs>"
    ))));
    assert!(!log.contains("<ls-tree> <-rz> <--full-tree> <--hostile-ref>"));
    assert!(
        log.contains("STDIN <contents aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa>\nSTDIN <flush>")
    );
}

#[test]
fn supplies_exact_environment_and_literal_path_argv() {
    let h = Harness::new("default");
    h.load("main", &["-odd.rs"]).unwrap();
    let log = h.log();
    assert!(
        log.contains("ENV GIT_TERMINAL_PROMPT=<0> GIT_NO_LAZY_FETCH=<1> GIT_OPTIONAL_LOCKS=<0>")
    );
    assert!(log.contains("ENV GIT_NO_REPLACE_OBJECTS=<1> GIT_CONFIG_NOSYSTEM=<1> GIT_CONFIG_GLOBAL=</dev/null> LC_ALL=<C>"));
    assert!(log.contains("ABSENT PATH=<> HOME=<> XDG_CONFIG_HOME=<> GIT_DIR=<>"));
    assert!(log.contains("<--> <:(top,literal)-odd.rs>"));
    assert_eq!(log.matches("BEGIN\n").count(), 3);
}

#[test]
fn distinguishes_regular_missing_and_forbidden_object_kinds() {
    let h = Harness::new("types");
    let result = h
        .load("main", &["a.rs", "absent.rs", "gitlink", "link"])
        .unwrap();
    assert_eq!(
        result.paths[Path::new("a.rs")],
        GitPathContent::Blob(b"abc".to_vec())
    );
    assert_eq!(
        result.paths[Path::new("absent.rs")],
        GitPathContent::Missing
    );
    assert_eq!(
        result.paths[Path::new("gitlink")],
        GitPathContent::Unknown(GitUnknownReason::ForbiddenObjectKind)
    );
    assert_eq!(
        result.paths[Path::new("link")],
        GitPathContent::Unknown(GitUnknownReason::ForbiddenObjectKind)
    );
}

#[test]
fn rejects_invalid_framing_without_publishing_content() {
    let h = Harness::new("bad_framing");
    assert_eq!(
        h.load("main", &["a.rs"]),
        Err(GitRevisionError::InvalidFraming)
    );
}

#[test]
fn applies_inclusive_blob_budget_without_waiting_for_payload() {
    let h = Harness::new("budget");
    let result = h.load("main", &["large.bin"]).unwrap();
    assert_eq!(
        result.paths[Path::new("large.bin")],
        GitPathContent::Unknown(GitUnknownReason::BudgetExhausted)
    );
}

#[test]
fn validates_duplicate_and_path_count_budgets_before_spawn() {
    let duplicate = Harness::new("default");
    assert_eq!(
        duplicate.load("main", &["a.rs", "a.rs"]),
        Err(GitRevisionError::InvalidInput)
    );
    assert!(!duplicate.root.join("protocol.log").exists());

    let too_many = Harness::new("default");
    let paths = (0..513).map(|n| format!("p{n:03}.rs")).collect::<Vec<_>>();
    let refs = paths.iter().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        too_many.load("main", &refs),
        Err(GitRevisionError::InvalidInput)
    );
    assert!(!too_many.root.join("protocol.log").exists());
}

#[test]
fn maps_ref_failure_only_to_missing_ref() {
    let h = Harness::new("missing_ref");
    assert_eq!(
        h.load("missing", &["a.rs"]),
        Err(GitRevisionError::MissingRef)
    );
}
