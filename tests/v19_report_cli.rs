use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn project() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::write(
        root.path().join("01_core/formats.rs"),
        "fn f(s: &str) { match s {\n    \"yaml\" | \"yml\" => 1,\n    _ => 0\n} }\n",
    )
    .unwrap();
    fs::write(
        root.path().join("01_core/modes.rs"),
        "fn f(x: Mode) { match x {\n    Mode::A | Mode::B if ready() => 1,\n    Mode::C | Mode::D => 0\n} }\n",
    ).unwrap();
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(root)
        .args([".", "--checks", "v19", "--fail-on", "warning"])
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn save_reference(root: &Path) -> Vec<u8> {
    let output = run(root, &["--format", "sarif"]);
    assert!(output.status.success(), "{output:?}");
    fs::write(root.join("reference.sarif"), &output.stdout).unwrap();
    output.stdout
}

#[test]
fn reference_highlights_new_changed_and_moved_notes_without_failing() {
    let root = project();
    let original = save_reference(root.path());
    let formats = root.path().join("01_core/formats.rs");
    fs::write(
        &formats,
        format!("\n\n{}", fs::read_to_string(&formats).unwrap()),
    )
    .unwrap();
    fs::write(root.path().join("01_core/modes.rs"),
        "fn f(x: Mode) { match x {\n    Mode::A | Mode::E if ready() => 1,\n    Mode::C | Mode::D => 0,\n    Mode::F | Mode::G => 2\n} }\n").unwrap();
    let output = run(
        root.path(),
        &["--v19-reference", "reference.sarif", "--analysis-scope"],
    );
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains("novas: 1; alteradas: 1; novas ou alteradas: 0; sem alteração: 2"),
        "{text}"
    );
    assert!(text.contains("[sem alteração] linha 4"));
    assert!(text.contains("[alterada] linha 2"));
    assert!(text.contains("[nova] linha 4"));
    assert!(text.contains("Analysis scope:"));
    assert_eq!(
        fs::read(root.path().join("reference.sarif")).unwrap(),
        original
    );
    let output = run(
        root.path(),
        &["--v19-reference", "reference.sarif", "--format", "sarif"],
    );
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let run_data = &document["runs"][0];
    assert_eq!(
        run_data["properties"]["v19Comparison"],
        serde_json::json!({"new":1,"updated":1,"newOrUpdated":0,"unchanged":2})
    );
    let results = run_data["results"].as_array().unwrap();
    let states: Vec<_> = results
        .iter()
        .map(|result| result["properties"]["v19Change"].as_str().unwrap())
        .collect();
    assert_eq!(states, ["unchanged", "updated", "unchanged", "new"]);
    assert!(results.iter().all(|result| result["level"] == "note"));
    // A comparison report can itself be the next reference: current evidence only.
    fs::write(root.path().join("next.sarif"), output.stdout).unwrap();
    let next = run(root.path(), &["--v19-reference", "next.sarif"]);
    assert!(String::from_utf8(next.stdout)
        .unwrap()
        .contains("sem alteração: 4"));
    let quiet = run(
        root.path(),
        &["--v19-reference", "reference.sarif", "--quiet"],
    );
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());
}

