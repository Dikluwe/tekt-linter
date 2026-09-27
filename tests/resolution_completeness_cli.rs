use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::write(
        root.path().join("crystalline.toml"),
        "[layers]\nL1 = '01_core'\n",
    )
    .unwrap();
    // Missing lineage is intentional: instrumentation is independent of lint violations.
    fs::write(
        root.path().join("01_core/good.rs"),
        "use std::collections::BTreeMap;\npub fn sample() {}\n",
    )
    .unwrap();
    root
}
#[test]
fn complete_resolution_is_json_lines_even_when_lint_would_fail() {
    let root = fixture();
    let out = run(root.path(), &["--emit-resolution"]);
    assert!(out.status.success(), "{out:?}");
    assert!(!out.stdout.is_empty());
    for line in String::from_utf8(out.stdout).unwrap().lines() {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
    assert!(out.stderr.is_empty());
}
#[test]
fn quiet_preserves_success_without_resolution_output() {
    let root = fixture();
    let out = run(root.path(), &["--emit-resolution", "--quiet"]);
    assert!(out.status.success());
    assert!(out.stdout.is_empty() && out.stderr.is_empty());
}
#[test]
fn incomplete_resolution_never_publishes_partial_output() {
    for bytes in [b"pub fn broken( {".as_slice(), b"", b"\xff"] {
        for quiet in [false, true] {
            let root = fixture();
            fs::write(root.path().join("01_core/broken.rs"), bytes).unwrap();
            let mut args = vec!["--emit-resolution", "--checks", "v0"];
            if quiet {
                args.push("--quiet");
            }
            let out = run(root.path(), &args);
            assert_eq!(out.status.code(), Some(2), "{out:?}");
            assert!(out.stdout.is_empty());
            if quiet {
                assert!(out.stderr.is_empty());
            } else {
                assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete analysis"));
            }
        }
    }
}
#[test]
fn resolution_rejects_repair_modes_before_running_analysis() {
    for mode in ["--fix-hashes", "--update-snapshot"] {
        for dry in [false, true] {
            let root = fixture();
            let path = root.path().join("01_core/good.rs");
            let before = fs::read(&path).unwrap();
            let mut args = vec!["--emit-resolution", mode];
            if dry {
                args.push("--dry-run");
            }
            let out = run(root.path(), &args);
            assert!(!out.status.success(), "{out:?}");
            assert!(out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).contains("--emit-resolution cannot"));
            assert_eq!(before, fs::read(path).unwrap());
        }
    }
}
