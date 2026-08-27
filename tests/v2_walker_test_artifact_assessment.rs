//! Oráculo independente P0118 — somente evidência regular e aceita cobre produção.

use crystalline_lint::contracts::file_provider::{FileProvider, SourceError, SourceRole};
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::walker::FileWalker;
use std::fs;
use std::path::Path;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    fs::write(path, content).expect("write fixture");
}

#[test]
fn walker_marks_assert_only_python_test_by_path_and_covers_its_production() {
    let root = tempfile::tempdir().expect("tempdir");
    write(
        root.path(),
        "01_core/catalogo.py",
        "def catalogo(value):\n    return value\n",
    );
    write(
        root.path(),
        "01_core/test_catalogo.py",
        "def test_catalogo():\n    assert 2 + 2 == 4\n",
    );

    let files: Vec<_> = FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
        .files()
        .map(Result::unwrap)
        .collect();
    let production = files
        .iter()
        .find(|file| file.path.ends_with("01_core/catalogo.py"))
        .expect("production");
    let test = files
        .iter()
        .find(|file| file.path.ends_with("01_core/test_catalogo.py"))
        .expect("test");
    assert_eq!(production.source_role, SourceRole::Production);
    assert!(production.has_adjacent_test);
    assert_eq!(test.source_role, SourceRole::Test);
    assert!(!test.has_adjacent_test);
}

