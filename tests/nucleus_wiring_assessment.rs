use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nucleus_wiring")
}

fn run(checks: &str) -> (i32, Vec<String>) {
    run_at(&fixture(), checks)
}

fn run_at(root: &Path, checks: &str) -> (i32, Vec<String>) {
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(root)
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

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
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

#[test]
fn one_nucleus_byte_invalidates_both_pins_and_both_code_hashes() {
    let temp = tempfile::tempdir().unwrap();
    copy_tree(&fixture(), temp.path());
    let path = temp.path().join("00_nucleo/prompts/_nuclei/path.tekt");
    let changed = std::fs::read_to_string(&path)
        .unwrap()
        .replace("logical identity", "logical identity!");
    std::fs::write(path, changed).unwrap();
    let (_, rules) = run_at(temp.path(), "v5,v26");
    assert_eq!(
        rules.iter().filter(|id| *id == "V5").count(),
        2,
        "{rules:?}"
    );
    assert_eq!(
        rules.iter().filter(|id| *id == "V26").count(),
        2,
        "{rules:?}"
    );
}
