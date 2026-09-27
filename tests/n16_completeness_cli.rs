use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(root: &Path, quiet: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"));
    command
        .arg(root)
        .args(["--checks", "v16", "--format", "n16-summary"]);
    if quiet {
        command.arg("--quiet");
    }
    command.output().unwrap()
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::write(
        root.path().join("crystalline.toml"),
        "[layers]\nL1 = '01_core'\n",
    )
    .unwrap();
    root
}
#[test]
fn incomplete_sources_never_produce_n16_statistics() {
    for bytes in [b"pub fn broken( {".as_slice(), b"", b"\xff"] {
        for quiet in [false, true] {
            let root = fixture();
            fs::write(root.path().join("01_core/good.rs"), "pub fn good() {}\n").unwrap();
            fs::write(root.path().join("01_core/broken.rs"), bytes).unwrap();
            let out = run(root.path(), quiet);
            assert_eq!(out.status.code(), Some(2), "{out:?}");
            assert!(out.stdout.is_empty(), "partial summary: {out:?}");
            if quiet {
                assert!(out.stderr.is_empty());
            } else {
                let errors = String::from_utf8_lossy(&out.stderr);
                assert!(errors.contains("broken.rs"), "{errors}");
                assert!(
                    errors.contains("V0") || errors.contains("PARSE"),
                    "{errors}"
                );
            }
        }
    }
}
#[test]
fn empty_project_and_completely_parsed_sources_still_produce_summaries() {
    for with_source in [false, true] {
        let root = fixture();
        if with_source {
            fs::write(root.path().join("01_core/good.rs"), "pub fn good() {}\n").unwrap();
        }
        let out = run(root.path(), false);
        assert!(out.status.success(), "{out:?}");
        assert!(!out.stdout.is_empty());
        assert!(out.stderr.is_empty());
        let silent = run(root.path(), true);
        assert!(silent.status.success());
        assert!(silent.stdout.is_empty() && silent.stderr.is_empty());
    }
}