#[test]
fn reference_is_portable_across_roots_and_ambiguous_changes_are_explicit() {
    let original = project();
    let bytes = save_reference(original.path());
    let copy = project();
    fs::write(copy.path().join("reference.sarif"), bytes).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(copy.path())
        .arg(copy.path()) // Absolute analysis root, unlike the saved run's `.`.
        .args(["--checks", "v19", "--v19-reference", "reference.sarif"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("sem alteração: 3"));
    let modes = copy.path().join("01_core/modes.rs");
    let source = fs::read_to_string(&modes)
        .unwrap()
        .replace("Mode::A", "Mode::Z");
    fs::write(&modes, format!("\n{source}")).unwrap();
    let output = run(copy.path(), &["--v19-reference", "reference.sarif"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(
        text.contains("novas ou alteradas: 1; sem alteração: 2"),
        "{text}"
    );
    assert!(text.contains("[nova ou alterada] linha 3"));
}

#[test]
fn invalid_or_incomplete_reference_never_becomes_an_empty_reference() {
    let root = project();
    let valid: serde_json::Value = serde_json::from_slice(&save_reference(root.path())).unwrap();
    let mut incomplete = valid.clone();
    incomplete["runs"][0]["properties"]["v19Reference"]["complete"] = false.into();
    let mut unselected = valid.clone();
    unselected["runs"][0]["properties"]["v19Reference"]["selected"] = false.into();
    let mut version = valid.clone();
    version["runs"][0]["properties"]["v19Reference"]["schema"] = "future-version".into();
    let mut invalid_path = valid.clone();
    invalid_path["runs"][0]["properties"]["v19Reference"]["notes"][0]["path"] =
        "../outside.rs".into();
    let mut missing_field = valid.clone();
    missing_field["runs"][0]["properties"]["v19Reference"]
        .as_object_mut()
        .unwrap()
        .remove("notes");
    let mut multiple = valid.clone();
    multiple["runs"]
        .as_array_mut()
        .unwrap()
        .push(valid["runs"][0].clone());
    for bytes in [
        b"not JSON".to_vec(),
        b"{}".to_vec(),
        serde_json::to_vec(&incomplete).unwrap(),
        serde_json::to_vec(&unselected).unwrap(),
        serde_json::to_vec(&version).unwrap(),
        serde_json::to_vec(&invalid_path).unwrap(),
        serde_json::to_vec(&missing_field).unwrap(),
        serde_json::to_vec(&multiple).unwrap(),
    ] {
        fs::write(root.path().join("invalid.sarif"), bytes).unwrap();
        for path in ["invalid.sarif", "missing.sarif"] {
            let output = run(root.path(), &["--v19-reference", path]);
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
            let quiet = run(root.path(), &["--v19-reference", path, "--quiet"]);
            assert_eq!(quiet.status.code(), Some(2));
            assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());
        }
    }
    fs::write(root.path().join("01_core/broken.rs"), "fn broken(").unwrap();
    let output = run(root.path(), &["--v19-reference", "reference.sarif"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("análise incompleta"));
    let incomplete_output = run(root.path(), &["--format", "sarif"]);
    let document: serde_json::Value = serde_json::from_slice(&incomplete_output.stdout).unwrap();
    assert_eq!(
        document["runs"][0]["properties"]["v19Reference"]["complete"],
        false
    );
}

#[test]
fn empty_reference_and_severity_overrides_keep_information_separate_from_failure() {
    let empty = tempfile::tempdir().unwrap();
    let baseline = save_reference(empty.path());
    let root = project();
    fs::write(root.path().join("reference.sarif"), baseline).unwrap();
    let output = run(root.path(), &["--v19-reference", "reference.sarif"]);
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("novas: 3; alteradas: 0"));
    let zero = run(empty.path(), &["--v19-reference", "reference.sarif"]);
    assert!(zero.status.success());
    assert!(String::from_utf8(zero.stdout)
        .unwrap()
        .contains("novas: 0; alteradas: 0"));
    fs::write(
        root.path().join("crystalline.toml"),
        "[rules]\nV19 = { level = \"warning\" }\n",
    )
    .unwrap();
    let output = run(root.path(), &["--v19-reference", "reference.sarif"]);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.matches("warning:").count(), 3);
    assert!(text.contains("novas: 0; alteradas: 0"));
    assert!(!text.contains("[nova]"));
}

#[test]
fn consolidated_notes_preserve_details_scope_and_success_exit() {
    let root = project();
    let output = run(root.path(), &["--analysis-scope"]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.matches("Notas informativas V19").count(), 1);
    assert!(text.contains("braços: 3; arquivos: 2"));
    assert_eq!(text.matches("Executar o corpo não comprova").count(), 1);
    assert_eq!(
        text.matches("Alternativas escritas nos grupos: 2").count(),
        3
    );
    assert!(text.contains("formats.rs — notas: 1"));
    assert!(text.contains("modes.rs — notas: 2"));
    assert!(text.contains("linha 2, coluna 5"));
    assert!(text.contains("linha 3, coluna 5"));
    assert!(text.contains("guard: presente"));
    assert!(text.contains("guard: ausente"));
    assert!(text.contains("`\"yaml\" | \"yml\"`"));
    assert!(text.contains("Analysis scope:"));
    assert!(!text.contains("No violations found"));
    let quiet = run(root.path(), &["--quiet", "--analysis-scope"]);
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());
}

#[test]
fn sarif_keeps_each_note_and_explicit_severity_override_still_fails() {
    let root = project();
    let output = run(root.path(), &["--format", "sarif"]);
    assert!(output.status.success());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let results = document["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for result in results {
        assert_eq!(result["ruleId"], "V19");
        assert_eq!(result["level"], "note");
        assert!(result["message"]["text"]
            .as_str()
            .unwrap()
            .starts_with("Este braço agrupa alternativas."));
        assert!(
            result["locations"][0]["physicalLocation"]["region"]["startLine"]
                .as_u64()
                .unwrap()
                >= 2
        );
        assert_eq!(
            result["locations"][0]["physicalLocation"]["region"]["startColumn"],
            5
        );
    }
    fs::write(
        root.path().join("crystalline.toml"),
        "[rules]\nV19 = { level = \"warning\" }\n",
    )
    .unwrap();
    let output = run(root.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("Notas informativas V19"));
    assert_eq!(text.matches("warning:").count(), 3);
    let quiet = run(root.path(), &["--quiet"]);
    assert_eq!(quiet.status.code(), Some(1));
    assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());
}
