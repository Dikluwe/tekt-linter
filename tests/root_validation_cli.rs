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
#[test]
fn invalid_roots_always_produce_sarif_v0_independent_of_checks() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("file.rs");
    fs::write(&file, "pub fn sample() {}\n").unwrap();
    for root in [file, temp.path().join("missing")] {
        for checks in ["v0", "v16", "v1,v5"] {
            let out = run(&root, &["--checks", checks, "--format", "sarif"]);
            assert_eq!(out.status.code(), Some(1), "{out:?}");
            let json: serde_json::Value =
                serde_json::from_slice(&out.stdout).expect("SARIF on root failure");
            let findings = json["runs"][0]["results"].as_array().unwrap();
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0]["ruleId"], "V0");
            assert!(out.stderr.is_empty());
            let silent = run(&root, &["--checks", checks, "--quiet"]);
            assert_eq!(silent.status.code(), Some(1));
            assert!(silent.stdout.is_empty() && silent.stderr.is_empty());
        }
    }
}
#[test]
fn invalid_resolution_root_preserves_jsonl_channel_and_quiet() {
    let temp = tempfile::tempdir().unwrap();
    for quiet in [false, true] {
        let mut args = vec!["--emit-resolution", "--checks", "v0"];
        if quiet {
            args.push("--quiet");
        }
        let out = run(&temp.path().join("missing"), &args);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert_eq!(out.stderr.is_empty(), quiet);
    }
}
#[test]
fn n16_invalid_root_remains_a_normal_error_with_only_v16_selected() {
    let temp = tempfile::tempdir().unwrap();
    let out = run(
        &temp.path().join("missing"),
        &["--checks", "v16", "--format", "n16-summary"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("V0"));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
}
#[test]
fn existing_empty_root_is_valid_and_missing_explicit_config_still_fails() {
    let temp = tempfile::tempdir().unwrap();
    let normal = run(temp.path(), &["--checks", "v0", "--format", "sarif"]);
    assert!(normal.status.success(), "{normal:?}");
    let json: serde_json::Value = serde_json::from_slice(&normal.stdout).unwrap();
    assert!(json["runs"][0]["results"].as_array().unwrap().is_empty());
    let config = run(temp.path(), &["--config", "missing.toml"]);
    assert_eq!(config.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&config.stderr).contains("explicit config"));
}
