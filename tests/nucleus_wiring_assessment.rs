use serde_json::Value;
use std::{path::PathBuf, process::Command};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nucleus_wiring")
}

fn run(checks: &str) -> (i32, Vec<String>) {
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(fixture())
        .args([".", "--checks", checks, "--format", "sarif"])
        .output()
        .unwrap();
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rules = json["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["ruleId"].as_str().unwrap().to_owned())
        .collect();
    (output.status.code().unwrap_or(-1), rules)
}

#[test]
fn shared_nucleus_does_not_break_prompt_bijection() {
    let (_, v15) = run("v15");
    assert!(v15.is_empty(), "{v15:?}");
    let (_, v26) = run("v26");
    assert!(v26.is_empty(), "{v26:?}");
}

#[test]
fn direct_code_reference_to_tekt_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("01_core")).unwrap();
    std::fs::create_dir_all(root.path().join("00_nucleo/prompts/_nuclei")).unwrap();
    std::fs::write(
        root.path().join("crystalline.toml"),
        "[project]\nroot='.'\n[layers]\nL0='00_nucleo'\nL1='01_core'\n",
    )
    .unwrap();
    std::fs::write(root.path().join("00_nucleo/prompts/_nuclei/x.tekt"), "tekt=1\nkind='nucleus'\nid='x'\ntitle='x'\n[[claims]]\nid='x'\nlevel='must'\nstatement='x'\n").unwrap();
    std::fs::write(root.path().join("01_core/x.rs"), "//! @prompt 00_nucleo/prompts/_nuclei/x.tekt\n//! @prompt-hash 00000000\n//! @layer L1\nfn x(){}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(root.path())
        .args([".", "--checks", "v1,v26", "--format", "sarif"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("V26"), "{text}");
}
