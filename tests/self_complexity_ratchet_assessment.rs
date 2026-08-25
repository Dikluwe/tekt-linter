use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const MANIFEST: &str = include_str!("../00_nucleo/assessments/0037-self-complexity.tsv");

fn normalized(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn identity(rule: &str, path: &str, message: &str) -> String {
    let path = path.strip_prefix("./").unwrap_or(path);
    let evidence = format!("{rule}|./{path}|{} [{rule}]", normalized(message));
    hex::encode(Sha256::digest(evidence.as_bytes()))
}

fn accepted_baseline() -> BTreeMap<(String, String, String), usize> {
    let mut baseline = BTreeMap::new();
    for line in MANIFEST.lines().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 12, "invalid manifest row: {line}");
        if fields[8].starts_with("ACCEPT-") {
            let key = (
                fields[1].to_owned(),
                fields[3].to_owned(),
                fields[5].to_owned(),
            );
            *baseline.entry(key).or_insert(0) += 1;
        }
    }
    baseline
}

fn current_findings() -> BTreeMap<(String, String, String), usize> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(&root)
        .args([
            ".",
            "--checks",
            "v16,v17,v19,v20",
            "--format",
            "sarif",
            "--fail-on",
            "error",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "self-lint failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut findings = BTreeMap::new();
    for result in document["runs"][0]["results"].as_array().unwrap() {
        let rule = result["ruleId"].as_str().unwrap();
        assert!(
            !matches!(rule, "V16" | "V17"),
            "actionable finding: {result}"
        );
        assert!(matches!(rule, "V19" | "V20"), "unexpected rule: {result}");
        let path = result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        let message = result["message"]["text"].as_str().unwrap();
        let clean_path = path.strip_prefix("./").unwrap_or(path).to_owned();
        let key = (rule.to_owned(), clean_path, identity(rule, path, message));
        *findings.entry(key).or_insert(0) += 1;
    }
    findings
}

#[test]
fn accepted_complexity_is_exact_and_has_no_actionable_regression() {
    assert_eq!(current_findings(), accepted_baseline());
}

#[test]
fn identity_ignores_lines_and_presentation_whitespace() {
    let compact = "Braço condensa (`A | B`)";
    let formatted = "Braço  condensa   (`A | B`)";
    assert_eq!(
        identity("V19", "./01_core/example.rs", compact),
        identity("V19", "01_core/example.rs", formatted)
    );
}
