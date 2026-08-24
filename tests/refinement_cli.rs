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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "PRESERVED\n");
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

#[test]
fn refine_revisions_reads_commits_without_changing_worktree() {
    let repository = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repository.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(git(&["init", "-q"]).status.success());
    assert!(git(&["config", "user.email", "fixture@example.invalid"])
        .status
        .success());
    assert!(git(&["config", "user.name", "Fixture"]).status.success());
    std::fs::write(
        repository.path().join("sample.rs"),
        "enum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "before"]).status.success());
    let before = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    std::fs::write(
        repository.path().join("sample.rs"),
        "// formatting only\nenum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "after"]).status.success());
    let after = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    std::fs::write(repository.path().join("dirty.txt"), "must survive\n").unwrap();

    let contract = repository.path().join("refinement.toml");
    std::fs::write(
        &contract,
        "id='git-fixture'\n[[observable]]\nkey='stable.variants'\nlanguage='rust'\nfile='sample.rs'\nquery='(enum_item name: (type_identifier) @_name body: (enum_variant_list) @value (#eq? @_name \"Stable\"))'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n[[relation]]\nkind='preserve'\nsource='stable.variants'\ntarget='stable.variants'\n",
    )
    .unwrap();
    let status_before = git(&["status", "--porcelain=v1", "-z"]).stdout;
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine-revisions",
            repository.path().to_str().unwrap(),
            "--before-ref",
            &before,
            "--after-ref",
            &after,
            "--contract",
            contract.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "PRESERVED\n");
    assert_eq!(
        git(&["status", "--porcelain=v1", "-z"]).stdout,
        status_before
    );
    assert_eq!(
        std::fs::read_to_string(repository.path().join("dirty.txt")).unwrap(),
        "must survive\n"
    );

    let exported = tempfile::tempdir().unwrap();
    for (name, oid) in [("before", before.as_str()), ("after", after.as_str())] {
        let root = exported.path().join(name);
        std::fs::create_dir(&root).unwrap();
        let content = git(&["show", &format!("{oid}:sample.rs")]).stdout;
        std::fs::write(root.join("sample.rs"), content).unwrap();
        snapshot(
            &root,
            &contract,
            &exported.path().join(format!("{name}.json")),
        );
    }
    let manual = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine",
            "--before",
            exported.path().join("before.json").to_str().unwrap(),
            "--after",
            exported.path().join("after.json").to_str().unwrap(),
            "--contract",
            contract.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(manual.status.code(), output.status.code());
    assert_eq!(manual.stdout, output.stdout);
}

#[test]
fn refine_revisions_reports_violated_unknown_and_input_errors() {
    let repository = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repository.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(git(&["init", "-q"]).status.success());
    assert!(git(&["config", "user.email", "fixture@example.invalid"])
        .status
        .success());
    assert!(git(&["config", "user.name", "Fixture"]).status.success());
    let contract = repository.path().join("refinement.toml");
    std::fs::write(&contract, "id='git-negative'\n[[observable]]\nkey='stable.variants'\nlanguage='rust'\nfile='sample.rs'\nquery='(enum_item name: (type_identifier) @_name body: (enum_variant_list) @value (#eq? @_name \"Stable\"))'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n[[relation]]\nkind='preserve'\nsource='stable.variants'\ntarget='stable.variants'\n").unwrap();
    std::fs::write(
        repository.path().join("sample.rs"),
        "enum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "before"]).status.success());
    let before = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    std::fs::write(
        repository.path().join("sample.rs"),
        "enum Stable { Alpha }\n",
    )
    .unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "violated"]).status.success());
    let violated = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    assert!(git(&["rm", "-q", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "unknown"]).status.success());
    let unknown = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    std::fs::write(repository.path().join("dirty.txt"), "unchanged\n").unwrap();
    let status_before = git(&["status", "--porcelain=v1", "-z"]).stdout;

    let run_refs = |source: &str, target: &str| {
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "refine-revisions",
                repository.path().to_str().unwrap(),
                "--before-ref",
                source,
                "--after-ref",
                target,
                "--contract",
                contract.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let violation = run_refs(&before, &violated);
    assert_eq!(violation.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&violation.stdout).contains("VIOLATED"));
    let inconclusive = run_refs(&before, &unknown);
    assert_eq!(inconclusive.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&inconclusive.stdout).contains("missing-observable"));
    assert_eq!(run_refs("--help", &unknown).status.code(), Some(2));
    assert_eq!(run_refs("does-not-exist", &unknown).status.code(), Some(2));
    assert_eq!(
        git(&["status", "--porcelain=v1", "-z"]).stdout,
        status_before
    );

    let not_git = tempfile::tempdir().unwrap();
    let error = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine-revisions",
            not_git.path().to_str().unwrap(),
            "--before-ref",
            "HEAD",
            "--after-ref",
            "HEAD",
            "--contract",
            contract.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(error.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn refine_revisions_does_not_follow_symlinks_or_run_git_extensions() {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let repository = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let marker = outside.path().join("executed");
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repository.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(git(&["init", "-q"]).status.success());
    assert!(git(&["config", "user.email", "fixture@example.invalid"])
        .status
        .success());
    assert!(git(&["config", "user.name", "Fixture"]).status.success());
    std::fs::write(
        outside.path().join("outside.rs"),
        "enum Stable { Escaped }\n",
    )
    .unwrap();
    symlink(
        outside.path().join("outside.rs"),
        repository.path().join("sample.rs"),
    )
    .unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "symlink"]).status.success());
    let symlink_oid = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();

    let hook = repository.path().join(".git/hooks/post-checkout");
    std::fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).unwrap();
    let mut permissions = std::fs::metadata(&hook).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&hook, permissions).unwrap();
    assert!(git(&[
        "config",
        "filter.hostile.smudge",
        &format!("touch {}", marker.display())
    ])
    .status
    .success());
    assert!(git(&[
        "config",
        "filter.hostile.clean",
        &format!("touch {}", marker.display())
    ])
    .status
    .success());

    let contract = repository.path().join("refinement.toml");
    std::fs::write(&contract, "id='git-hostile'\n[[observable]]\nkey='stable.variants'\nlanguage='rust'\nfile='sample.rs'\nquery='(enum_item name: (type_identifier) @_name body: (enum_variant_list) @value (#eq? @_name \"Stable\"))'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n[[relation]]\nkind='preserve'\nsource='stable.variants'\ntarget='stable.variants'\n").unwrap();
    let run = |oid: &str| {
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "refine-revisions",
                repository.path().to_str().unwrap(),
                "--before-ref",
                oid,
                "--after-ref",
                oid,
                "--contract",
                contract.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let symlink_result = run(&symlink_oid);
    assert_eq!(symlink_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&symlink_result.stdout).contains("unsupported-parser"));
    assert!(!marker.exists());

    assert!(git(&["rm", "-q", "sample.rs"]).status.success());
    assert!(git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{symlink_oid},sample.rs")
    ])
    .status
    .success());
    assert!(git(&["commit", "-qm", "gitlink"]).status.success());
    let gitlink_oid = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    let gitlink_result = run(&gitlink_oid);
    assert_eq!(gitlink_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&gitlink_result.stdout).contains("unsupported-parser"));
    assert!(!marker.exists());
}

