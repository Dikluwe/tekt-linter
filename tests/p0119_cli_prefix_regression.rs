use std::fs;
use std::process::Command;

#[test]
fn lint_flags_before_init_are_rejected_without_writing() {
    for prefix in [
        &["--config", "elsewhere.toml"][..],
        &["--config", "crystalline.toml"][..],
        &["--fix-hashes"][..],
        &["--update-snapshot"][..],
        &["--dry-run"][..],
        &["--quiet"][..],
        &["--format", "sarif"][..],
        &["--format", "text"][..],
        &["--fail-on", "error"][..],
        &["--checks", "v1"][..],
        &[
            "--checks",
            "v1,v2,v3,v4,v5,v6,v7,v8,v9,v10,v11,v12,v13,v14,v15,v16,v17,v18,v19,v20,v21,v23,v24,v25,v26",
        ][..],
    ] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("00_nucleo")).unwrap();
        fs::create_dir(root.path().join("01_core")).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"));
        command.args(prefix).arg("init").arg(root.path());
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(2), "prefix={prefix:?}");
        assert!(!root.path().join("crystalline.toml").exists());
    }
}
