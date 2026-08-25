#![cfg(windows)]

use crystalline_lint::infra::git_refinement::{load_revision_with_git, GitRevisionError};
use std::ffi::{c_void, OsStr};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const WATCHDOG: Duration = Duration::from_secs(15);

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit_handle: i32, process_id: u32) -> *mut c_void;
    fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn GetCurrentProcess() -> *mut c_void;
    fn GetProcessHandleCount(process: *mut c_void, count: *mut u32) -> i32;
}

const SYNCHRONIZE: u32 = 0x0010_0000;
const WAIT_OBJECT_0: u32 = 0;

const HOSTILE_GIT: &str = r#"
use std::ffi::c_void;
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[repr(C)]
struct BasicLimit { per_process: i64, per_job: i64, flags: u32, min_ws: usize, max_ws: usize,
    active: u32, affinity: usize, priority: u32, scheduling: u32 }
#[repr(C)]
struct IoCounters { read_ops: u64, write_ops: u64, other_ops: u64, read_bytes: u64,
    write_bytes: u64, other_bytes: u64 }
#[repr(C)]
struct ExtendedLimit { basic: BasicLimit, io: IoCounters, process_memory: usize,
    job_memory: usize, peak_process: usize, peak_job: usize }

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn IsProcessInJob(process: *mut c_void, job: *mut c_void, result: *mut i32) -> i32;
    fn QueryInformationJobObject(job: *mut c_void, class: i32, info: *mut c_void,
        length: u32, returned: *mut u32) -> i32;
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.iter().any(|a| a == "--descendant") {
        thread::sleep(Duration::from_secs(60));
        return;
    }
    let root = std::env::current_dir().unwrap();
    let mut in_job = 0;
    let associated = unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut in_job) != 0 && in_job != 0 };
    let mut limits: ExtendedLimit = unsafe { std::mem::zeroed() };
    let queried = unsafe { QueryInformationJobObject(std::ptr::null_mut(), 9,
        &mut limits as *mut _ as *mut c_void, std::mem::size_of::<ExtendedLimit>() as u32,
        std::ptr::null_mut()) != 0 };
    let kill_on_close = queried && limits.basic.flags & 0x0000_2000 != 0;
    fs::write(root.join("job-state"), format!("associated={associated}\nkill_on_close={kill_on_close}\n")).unwrap();

    if !associated || !kill_on_close {
        // Hostile code was reached outside the required containment. This must be
        // classified as ContainmentFailure, never as an ordinary Git failure.
        std::process::exit(73);
    }
    let me = std::env::current_exe().unwrap();
    let child = Command::new(me).arg("--descendant").stdin(Stdio::null())
        .stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
    fs::write(root.join("descendant-pid"), child.id().to_string()).unwrap();
    fs::write(root.join("leader-pid"), std::process::id().to_string()).unwrap();
    if root.join("leader-exits").exists() { return; }
    thread::sleep(Duration::from_secs(60));
}
"#;

struct Harness {
    root: TempDir,
    executable: PathBuf,
}

impl Harness {
    fn new(leader_exits: bool) -> Self {
        let root = tempfile::tempdir().expect("create private Windows Job fixture");
        fs::create_dir_all(root.path().join(".git/objects")).unwrap();
        if leader_exits {
            fs::write(root.path().join("leader-exits"), b"").unwrap();
        }
        let source = root.path().join("hostile_git.rs");
        let executable = root.path().join("hostile-git.exe");
        fs::write(&source, HOSTILE_GIT).unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        assert!(Command::new(rustc)
            .args(["--edition=2021", "-o"])
            .arg(&executable)
            .arg(&source)
            .status()
            .expect("compile private hostile executable")
            .success());
        Self { root, executable }
    }

    fn invoke(
        &self,
    ) -> (
        Result<crystalline_lint::infra::git_refinement::GitRevisionContent, GitRevisionError>,
        Duration,
    ) {
        let executable = self.executable.clone();
        let repository = self.root.path().to_path_buf();
        let (send, receive) = mpsc::sync_channel(1);
        let started = Instant::now();
        thread::spawn(move || {
            let result = load_revision_with_git(
                &executable,
                &repository,
                OsStr::new("main"),
                &[PathBuf::from("src/lib.rs")],
            );
            let _ = send.send(result);
        });
        let result = receive
            .recv_timeout(WATCHDOG)
            .expect("Windows Job gate exceeded its independent 15 second watchdog");
        (result, started.elapsed())
    }

    fn pid(&self, name: &str) -> u32 {
        fs::read_to_string(self.root.path().join(name))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
}

fn assert_process_terminated(pid: u32) {
    let handle = unsafe { OpenProcess(SYNCHRONIZE, 0, pid) };
    if handle.is_null() {
        return;
    }
    let wait = unsafe { WaitForSingleObject(handle, 1_000) };
    unsafe { CloseHandle(handle) };
    assert_eq!(wait, WAIT_OBJECT_0, "owned hostile PID {pid} remains alive");
}

fn handle_count() -> u32 {
    let mut count = 0;
    assert_ne!(
        unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) },
        0
    );
    count
}

fn assert_job_contract(root: &Path) {
    assert_eq!(
        fs::read_to_string(root.join("job-state")).unwrap(),
        "associated=true\nkill_on_close=true\n"
    );
}

#[test]
fn timeout_kills_leader_and_pipe_independent_descendant_and_closes_handles() {
    let before = handle_count();
    let harness = Harness::new(false);
    let (result, elapsed) = harness.invoke();
    assert_eq!(result, Err(GitRevisionError::Timeout));
    assert!(elapsed < WATCHDOG);
    assert_job_contract(harness.root.path());
    assert_process_terminated(harness.pid("leader-pid"));
    assert_process_terminated(harness.pid("descendant-pid"));
    let after = handle_count();
    assert!(
        after <= before + 2,
        "Job/process handles leaked: before={before}, after={after}"
    );
}

#[test]
fn leader_exit_cannot_leave_a_descendant_alive_or_block_the_caller() {
    let harness = Harness::new(true);
    let (result, elapsed) = harness.invoke();
    assert!(
        result.is_err(),
        "incomplete Git framing must not be published"
    );
    assert!(elapsed < WATCHDOG);
    assert_job_contract(harness.root.path());
    assert_process_terminated(harness.pid("descendant-pid"));
}
