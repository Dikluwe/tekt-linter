use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const PROMPT: &str = "00_nucleo/prompts/shared.md";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    rule: String,
    message: String,
    uri: String,
    line: u64,
    column: u64,
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prompt_ownership_wiring")
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn project() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    copy_tree(&fixture_root(), root.path());
    root
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn rust_source(name: &str) -> String {
    format!(
        "//! Crystalline Lineage\n//! @prompt {PROMPT}\n//! @prompt-hash 00000000\n//! @layer L1\n\npub fn {name}() {{}}\n"
    )
}

fn language_source(extension: &str, name: &str) -> String {
    let (prefix, body) = match extension {
        "rs" => ("//!", format!("pub fn {name}() {{}}")),
        "ts" | "tsx" => ("//", format!("export function {name}(): void {{}}")),
        "py" => ("#", format!("def {name}():\n    pass")),
        "c" | "h" => ("//", format!("void {name}(void) {{}}")),
        "cpp" | "hpp" | "cc" | "cxx" | "hxx" => ("//", format!("void {name}() {{}}")),
        "zig" => ("//!", format!("pub fn {name}() void {{}}")),
        "go" => ("//", format!("package core\n\nfunc {name}() {{}}")),
        "java" => ("//", format!("class {name} {{}}")),
        "ex" | "exs" => ("#", format!("defmodule {name} do\nend")),
        other => panic!("unsupported fixture extension: {other}"),
    };
    format!(
        "{prefix} Crystalline Lineage\n{prefix} @prompt {PROMPT}\n{prefix} @prompt-hash 00000000\n{prefix} @layer L1\n\n{body}\n"
    )
}

fn run(root: &Path, checks: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(root)
        .args([
            ".",
            "--checks",
            checks,
            "--format",
            "sarif",
            "--fail-on",
            "error",
        ])
        .output()
        .unwrap()
}

fn findings(output: &Output) -> Vec<Finding> {
    let document: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "invalid SARIF ({error}); status={:?}; stdout={}; stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    document["runs"][0]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| Finding {
            rule: result["ruleId"].as_str().unwrap().to_owned(),
            message: result["message"]["text"].as_str().unwrap().to_owned(),
            uri: result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .unwrap()
                .to_owned(),
            line: result["locations"][0]["physicalLocation"]["region"]["startLine"]
                .as_u64()
                .unwrap(),
            column: result["locations"][0]["physicalLocation"]["region"]["startColumn"]
                .as_u64()
                .unwrap(),
        })
        .collect()
}

fn only_rule<'a>(all: &'a [Finding], rule: &str) -> Vec<&'a Finding> {
    all.iter().filter(|finding| finding.rule == rule).collect()
}

#[test]
fn real_binary_reports_one_global_v15_with_sorted_complete_consumers() {
    let root = project();
    write(root.path(), "01_core/zeta.rs", &rust_source("zeta"));
    write(root.path(), "01_core/alpha.rs", &rust_source("alpha"));

    let output = run(root.path(), "v7,v15");
    let all = findings(&output);
    assert!(only_rule(&all, "V7").is_empty(), "findings: {all:#?}");
    let v15 = only_rule(&all, "V15");
    assert_eq!(v15.len(), 1, "findings: {all:#?}");
    let finding = v15[0];
    assert!(finding.message.contains(PROMPT), "{finding:#?}");
    assert!(finding.message.contains('2'), "{finding:#?}");
    let alpha = finding.message.find("01_core/alpha.rs").unwrap();
    let zeta = finding.message.find("01_core/zeta.rs").unwrap();
    assert!(alpha < zeta, "{finding:#?}");
    assert!(finding.uri.ends_with("01_core/alpha.rs"), "{finding:#?}");
    assert_eq!((finding.line, finding.column), (1, 1));
    assert!(!output.status.success(), "V15 Error must fail the command");
}

#[test]
fn diagnosis_is_independent_of_creation_order() {
    fn evaluate(order: [&str; 2]) -> Vec<Finding> {
        let root = project();
        for relative in order {
            let name = Path::new(relative).file_stem().unwrap().to_str().unwrap();
            write(root.path(), relative, &rust_source(name));
        }
        findings(&run(root.path(), "v15"))
    }

    assert_eq!(
        evaluate(["01_core/zeta.rs", "01_core/alpha.rs"]),
        evaluate(["01_core/alpha.rs", "01_core/zeta.rs"])
    );
}

#[test]
fn missing_header_remains_v1_not_v15() {
    let missing = project();
    write(
        missing.path(),
        "01_core/missing.rs",
        "pub fn missing() {}\n",
    );
    let missing_output = run(missing.path(), "v1,v15");
    let missing_findings = findings(&missing_output);
    assert_eq!(
        only_rule(&missing_findings, "V1").len(),
        1,
        "{missing_findings:#?}"
    );
    assert!(
        only_rule(&missing_findings, "V15").is_empty(),
        "{missing_findings:#?}"
    );

}

#[test]
fn test_and_non_productive_origins_do_not_create_phantom_ownership() {
    let root = project();
    write(root.path(), "01_core/owner.rs", &rust_source("owner"));
    write(root.path(), "tests/ghost.rs", &rust_source("test_ghost"));
    write(root.path(), "05_lab/ghost.rs", &rust_source("lab_ghost"));

    let output = run(root.path(), "v15");
    let all = findings(&output);
    assert!(only_rule(&all, "V15").is_empty(), "{all:#?}");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn configured_exclusion_does_not_create_phantom_ownership() {
    let root = project();
    write(root.path(), "01_core/owner.rs", &rust_source("owner"));
    write(root.path(), "01_core/excluded.rs", "pub fn excluded() {}\n");

    let output = run(root.path(), "v1,v15");
    let all = findings(&output);
    assert!(only_rule(&all, "V1").is_empty(), "{all:#?}");
    assert!(only_rule(&all, "V15").is_empty(), "{all:#?}");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn every_available_parser_publishes_a_canonical_prompt_header() {
    for extension in [
        "rs", "ts", "tsx", "py", "c", "h", "cpp", "hpp", "cc", "cxx", "hxx", "zig", "go", "java",
        "ex", "exs",
    ] {
        let root = project();
        write(
            root.path(),
            &format!("01_core/alpha.{extension}"),
            &language_source(extension, "Alpha"),
        );
        write(
            root.path(),
            &format!("01_core/zeta.{extension}"),
            &language_source(extension, "Zeta"),
        );

        let parsed = findings(&run(root.path(), "v1"));
        assert!(
            only_rule(&parsed, "V1").is_empty(),
            "{extension} did not publish a valid canonical prompt header: {parsed:#?}"
        );
    }
}

#[test]
fn every_available_parser_has_the_same_global_ownership_semantics() {
    for extension in [
        "rs", "ts", "tsx", "py", "c", "h", "cpp", "hpp", "cc", "cxx", "hxx", "zig", "go", "java",
        "ex", "exs",
    ] {
        let root = project();
        write(
            root.path(),
            &format!("01_core/alpha.{extension}"),
            &language_source(extension, "Alpha"),
        );
        write(
            root.path(),
            &format!("01_core/zeta.{extension}"),
            &language_source(extension, "Zeta"),
        );

        let ownership = findings(&run(root.path(), "v15"));
        let v15 = only_rule(&ownership, "V15");
        assert_eq!(
            v15.len(),
            1,
            "{extension} did not participate exactly once: {ownership:#?}"
        );
        assert!(v15[0].message.contains(&format!("alpha.{extension}")));
        assert!(v15[0].message.contains(&format!("zeta.{extension}")));
    }
}
