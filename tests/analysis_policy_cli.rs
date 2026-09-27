use serde_json::Value;
use std::fs;
use std::process::{Command, Output};

fn run(config: &str, args: &[&str]) -> Output {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::write(
        root.path().join("01_core/example.rs"),
        "pub fn example() {}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("crystalline.toml"),
        format!("[layers]\nL1 = '01_core'\n{config}"),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg(root.path())
        .args(args)
        .output()
        .unwrap()
}
fn results(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}
#[test]
fn unknown_checks_and_invalid_config_never_report_success() {
    for check in ["v999", "V1", "", "v1,", "v01"] {
        let output = run("", &["--checks", check]);
        assert!(!output.status.success(), "{check}");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("No violations"));
    }
    for config in [
        "[typo]\nx = true",
        "[rules]\nV99 = { level = 'error' }",
        "[rules]\nV1 = { level = 'typo' }",
        "[rules]\nV1 = { leve = 'info' }",
        "[rules]\nV1 = { languages = ['kotlin'] }",
        "[rules]\nV1 = { languages = [] }",
        "[rules]\nV0 = { level = 'info' }",
        "[rules]\nV8 = { languages = ['rust'] }",
        "[rules]\nV10 = { level = 'warning' }",
        "[rules]\nV7 = { languages = ['rust'] }",
        "[languages]\nrust = { enabled = false }",
        "[project]\nroot = 'other'",
    ] {
        let output = run(config, &["--checks", "v1"]);
        assert!(!output.status.success(), "accepted {config}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("config error"),
            "{config}"
        );
    }
}
#[test]
fn policy_controls_severity_language_and_exit() {
    let info = run(
        "[rules]\nV1 = { level = 'info' }",
        &["--checks", "v1", "--format", "sarif"],
    );
    assert!(info.status.success());
    assert_eq!(results(&info)["runs"][0]["results"][0]["level"], "note");
    let excluded = run(
        "[rules]\nV1 = { languages = ['python'] }",
        &["--checks", "v1", "--format", "sarif"],
    );
    assert!(excluded.status.success());
    assert_eq!(
        results(&excluded)["runs"][0]["results"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let warning = run(
        "[rules]\nV1 = { level = 'warning' }",
        &["--checks", "v1", "--fail-on", "warning"],
    );
    assert_eq!(warning.status.code(), Some(1));
}
#[test]
fn scope_is_honest_machine_readable_and_quiet_is_silent() {
    let config = "[rules]\nV1 = { level = 'info' }\n[excluded]\nbuild = 'target'";
    let output = run(
        config,
        &["--checks", "v1", "--analysis-scope", "--format", "sarif"],
    );
    let document = results(&output);
    let scope = &document["runs"][0]["properties"]["analysisScope"];
    assert_eq!(scope["filesParsed"], 1);
    assert_eq!(scope["languagesObserved"]["rust"], 1);
    assert_eq!(scope["selectedRules"], serde_json::json!(["V0", "V1"]));
    assert_eq!(
        scope["configuredExcludedDirectories"],
        serde_json::json!(["target"])
    );
    assert!(scope["limitations"].as_array().unwrap().len() >= 4);
    let quiet = run(config, &["--checks", "v1", "--analysis-scope", "--quiet"]);
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());
    let repair = run(config, &["--analysis-scope", "--fix-hashes", "--dry-run"]);
    assert!(!repair.status.success());
}
#[test]
fn explicit_missing_config_is_not_defaulted() {
    let output = run("", &["--config", "missing.toml"]);
    assert_eq!(output.status.code(), Some(2));
}
