use serde_json::Value;
use std::fs;
use std::process::{Command, Output};

fn run(config: &str, args: &[&str]) -> Output {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::create_dir_all(root.path().join("00_nucleo/prompts")).unwrap();
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

#[test]
fn project_rule_switches_override_selection_and_report_scope() {
    let disabled = run(
        "[rules]\nV1 = { enabled = false }",
        &["--checks", "v1", "--format", "sarif"],
    );
    assert!(disabled.status.success(), "{:?}", disabled);
    let doc = results(&disabled);
    assert!(doc["runs"][0]["results"].as_array().unwrap().is_empty());
    let scope = &doc["runs"][0]["properties"]["analysisScope"];
    assert_eq!(scope["selectedRules"], serde_json::json!(["V0"]));
    assert!(scope["disabledRules"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("V1")));
    assert_eq!(scope["configuredRulePolicy"]["V1"]["enabled"], false);
    let enabled = run(
        "[rules]\nV1 = { enabled = true }",
        &["--checks", "v2", "--format", "sarif"],
    );
    assert_eq!(enabled.status.code(), Some(1));
    assert!(results(&enabled)["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["ruleId"] == "V1"));
    let quiet = run(
        "[rules]\nV1 = { enabled = false }",
        &["--checks", "v1", "--quiet"],
    );
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());
}

#[test]
fn every_rule_accepts_individual_switches_but_invalid_config_still_fails() {
    for id in 0..=27 {
        for enabled in [false, true] {
            let output = run(
                &format!("[rules]\nV{id} = {{ enabled = {enabled} }}"),
                &["--checks", "v0", "--format", "sarif"],
            );
            assert!(
                !String::from_utf8_lossy(&output.stderr).contains("config error"),
                "{id}: {:?}",
                output
            );
            let doc = results(&output);
            let selected = doc["runs"][0]["properties"]["analysisScope"]["selectedRules"]
                .as_array()
                .unwrap();
            assert_eq!(
                selected.contains(&serde_json::json!(format!("V{id}"))),
                enabled,
                "V{id}"
            );
        }
    }
    for config in [
        "[rules]\nV1 = { enabled = 'false' }",
        "[rules]\nV28 = { enabled = false }",
        "[rules]\nV1 = { enabled = false, level = 'typo' }",
    ] {
        let output = run(config, &["--checks", "v1"]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config error"));
    }
}

#[test]
fn final_disable_flags_override_project_enabling() {
    let output = run(
        "[rules]\nV5 = { enabled = true }\nV6 = { enabled = true }",
        &[
            "--checks",
            "v0",
            "--no-drift",
            "--no-stale",
            "--format",
            "sarif",
        ],
    );
    assert!(output.status.success());
    assert_eq!(
        results(&output)["runs"][0]["properties"]["analysisScope"]["selectedRules"],
        serde_json::json!(["V0"])
    );
}

#[test]
fn fatal_architecture_rules_can_be_disabled_and_reenabled() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::create_dir(root.path().join("lab")).unwrap();
    fs::write(
        root.path().join("01_core/use_lab.rs"),
        "use crate::lab::experiment;\npub fn example() { experiment(); }\n",
    )
    .unwrap();
    fs::write(root.path().join("rogue.rs"), "pub fn rogue() {}\n").unwrap();
    for id in [8, 10] {
        for enabled in [false, true] {
            fs::write(root.path().join("crystalline.toml"), format!("[layers]\nL1 = '01_core'\nlab = 'lab'\n[module_layers]\nlab = 'lab'\n[rules]\nV{id} = {{ enabled = {enabled} }}\n")).unwrap();
            let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
                .arg(root.path())
                .args(["--checks", &format!("v{id}"), "--format", "sarif"])
                .output()
                .unwrap();
            assert_eq!(output.status.success(), !enabled, "V{id}: {:?}", output);
            let doc = results(&output);
            assert_eq!(
                doc["runs"][0]["results"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v["ruleId"] == format!("V{id}")),
                enabled
            );
        }
    }
}

#[test]
fn disabling_v0_does_not_turn_incomplete_analysis_into_success_or_allow_repairs() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::create_dir_all(root.path().join("00_nucleo/prompts/not-a-file.md")).unwrap();
    let source = root.path().join("01_core/example.rs");
    let code = "//! @prompt 00_nucleo/prompts/not-a-file.md\n//! @prompt-hash 12345678\n//! @layer L1\npub fn example() {}\n";
    fs::write(&source, code).unwrap();
    fs::write(root.path().join("crystalline.toml"), "[layers]\nL1='01_core'\n[rules]\nV0={enabled=false}\nV5={enabled=false}\nV6={enabled=false}\nV26={enabled=false}\n").unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .arg(root.path())
            .args(["--checks", "v0"])
            .args(extra)
            .output()
            .unwrap()
    };
    let output = run(&["--format", "sarif"]);
    assert_eq!(output.status.code(), Some(2), "{:?}", output);
    let doc = results(&output);
    assert!(doc["runs"][0]["results"].as_array().unwrap().is_empty());
    assert_eq!(
        doc["runs"][0]["properties"]["analysisScope"]["analysisComplete"],
        false
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("incomplete analysis"));
    for action in ["--fix-hashes", "--update-snapshot"] {
        let output = run(&[action, "--dry-run"]);
        assert_eq!(output.status.code(), Some(2), "{:?}", output);
        assert_eq!(fs::read_to_string(&source).unwrap(), code);
    }
    let output = run(&["--quiet"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}
