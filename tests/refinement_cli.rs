use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/refinement")
        .join(name)
}

fn run(after: &str, format: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine",
            "--before",
            fixture("before.json").to_str().unwrap(),
            "--after",
            fixture(after).to_str().unwrap(),
            "--contract",
            fixture("contract.toml").to_str().unwrap(),
            "--format",
            format,
        ])
        .output()
        .unwrap()
}

#[test]
fn preserved_exits_zero() {
    let output = run("after-preserved.json", "text");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "PRESERVED\n");
}

#[test]
fn violated_exits_one_with_witness() {
    let output = run("after-violated.json", "text");
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("VIOLATED [font-identity:preserve]"));
    assert!(stdout.contains("known(wght=650)"));
    assert!(stdout.contains("known(default)"));
}

#[test]
fn unknown_exits_two() {
    let output = run("after-unknown.json", "text");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("UNKNOWN [font-identity:preserve]"));
}

#[test]
fn sarif_is_valid_and_uses_refinement_rule() {
    let output = run("after-violated.json", "sarif");
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["version"], "2.1.0");
    assert_eq!(value["runs"][0]["results"][0]["ruleId"], "REFINEMENT");
}
