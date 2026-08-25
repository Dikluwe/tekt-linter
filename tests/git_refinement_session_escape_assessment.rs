#![cfg(target_os = "linux")]

use crystalline_lint::infra::git_refinement::{load_revision_with_git, GitRevisionError};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const WATCHDOG: Duration = Duration::from_secs(15);
const FIXTURE_LIFETIME_SECONDS: u64 = 60;

#[derive(Clone, Copy)]
enum Escape {
    BeforeTimeoutKeepPipes,
    BeforeTimeoutClosePipes,
    DoubleForkIntermediateExits,
    DuringTimeoutKeepPipes,
}

struct Fixture {
    _root: TempDir,
    repository: PathBuf,
    git: PathBuf,
    pid_file: PathBuf,
}

#[derive(Clone, Debug)]
struct ProcessIdentity {
    pid: i32,
    start_time: String,
}

impl Fixture {
    fn new(escape: Escape) -> Self {
        let root = tempfile::tempdir().expect("create fixture root");
        let repository = root.path().join("repository");
        fs::create_dir_all(repository.join(".git/objects")).expect("create object database");
        let git = root.path().join("hostile-git");
        let pid_file = root.path().join("escaped.pid");
        let escaped = escaped_program(&pid_file, escape);
        let script = format!(
            "#!/bin/sh\ncase \"$*\" in\n  *rev-parse*)\n    {escaped}\n    printf '%s\\n' 0123456789012345678901234567890123456789\n    exit 0\n    ;;\nesac\nexit 97\n"
        );
        fs::write(&git, script).expect("write hostile git fixture");
        let mut permissions = fs::metadata(&git).expect("stat fixture").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&git, permissions).expect("make fixture executable");
        Self {
            _root: root,
            repository,
            git,
            pid_file,
        }
    }

    fn published_identity(&self) -> ProcessIdentity {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Ok(raw) = fs::read_to_string(&self.pid_file) {
                let pid = raw.trim().parse().expect("fixture published numeric PID");
                if let Some(start_time) = process_start_time(pid) {
                    return ProcessIdentity { pid, start_time };
                }
            }
            assert!(
                Instant::now() < deadline,
                "fixture did not publish a live PID"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn escaped_program(pid_file: &Path, escape: Escape) -> String {
    let pid_file = shell_single_quote(pid_file.as_os_str().to_string_lossy().as_ref());
    let await_publication = format!("while [ ! -s {pid_file} ]; do /bin/sleep 0.01; done");
    let leaf =
        format!("printf '%s\\n' \"$$\" > {pid_file}; exec /bin/sleep {FIXTURE_LIFETIME_SECONDS}");
    let leaf = shell_single_quote(&leaf);
    match escape {
        Escape::BeforeTimeoutKeepPipes => {
            format!("/usr/bin/setsid /bin/sh -c {leaf} & {await_publication}")
        }
        Escape::BeforeTimeoutClosePipes => {
            format!("/usr/bin/setsid /bin/sh -c {leaf} >/dev/null 2>&1 & {await_publication}")
        }
        Escape::DoubleForkIntermediateExits => {
            let middle =
                shell_single_quote(&format!("/bin/sh -c {leaf} & {await_publication}; exit 0"));
            format!("/usr/bin/setsid /bin/sh -c {middle} & {await_publication}")
        }
        Escape::DuringTimeoutKeepPipes => format!(
            "/bin/sh -c {} & /bin/sleep {FIXTURE_LIFETIME_SECONDS}",
            shell_single_quote(&format!(
                "/bin/sleep 1; exec /usr/bin/setsid /bin/sh -c {leaf}"
            ))
        ),
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn process_start_time(pid: i32) -> Option<String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_name = stat.rsplit_once(") ")?.1;
    let mut fields = after_name.split_whitespace();
    let state = fields.next()?;
    if state == "Z" {
        return None;
    }
    // Field 22 overall is field 20 after removing PID and comm.
    fields.nth(18).map(str::to_owned)
}

fn identity_is_alive(identity: &ProcessIdentity) -> bool {
    process_start_time(identity.pid).as_deref() == Some(identity.start_time.as_str())
}

fn cleanup_if_same_process(identity: &ProcessIdentity) {
    if identity_is_alive(identity) {
        let _ = Command::new("/bin/kill")
            .args(["-KILL", &identity.pid.to_string()])
            .status();
    }
}

fn confront(escape: Escape) {
    let fixture = Fixture::new(escape);
    let git = fixture.git.clone();
    let repository = fixture.repository.clone();
    let (tx, rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        let result = load_revision_with_git(
            &git,
            &repository,
            OsStr::new("HEAD"),
            &[PathBuf::from("src/lib.rs")],
        );
        let _ = tx.send(result.map(|_| ()));
    });

    let identity = fixture.published_identity();
    let adapter_result = rx.recv_timeout(WATCHDOG);
    let escaped_before_fixture_cleanup = identity_is_alive(&identity);

    // Defensive cleanup is deliberately after both observations. It cannot make the
    // adapter result or the lifecycle assertion pass.
    cleanup_if_same_process(&identity);
    let _ = worker.join();
    let cleanup_deadline = Instant::now() + Duration::from_secs(2);
    while identity_is_alive(&identity) && Instant::now() < cleanup_deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !identity_is_alive(&identity),
        "fixture cleanup left PID {} alive",
        identity.pid
    );

    assert!(
        adapter_result.is_ok(),
        "adapter exceeded the independent 15-second watchdog"
    );
    assert_eq!(
        adapter_result.unwrap(),
        Err(GitRevisionError::ContainmentFailure),
        "an escaped session must fail closed before publishing bytes"
    );
    assert!(
        !escaped_before_fixture_cleanup,
        "escaped PID {} remained alive; fixture cleanup is not adapter success",
        identity.pid
    );
}

#[test]
#[ignore = "future sealed verification: deliberate session escape is outside local mode"]
fn setsid_keeps_pipes_open_after_leader_exits() {
    confront(Escape::BeforeTimeoutKeepPipes);
}

#[test]
#[ignore = "future sealed verification: deliberate session escape is outside local mode"]
fn setsid_closes_pipes_but_remains_in_scope() {
    confront(Escape::BeforeTimeoutClosePipes);
}

#[test]
#[ignore = "future sealed verification: deliberate session escape is outside local mode"]
fn double_fork_intermediate_exit_does_not_release_leaf() {
    confront(Escape::DoubleForkIntermediateExits);
}

#[test]
#[ignore = "future sealed verification: deliberate session escape is outside local mode"]
fn escape_during_timeout_remains_bounded_and_contained() {
    confront(Escape::DuringTimeoutKeepPipes);
}
