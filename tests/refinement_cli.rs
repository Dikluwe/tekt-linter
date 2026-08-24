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

#[test]
fn snapshot_is_deterministic_and_refines_against_itself() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.json");
    let second = dir.path().join("second.json");
    for output in [&first, &second] {
        let result = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "snapshot",
                fixture("project").to_str().unwrap(),
                "--contract",
                fixture("snapshot-contract.toml").to_str().unwrap(),
                "--artifact-id",
                "fixture",
                "--output",
                output.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(0));
    }
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );

    let result = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine",
            "--before",
            first.to_str().unwrap(),
            "--after",
            second.to_str().unwrap(),
            "--contract",
            fixture("snapshot-contract.toml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(String::from_utf8(result.stdout).unwrap(), "PRESERVED\n");
}

fn snapshot(project: &std::path::Path, contract: &std::path::Path, output: &std::path::Path) {
    let result = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "snapshot",
            project.to_str().unwrap(),
            "--contract",
            contract.to_str().unwrap(),
            "--artifact-id",
            project.file_name().unwrap().to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
}

#[test]
fn historical_oracles_accept_fix_and_reject_regression() {
    for name in ["context", "field", "authority"] {
        let root = fixture("oracles").join(name);
        let contract = root.join("contract.toml");
        let dir = tempfile::tempdir().unwrap();
        let before = dir.path().join("before.json");
        let after = dir.path().join("after.json");
        snapshot(&root.join("before"), &contract, &before);
        snapshot(&root.join("after"), &contract, &after);

        let fixed = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "refine",
                "--before",
                before.to_str().unwrap(),
                "--after",
                after.to_str().unwrap(),
                "--contract",
                contract.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(fixed.status.code(), Some(0), "fix oracle {name}");

        let regression = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "refine",
                "--before",
                after.to_str().unwrap(),
                "--after",
                before.to_str().unwrap(),
                "--contract",
                contract.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert_eq!(
            regression.status.code(),
            Some(1),
            "regression oracle {name}"
        );
    }
}
