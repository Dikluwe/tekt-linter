#![cfg(unix)]

use crystalline_lint::infra::git_refinement::{
    load_revision_with_git, GitPathContent, GitRevisionError, GitUnknownReason,
};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const WATCHDOG: Duration = Duration::from_secs(15);
const FIXTURE: &str = include_str!("fixtures/git_refinement_stream_lifecycle/fake-git.sh");

struct Scenario {
    root: PathBuf,
    executable: PathBuf,
}

impl Scenario {
    fn new(marker: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "crystalline-p0101-b1-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".git/objects/info")).expect("create object info");
        fs::create_dir_all(root.join(".git/objects/pack")).expect("create object pack");
        fs::write(root.join(marker), b"").expect("write scenario marker");
        let executable = root.join("fake-git");
        fs::write(&executable, FIXTURE).expect("write controlled executable");
        let mut permissions = fs::metadata(&executable)
            .expect("fixture metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("make fixture executable");
        Self { root, executable }
    }

    fn invoke(
        &self,
    ) -> (
        Result<crystalline_lint::infra::git_refinement::GitRevisionContent, GitRevisionError>,
        Duration,
    ) {
        let executable = self.executable.clone();
        let root = self.root.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let started = Instant::now();
            let result = load_revision_with_git(
                &executable,
                &root,
                OsStr::new("main"),
                &[PathBuf::from("src/lib.rs")],
            );
            let _ = sender.send((result, started.elapsed()));
        });
        receiver
            .recv_timeout(WATCHDOG)
            .expect("adapter exceeded the external 15 second watchdog")
    }

    fn descendant_pid(&self) -> u32 {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Ok(value) = fs::read_to_string(self.root.join("descendant.pid")) {
                return value.trim().parse().expect("fixture descendant PID");
            }
            assert!(
                Instant::now() < deadline,
                "fixture did not publish descendant PID"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Scenario {
    fn drop(&mut self) {
        if let Ok(value) = fs::read_to_string(self.root.join("descendant.pid")) {
            if let Ok(pid) = value.trim().parse::<u32>() {
                let proc_cwd = Path::new("/proc").join(pid.to_string()).join("cwd");
                let belongs_to_fixture = fs::read_link(proc_cwd)
                    .map(|cwd| cwd == self.root)
                    .unwrap_or(true);
                if belongs_to_fixture {
                    let _ = std::process::Command::new("/bin/kill")
                        .args(["-KILL", &pid.to_string()])
                        .status();
                }
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn pid_is_alive(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
        || std::process::Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
}

fn assert_pid_terminated(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if !pid_is_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !pid_is_alive(pid),
        "fixture descendant {pid} survived containment"
    );
}

#[test]
fn oversized_header_with_open_pipe_is_budget_exhausted_before_deadline_and_reaped() {
    let scenario = Scenario::new("scenario-oversized");
    let (result, elapsed) = scenario.invoke();
    let descendant = scenario.descendant_pid();

    let revision = result.expect("declared blob overflow is a typed path uncertainty");
    assert_eq!(
        revision.paths.get(Path::new("src/lib.rs")),
        Some(&GitPathContent::Unknown(GitUnknownReason::BudgetExhausted))
    );
    assert!(
        elapsed < Duration::from_secs(10),
        "overflow waited for the Git deadline: {elapsed:?}"
    );
    assert_pid_terminated(descendant);
}

#[test]
fn leader_exit_with_descendant_holding_pipes_returns_bounded_and_reaps_containment() {
    let scenario = Scenario::new("scenario-partial-pipes");
    let (result, elapsed) = scenario.invoke();
    let descendant = scenario.descendant_pid();

    assert!(
        matches!(
            result,
            Err(GitRevisionError::InvalidFraming | GitRevisionError::ContainmentFailure)
        ),
        "partial framing must fail closed after lifecycle completion: {result:?}"
    );
    assert!(
        elapsed < WATCHDOG,
        "reader joins escaped the watchdog: {elapsed:?}"
    );
    assert_pid_terminated(descendant);
}

#[test]
fn descendant_without_inherited_pipes_still_prevents_content_publication() {
    let scenario = Scenario::new("scenario-detached-pipes");
    let (result, elapsed) = scenario.invoke();
    let descendant = scenario.descendant_pid();

    assert_eq!(result, Err(GitRevisionError::ContainmentFailure));
    assert!(
        elapsed < WATCHDOG,
        "containment proof escaped the watchdog: {elapsed:?}"
    );
    assert_pid_terminated(descendant);
}

#[test]
fn transcript_cap_with_open_pipe_is_invalid_framing_before_timeout() {
    let scenario = Scenario::new("scenario-transcript-cap");
    let (result, elapsed) = scenario.invoke();
    let descendant = scenario.descendant_pid();

    assert_eq!(result, Err(GitRevisionError::InvalidFraming));
    assert!(
        elapsed < Duration::from_secs(10),
        "transcript cap became timeout: {elapsed:?}"
    );
    assert_pid_terminated(descendant);
}