#[test]
fn refine_revisions_rejects_path_and_blob_budget_exhaustion() {
    let repository = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(repository.path())
            .args(args)
            .output()
            .unwrap()
    };
    assert!(git(&["init", "-q"]).status.success());
    assert!(git(&["config", "user.email", "fixture@example.invalid"])
        .status
        .success());
    assert!(git(&["config", "user.name", "Fixture"]).status.success());
    let mut oversized = b"enum Stable { Alpha }\n".to_vec();
    oversized.resize(4 * 1024 * 1024 + 1, b' ');
    std::fs::write(repository.path().join("sample.rs"), oversized).unwrap();
    assert!(git(&["add", "sample.rs"]).status.success());
    assert!(git(&["commit", "-qm", "oversized"]).status.success());
    let oid = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    let contract = repository.path().join("blob.toml");
    std::fs::write(&contract, "id='budget'\n[[observable]]\nkey='x'\nlanguage='rust'\nfile='sample.rs'\nquery='(enum_item body: (enum_variant_list) @value)'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n[[relation]]\nkind='preserve'\nsource='x'\ntarget='x'\n").unwrap();
    let run = |contract: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "refine-revisions",
                repository.path().to_str().unwrap(),
                "--before-ref",
                &oid,
                "--after-ref",
                &oid,
                "--contract",
                contract.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let blob = run(&contract);
    assert_eq!(blob.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&blob.stdout).contains("budget-exhausted"));

    let mut many = String::from("id='paths'\n");
    for index in 0..513 {
        many.push_str(&format!("[[observable]]\nkey='k{index}'\nlanguage='rust'\nfile='p{index}.rs'\nquery='(identifier) @value'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n"));
    }
    many.push_str("[[relation]]\nkind='preserve'\nsource='k0'\ntarget='k0'\n");
    let paths = repository.path().join("paths.toml");
    std::fs::write(&paths, many).unwrap();
    let path_result = run(&paths);
    assert_eq!(path_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&path_result.stderr).contains("path budget exceeded"));

    assert!(git(&["rm", "-q", "sample.rs"]).status.success());
    let mut total_contract = String::from("id='total-budget'\n");
    for index in 0..9 {
        let mut content = format!("// {index}\nenum Stable {{ Alpha }}\n").into_bytes();
        content.resize(4 * 1024 * 1024, b' ');
        let name = format!("large{index}.rs");
        std::fs::write(repository.path().join(&name), content).unwrap();
        assert!(git(&["add", &name]).status.success());
        total_contract.push_str(&format!("[[observable]]\nkey='large{index}'\nlanguage='rust'\nfile='{name}'\nquery='(enum_item body: (enum_variant_list) @value)'\ncapture='value'\ncardinality='one'\non_missing='unknown'\n"));
    }
    total_contract.push_str("[[relation]]\nkind='preserve'\nsource='large0'\ntarget='large0'\n");
    assert!(git(&["commit", "-qm", "total budget"]).status.success());
    let total_oid = String::from_utf8(git(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string();
    let total_path = repository.path().join("total.toml");
    std::fs::write(&total_path, total_contract).unwrap();
    let total = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine-revisions",
            repository.path().to_str().unwrap(),
            "--before-ref",
            &total_oid,
            "--after-ref",
            &total_oid,
            "--contract",
            total_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(total.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&total.stdout).contains("budget-exhausted"));
}
