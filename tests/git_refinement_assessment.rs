use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn init(root: &Path) {
    assert!(git(root, &["init", "-q"]).status.success());
    assert!(git(
        root,
        &["config", "user.email", "assessment@example.invalid"]
    )
    .status
    .success());
    assert!(git(root, &["config", "user.name", "Assessment"])
        .status
        .success());
}

fn commit(root: &Path, message: &str) -> String {
    assert!(git(root, &["add", "."]).status.success());
    assert!(git(root, &["commit", "-qm", message]).status.success());
    String::from_utf8(git(root, &["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_owned()
}

fn contract(root: &Path, file: &str, on_missing: &str) -> PathBuf {
    let path = root.join("assessment-contract.toml");
    fs::write(
        &path,
        format!(
            "id='git-assessment'\n[[observable]]\nkey='stable.variants'\nlanguage='rust'\nfile='{file}'\nquery='(enum_item name: (type_identifier) @_name body: (enum_variant_list) @value (#eq? @_name \"Stable\"))'\ncapture='value'\ncardinality='one'\non_missing='{on_missing}'\n[[relation]]\nkind='preserve'\nsource='stable.variants'\ntarget='stable.variants'\n"
        ),
    )
    .unwrap();
    path
}

fn refine(root: &Path, before: &str, after: &str, contract: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .args([
            "refine-revisions",
            root.to_str().unwrap(),
            "--before-ref",
            before,
            "--after-ref",
            after,
            "--contract",
            contract.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

fn assert_input_blocked(output: Output) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(String::from_utf8_lossy(&output.stdout), "PRESERVED\n");
}

fn byte_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries: Vec<_> = fs::read_dir(current)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_owned();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                snapshot.insert(relative.clone(), b"<directory>".to_vec());
                visit(root, &path, snapshot);
            } else if kind.is_symlink() {
                snapshot.insert(
                    relative,
                    fs::read_link(&path)
                        .unwrap()
                        .as_os_str()
                        .as_encoded_bytes()
                        .to_vec(),
                );
            } else {
                snapshot.insert(relative, fs::read(path).unwrap());
            }
        }
    }
    let mut snapshot = BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}

#[test]
fn pathspec_magic_in_contract_file_is_rejected_as_input() {
    let repository = tempfile::tempdir().unwrap();
    init(repository.path());
    fs::create_dir(repository.path().join("nested")).unwrap();
    fs::write(
        repository.path().join("safe.rs"),
        "enum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    fs::write(
        repository.path().join("nested/secret.rs"),
        "enum Stable { Secret }\n",
    )
    .unwrap();
    let before = commit(repository.path(), "before");
    fs::write(
        repository.path().join("nested/secret.rs"),
        "enum Stable { Changed }\n",
    )
    .unwrap();
    let after = commit(repository.path(), "after");
    let contract = contract(repository.path(), ":(glob)**/*.rs", "absent");

    assert_input_blocked(refine(repository.path(), &before, &after, &contract));
}

#[test]
#[ignore = "RED congelado: política de Git alternates aguarda decisão arquitetural"]
fn repository_cannot_read_commits_from_external_alternate() {
    let external = tempfile::tempdir().unwrap();
    init(external.path());
    fs::write(
        external.path().join("sample.rs"),
        "enum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    let before = commit(external.path(), "external before");
    fs::write(
        external.path().join("sample.rs"),
        "// same observation\nenum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    let after = commit(external.path(), "external after");

    let repository = tempfile::tempdir().unwrap();
    init(repository.path());
    fs::create_dir_all(repository.path().join(".git/objects/info")).unwrap();
    fs::write(
        repository.path().join(".git/objects/info/alternates"),
        format!("{}\n", external.path().join(".git/objects").display()),
    )
    .unwrap();
    assert!(git(
        repository.path(),
        &["cat-file", "-e", &format!("{before}^{{commit}}")]
    )
    .status
    .success());
    let contract = contract(repository.path(), "sample.rs", "unknown");

    assert_input_blocked(refine(repository.path(), &before, &after, &contract));
}

#[test]
fn success_and_error_preserve_every_preexisting_repository_byte() {
    let repository = tempfile::tempdir().unwrap();
    init(repository.path());
    fs::write(
        repository.path().join("sample.rs"),
        "enum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    let before = commit(repository.path(), "before");
    fs::write(
        repository.path().join("sample.rs"),
        "// rewrite\nenum Stable { Alpha, Beta }\n",
    )
    .unwrap();
    let after = commit(repository.path(), "after");
    let contract = contract(repository.path(), "sample.rs", "unknown");
    fs::write(
        repository.path().join("sample.rs"),
        b"dirty tracked bytes\n",
    )
    .unwrap();
    fs::write(
        repository.path().join("untracked.bin"),
        b"keep me\0exactly\n",
    )
    .unwrap();
    assert!(git(
        repository.path(),
        &["status", "--porcelain=v2", "--untracked-files=all"]
    )
    .status
    .success());
    let before_bytes = byte_snapshot(repository.path());

    let success = refine(repository.path(), &before, &after, &contract);
    assert_eq!(success.status.code(), Some(0));
    assert_eq!(byte_snapshot(repository.path()), before_bytes);

    let error = refine(
        repository.path(),
        "object-does-not-exist",
        &after,
        &contract,
    );
    assert_input_blocked(error);
    assert_eq!(byte_snapshot(repository.path()), before_bytes);
}

#[test]
fn oversized_blob_is_observably_budget_exhausted_not_preserved() {
    let repository = tempfile::tempdir().unwrap();
    init(repository.path());
    let mut oversized = b"enum Stable { Alpha, Beta }\n".to_vec();
    oversized.resize(5 * 1024 * 1024, b' ');
    fs::write(repository.path().join("sample.rs"), oversized).unwrap();
    let oid = commit(repository.path(), "oversized");
    let contract = contract(repository.path(), "sample.rs", "unknown");

    let output = refine(repository.path(), &oid, &oid, &contract);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("budget-exhausted"));
    assert_ne!(String::from_utf8_lossy(&output.stdout), "PRESERVED\n");
}
