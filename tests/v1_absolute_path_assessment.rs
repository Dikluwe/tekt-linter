//! Black-box regression for P0117: a symbolic absolute root is infrastructure,
//! never a collection of missing prompts.

use std::fs;
use std::process::Command;

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("00_nucleo/prompts")).unwrap();
    fs::create_dir_all(dir.path().join("01_core")).unwrap();
    fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/v01_pass/crystalline.toml"
        ),
        dir.path().join("crystalline.toml"),
    )
    .unwrap();
    fs::write(dir.path().join("00_nucleo/prompts/core.md"), "# Core\n").unwrap();
    fs::write(
        dir.path().join("01_core/core.rs"),
        "//! @prompt 00_nucleo/prompts/core.md\n//! @layer L1\nfn core() {}\n",
    )
    .unwrap();
    dir
}

fn run(path: &std::path::Path, format: &str) -> std::process::Output {
    run_from(
        path,
        format,
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
}

fn run_from(path: &std::path::Path, format: &str, cwd: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(cwd)
        .arg(path)
        .args(["--checks", "v1", "--format", format])
        .output()
        .unwrap()
}

#[test]
fn canonical_absolute_root_is_clean() {
    let project = project();
    let output = run(project.path(), "text");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("[V1]"));
}

#[test]
fn relative_and_canonical_roots_have_the_same_clean_semantics() {
    let project = project();
    let parent = project.path().parent().unwrap();
    let relative = std::path::Path::new(project.path().file_name().unwrap());
    let relative_output = run_from(relative, "sarif", parent);
    let absolute_output = run(project.path(), "sarif");
    assert!(relative_output.status.success());
    assert!(absolute_output.status.success());
    let relative_json: serde_json::Value = serde_json::from_slice(&relative_output.stdout).unwrap();
    let absolute_json: serde_json::Value = serde_json::from_slice(&absolute_output.stdout).unwrap();
    assert_eq!(
        relative_json["runs"][0]["results"],
        absolute_json["runs"][0]["results"]
    );
}

#[test]
fn truly_missing_prompt_remains_exactly_one_v1() {
    let project = project();
    fs::remove_file(project.path().join("00_nucleo/prompts/core.md")).unwrap();
    let output = run(project.path(), "text");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!output.status.success());
    assert_eq!(stdout.matches("[V1]").count(), 1, "{stdout}");
    assert_eq!(stdout.matches("[V0]").count(), 0, "{stdout}");
}

#[cfg(unix)]
#[test]
fn symbolic_absolute_root_is_one_fatal_v0_and_zero_v1_in_text_and_sarif() {
    use std::os::unix::fs::symlink;

    let project = project();
    let alias_parent = tempfile::tempdir().unwrap();
    let alias = alias_parent.path().join("project-alias");
    symlink(project.path(), &alias).unwrap();

    for format in ["text", "sarif"] {
        let output = run(&alias, format);
        assert!(!output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        if format == "sarif" {
            let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
            let results = value["runs"][0]["results"].as_array().unwrap();
            assert_eq!(results.iter().filter(|r| r["ruleId"] == "V0").count(), 1);
            assert_eq!(results.iter().filter(|r| r["ruleId"] == "V1").count(), 0);
            let finding = results.iter().find(|r| r["ruleId"] == "V0").unwrap();
            assert_eq!(finding["level"], "error");
            assert!(finding["message"]["text"]
                .as_str()
                .unwrap()
                .contains("raiz de prompts"));
        } else {
            assert_eq!(stdout.matches("[V0]").count(), 1, "{stdout}");
            assert_eq!(stdout.matches("[V1]").count(), 0, "{stdout}");
        }
        assert!(!stdout.contains("prompt inexistente"), "{stdout}");
    }
}

#[cfg(unix)]
#[test]
fn invalid_root_preempts_missing_header_v1() {
    use std::os::unix::fs::symlink;
    let project = project();
    fs::write(
        project.path().join("01_core/core.rs"),
        "//! @layer L1\nfn core() {}\n",
    )
    .unwrap();
    let holder = tempfile::tempdir().unwrap();
    let alias = holder.path().join("alias");
    symlink(project.path(), &alias).unwrap();
    let output = run(&alias, "text");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("[V0]").count(), 1, "{stdout}");
    assert_eq!(stdout.matches("[V1]").count(), 0, "{stdout}");
}

#[cfg(unix)]
#[test]
fn identical_prompt_failures_are_deduplicated_across_consumers() {
    use std::os::unix::fs::symlink;
    let project = project();
    fs::write(
        project.path().join("01_core/second.rs"),
        "//! @prompt 00_nucleo/prompts/core.md\n//! @layer L1\nfn second() {}\n",
    )
    .unwrap();
    fs::remove_file(project.path().join("00_nucleo/prompts/core.md")).unwrap();
    symlink(
        "absent.md",
        project.path().join("00_nucleo/prompts/core.md"),
    )
    .unwrap();
    let output = run(project.path(), "text");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("[V0]").count(), 1, "{stdout}");
    assert_eq!(stdout.matches("[V1]").count(), 0, "{stdout}");
}
