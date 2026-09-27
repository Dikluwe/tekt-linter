use crystalline_lint::{
    contracts::parse_error::ParseError,
    entities::{
        analysis_policy::{repair_is_blocked, ParseCounts},
        layer::Language,
    },
};
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
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::create_dir_all(root.path().join("00_nucleo/prompts")).unwrap();
    fs::write(
        root.path().join("crystalline.toml"),
        "[layers]\nL0 = '00_nucleo'\nL1 = '01_core'\n",
    )
    .unwrap();
    fs::write(
        root.path().join("00_nucleo/prompts/example.md"),
        "# Example\nHash do Código: 00000000\n\nOwner.\n",
    )
    .unwrap();
    fs::write(root.path().join("01_core/example.rs"), "//! Crystalline Lineage\n//! @prompt 00_nucleo/prompts/example.md\n//! @prompt-hash 00000000\n//! @layer L1\n//! @updated 2026-09-26\npub fn example() -> usize { 1 }\n").unwrap();
    root
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn incomplete_analysis_blocks_both_repairs_before_any_write() {
    for body in ["pub fn broken( {", ""] {
        for mode in ["--fix-hashes", "--update-snapshot"] {
            for dry in [false, true] {
                let root = fixture();
                let source = root.path().join("01_core/example.rs");
                let prompt = root.path().join("00_nucleo/prompts/example.md");
                let before = (fs::read(&source).unwrap(), fs::read(&prompt).unwrap());
                fs::write(root.path().join("01_core/broken.rs"), body).unwrap();
                let mut args = vec!["--checks", "v0", mode];
                if dry {
                    args.push("--dry-run");
                }
                let out = run(root.path(), &args);
                assert_eq!(out.status.code(), Some(2), "{mode}: {out:?}");
                assert!(String::from_utf8_lossy(&out.stderr).contains("incomplete analysis"));
                assert_eq!(
                    before,
                    (fs::read(&source).unwrap(), fs::read(&prompt).unwrap())
                );
            }
        }
    }
}
#[test]
fn reverse_only_drift_is_repaired_and_idempotent() {
    let root = fixture();
    success(&run(root.path(), &["--fix-hashes", "--checks", "v5"]));
    let source = root.path().join("01_core/example.rs");
    let prompt = root.path().join("00_nucleo/prompts/example.md");
    fs::write(
        &source,
        fs::read_to_string(&source)
            .unwrap()
            .replace("{ 1 }", "{ 2 }"),
    )
    .unwrap();
    let before = (fs::read(&source).unwrap(), fs::read(&prompt).unwrap());
    success(&run(root.path(), &["--checks", "v5"])); // forward digest still matches
    let dry = run(
        root.path(),
        &["--fix-hashes", "--dry-run", "--checks", "v0"],
    );
    success(&dry);
    assert!(!String::from_utf8_lossy(&dry.stdout).contains("Nothing to fix"));
    assert_eq!(
        before,
        (fs::read(&source).unwrap(), fs::read(&prompt).unwrap())
    );
    success(&run(root.path(), &["--fix-hashes", "--checks", "v0"]));
    assert_ne!(before.1, fs::read(&prompt).unwrap());
    assert!(fs::read_to_string(&source).unwrap().contains("{ 2 }"));
    let final_run = run(
        root.path(),
        &["--fix-hashes", "--dry-run", "--checks", "v5"],
    );
    success(&final_run);
    assert!(String::from_utf8_lossy(&final_run.stdout).contains("Nothing to fix"));
}
#[test]
fn invalid_reverse_metadata_blocks_the_entire_batch() {
    for metadata in [
        "",
        "Hash do Código: invalid\n",
        "Hash do Código: 00000000\nHash do Código: 11111111\n",
    ] {
        let root = fixture();
        let source = root.path().join("01_core/example.rs");
        let before = fs::read(&source).unwrap();
        let prompt = root.path().join("00_nucleo/prompts/example.md");
        fs::write(&prompt, format!("# Example\n{metadata}\nOwner.\n")).unwrap();
        let prompt_before = fs::read(&prompt).unwrap();
        let out = run(root.path(), &["--fix-hashes", "--checks", "v0"]);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(before, fs::read(&source).unwrap());
        assert_eq!(prompt_before, fs::read(&prompt).unwrap());
    }
}
#[test]
fn nonexistent_root_in_n16_is_an_error_without_panic() {
    let root = tempfile::tempdir().unwrap();
    for quiet in [false, true] {
        let mut args = vec!["--format", "n16-summary"];
        if quiet {
            args.push("--quiet");
        }
        let out = run(&root.path().join("missing"), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
        if quiet {
            assert!(out.stdout.is_empty() && out.stderr.is_empty());
        }
    }
}
#[test]
fn scope_reports_typed_parser_failures() {
    let root = fixture();
    fs::write(root.path().join("01_core/broken.rs"), "pub fn broken( {").unwrap();
    fs::write(root.path().join("01_core/empty.rs"), "").unwrap();
    let out = run(
        root.path(),
        &["--checks", "v0", "--format", "sarif", "--analysis-scope"],
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let scope = &doc["runs"][0]["properties"]["analysisScope"];
    for (field, value) in [
        ("sourceFilesRead", 3),
        ("filesParsed", 1),
        ("syntaxErrors", 1),
        ("emptySourceFiles", 1),
        ("unsupportedLanguageFiles", 0),
        ("sourceReadErrors", 0),
    ] {
        assert_eq!(scope[field], value, "{field}: {scope}");
    }
}
#[test]
fn typed_counts_cover_all_variants_and_merge_associatively() {
    let syntax = ParseCounts::from_error(&ParseError::SyntaxError {
        path: "x.rs".into(),
        line: 1,
        column: 1,
        message: "arbitrary text".into(),
    });
    let empty = ParseCounts::from_error(&ParseError::EmptySource {
        path: "x.rs".into(),
    });
    let unsupported = ParseCounts::from_error(&ParseError::UnsupportedLanguage {
        path: "x".into(),
        language: Language::Unknown,
    });
    assert_eq!(
        syntax.merge(empty).merge(unsupported),
        ParseCounts {
            syntax_errors: 1,
            empty_sources: 1,
            unsupported_languages: 1
        }
    );
    assert_eq!(
        syntax.merge(empty.merge(unsupported)),
        syntax.merge(empty).merge(unsupported)
    );
    assert!(!repair_is_blocked(&[]));
}