#[test]
fn every_supported_adjacent_convention_covers_and_removal_restores_uncovered() {
    let cases = [
        ("01_core/rust/catalogo.rs", "01_core/rust/catalogo_test.rs"),
        (
            "01_core/python/catalogo.py",
            "01_core/python/test_catalogo.py",
        ),
        (
            "01_core/python/catalogo.py",
            "01_core/python/catalogo_test.py",
        ),
        ("01_core/ts/catalogo.ts", "01_core/ts/catalogo.test.ts"),
        ("01_core/ts/catalogo.ts", "01_core/ts/catalogo.spec.ts"),
        ("01_core/ts/catalogo.ts", "01_core/ts/catalogo.test.tsx"),
        ("01_core/ts/catalogo.ts", "01_core/ts/catalogo.spec.tsx"),
        ("01_core/tsx/catalogo.tsx", "01_core/tsx/catalogo.test.ts"),
        ("01_core/tsx/catalogo.tsx", "01_core/tsx/catalogo.spec.ts"),
        ("01_core/tsx/catalogo.tsx", "01_core/tsx/catalogo.test.tsx"),
        ("01_core/tsx/catalogo.tsx", "01_core/tsx/catalogo.spec.tsx"),
        ("01_core/zig/catalogo.zig", "01_core/zig/catalogo_test.zig"),
        ("01_core/go/catalogo.go", "01_core/go/catalogo_test.go"),
        ("01_core/c/catalogo.c", "01_core/c/test_catalogo.c"),
        ("01_core/c/catalogo.c", "01_core/c/catalogo_test.c"),
        ("01_core/cpp/catalogo.cxx", "01_core/cpp/test_catalogo.cpp"),
        ("01_core/cpp/catalogo.cxx", "01_core/cpp/catalogo_test.cpp"),
        ("01_core/cpp/catalogo.cxx", "01_core/cpp/test_catalogo.cc"),
        ("01_core/cpp/catalogo.cxx", "01_core/cpp/catalogo_test.cc"),
        (
            "01_core/java/Catalogo.java",
            "01_core/java/CatalogoTest.java",
        ),
        (
            "01_core/java/Catalogo.java",
            "01_core/java/Catalogo_test.java",
        ),
        (
            "01_core/java/Catalogo.java",
            "01_core/java/TestCatalogo.java",
        ),
        (
            "01_core/elixir/catalogo.ex",
            "01_core/elixir/catalogo_test.exs",
        ),
    ];

    let mut mismatches = Vec::new();
    for (production_path, test_path) in cases {
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), production_path, "synthetic production\n");
        write(root.path(), test_path, "synthetic test\n");

        let with_test: Vec<_> =
            FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
                .files()
                .map(Result::unwrap)
                .collect();
        let production = with_test
            .iter()
            .find(|file| file.path.ends_with(production_path))
            .unwrap_or_else(|| panic!("production not walked: {production_path}"));
        let test = with_test
            .iter()
            .find(|file| file.path.ends_with(test_path))
            .unwrap_or_else(|| panic!("test not walked: {test_path}"));
        if production.source_role != SourceRole::Production {
            mismatches.push(format!("production role: {production_path}"));
        }
        if !production.has_adjacent_test {
            mismatches.push(format!(
                "candidate did not cover: {production_path} <- {test_path}"
            ));
        }
        if test.source_role != SourceRole::Test {
            mismatches.push(format!("test role: {test_path}"));
        }

        fs::remove_file(root.path().join(test_path)).expect("remove candidate");
        let without_test: Vec<_> =
            FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
                .files()
                .map(Result::unwrap)
                .collect();
        let production = without_test
            .iter()
            .find(|file| file.path.ends_with(production_path))
            .unwrap_or_else(|| panic!("production not walked after removal: {production_path}"));
        if production.has_adjacent_test {
            mismatches.push(format!(
                "removed candidate still covers: {production_path} <- {test_path}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "adjacency mismatches: {mismatches:#?}"
    );
}

#[test]
fn typescript_test_artifacts_ignore_nested_test_candidates() {
    let cases = [
        (
            "01_core/ts/catalogo.test.ts",
            "01_core/ts/catalogo.test.test.ts",
        ),
        (
            "01_core/ts/catalogo.spec.tsx",
            "01_core/ts/catalogo.spec.spec.tsx",
        ),
    ];

    for (test_path, nested_candidate) in cases {
        let root = tempfile::tempdir().expect("tempdir");
        write(root.path(), test_path, "synthetic test\n");
        write(root.path(), nested_candidate, "nested synthetic test\n");
        let files: Vec<_> =
            FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
                .files()
                .map(Result::unwrap)
                .collect();
        let test = files
            .iter()
            .find(|file| file.path.ends_with(test_path))
            .unwrap_or_else(|| panic!("test not walked: {test_path}"));
        assert_eq!(test.source_role, SourceRole::Test, "path={test_path}");
        assert!(
            !test.has_adjacent_test,
            "a test artifact cannot acquire coverage from {nested_candidate}"
        );
    }
}

#[test]
fn excluded_test_file_is_not_coverage_evidence() {
    let root = tempfile::tempdir().expect("tempdir");
    write(
        root.path(),
        "01_core/catalogo.py",
        "def catalogo():\n    return 1\n",
    );
    write(
        root.path(),
        "01_core/test_catalogo.py",
        "def test_x():\n    assert True\n",
    );
    let mut config = CrystallineConfig::default();
    config.excluded_files.insert(
        "synthetic_test".to_string(),
        "01_core/test_catalogo.py".to_string(),
    );

    let files: Vec<_> = FileWalker::new(root.path().to_path_buf(), config)
        .files()
        .map(Result::unwrap)
        .collect();
    assert_eq!(files.len(), 1, "excluded test must not be discovered");
    assert_eq!(files[0].source_role, SourceRole::Production);
    assert!(
        !files[0].has_adjacent_test,
        "excluded test cannot cover production"
    );
}

#[test]
fn directory_named_like_test_is_not_coverage_evidence() {
    let root = tempfile::tempdir().expect("tempdir");
    write(
        root.path(),
        "01_core/catalogo.py",
        "def catalogo():\n    return 1\n",
    );
    fs::create_dir_all(root.path().join("01_core/test_catalogo.py")).expect("test-like dir");
    let files: Vec<_> = FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
        .files()
        .map(Result::unwrap)
        .collect();
    assert_eq!(files.len(), 1);
    assert!(!files[0].has_adjacent_test);
}

#[cfg(unix)]
#[test]
fn symlink_named_like_test_is_not_coverage_evidence() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("tempdir");
    write(
        root.path(),
        "01_core/catalogo.py",
        "def catalogo():\n    return 1\n",
    );
    write(
        root.path(),
        "outside.py",
        "def test_x():\n    assert True\n",
    );
    symlink(
        root.path().join("outside.py"),
        root.path().join("01_core/test_catalogo.py"),
    )
    .expect("symlink");
    let files: Vec<_> = FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
        .files()
        .map(Result::unwrap)
        .collect();
    let production = files
        .iter()
        .find(|file| file.path.ends_with("01_core/catalogo.py"))
        .expect("production");
    assert!(!production.has_adjacent_test);
    assert!(files
        .iter()
        .all(|file| !file.path.ends_with("01_core/test_catalogo.py")));
}

#[cfg(unix)]
#[test]
fn accepted_non_utf8_source_name_becomes_explicit_source_error() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = tempfile::tempdir().expect("tempdir");
    let name = OsString::from_vec(b"catalogo_\xff.py".to_vec());
    fs::write(root.path().join(name), "value = 1\n").expect("write opaque source");

    let results: Vec<_> = FileWalker::new(root.path().to_path_buf(), CrystallineConfig::default())
        .files()
        .collect();
    assert_eq!(results.len(), 1);
    assert!(matches!(
        &results[0],
        Err(SourceError::Unreadable { reason, .. })
            if reason.contains("papel estrutural não classificável")
    ));
}
