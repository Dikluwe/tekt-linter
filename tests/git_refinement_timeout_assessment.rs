#![cfg(unix)]

use crystalline_lint::infra::git_refinement::{load_revision_with_git, GitRevisionError};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FIXTURE: &str = include_str!("fixtures/git_refinement_timeout/hostile-git.sh");
const WATCHDOG: Duration = Duration::from_secs(15);

struct Sandbox {
    root: PathBuf,
    executable: PathBuf,
}

impl Sandbox {
    fn new(scenario: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must follow epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "crystalline-p0100-b2-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".git/objects")).expect("create isolated repository envelope");
        fs::write(root.join(".scenario"), scenario).expect("write fixture scenario");

        let executable = root.join("hostile-git");
        fs::write(&executable, FIXTURE).expect("write controlled executable");
        let mut permissions = fs::metadata(&executable)
            .expect("stat fixture")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("make fixture executable");

        Self { root, executable }
    }

    fn invoke(
        &self,
    ) -> Result<crystalline_lint::infra::git_refinement::GitRevisionContent, GitRevisionError> {
        load_revision_with_git(
            &self.executable,
            &self.root,
            OsStr::new("main"),
            &[PathBuf::from("src/lib.rs")],
        )
    }

    fn recorded_pid(&self, name: &str) -> Option<u32> {
        fs::read_to_string(self.root.join(name))
            .ok()?
            .trim()
            .parse()
            .ok()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        for file in [".hostile-descendant-pid", ".hostile-parent-pid"] {
            if let Some(pid) = self.recorded_pid(file) {
                terminate_owned_pid(pid);
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn terminate_owned_pid(pid: u32) {
    let status_path = PathBuf::from(format!("/proc/{pid}/status"));
    if !status_path.exists() {
        return;
    }
    // Every PID considered here was written by this gate's private fixture. Avoid
    // process-name matching, process-group wildcards, or any repository-external PID.
    let _ = Command::new("/bin/kill")
        .args(["-KILL", &pid.to_string()])
        .status();
}

fn invoke_with_watchdog(
    sandbox: &Sandbox,
) -> (
    Result<crystalline_lint::infra::git_refinement::GitRevisionContent, GitRevisionError>,
    Duration,
) {
    let executable = sandbox.executable.clone();
    let root = sandbox.root.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    let started = Instant::now();
    thread::spawn(move || {
        let result = load_revision_with_git(
            &executable,
            &root,
            OsStr::new("main"),
            &[PathBuf::from("src/lib.rs")],
        );
        let _ = sender.send(result);
    });
    let result = receiver
        .recv_timeout(WATCHDOG)
        .expect("independent 15 s watchdog: adapter deadlocked beyond its 10 s deadline");
    (result, started.elapsed())
}

fn assert_pid_reaped(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(1);
    let proc_entry = PathBuf::from(format!("/proc/{pid}"));
    while proc_entry.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !proc_entry.exists(),
        "owned hostile PID {pid} was not reaped"
    );
}

#[test]
fn nonzero_status_is_process_failure_without_partial_publication() {
    let sandbox = Sandbox::new("status");
    assert_eq!(sandbox.invoke(), Err(GitRevisionError::MissingRef));
}

#[test]
fn closed_pipe_with_partial_stdout_is_invalid_framing() {
    let sandbox = Sandbox::new("partial");
    assert_eq!(sandbox.invoke(), Err(GitRevisionError::InvalidFraming));
}

#[test]
fn timeout_returns_only_after_the_hostile_process_is_reaped() {
    let sandbox = Sandbox::new("timeout");
    let (result, elapsed) = invoke_with_watchdog(&sandbox);
    assert_eq!(result, Err(GitRevisionError::Timeout));
    assert!(elapsed >= Duration::from_secs(9));
    assert!(elapsed < WATCHDOG);
    assert_pid_reaped(
        sandbox
            .recorded_pid(".hostile-parent-pid")
            .expect("fixture must record its PID"),
    );
}

#[test]
fn timeout_kills_and_reaps_a_hostile_descendant() {
    let sandbox = Sandbox::new("descendant");
    let (result, elapsed) = invoke_with_watchdog(&sandbox);
    assert_eq!(result, Err(GitRevisionError::Timeout));
    assert!(elapsed >= Duration::from_secs(9));
    assert!(elapsed < WATCHDOG);
    assert_pid_reaped(
        sandbox
            .recorded_pid(".hostile-descendant-pid")
            .expect("fixture must record descendant PID"),
    );
}
