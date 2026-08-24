use crystalline_lint::contracts::file_provider::{FileProvider, SourceFile};
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::walker::FileWalker;
use std::fs;
use std::path::{Path, PathBuf};

fn load_config(root: &Path, text: &str) -> Result<CrystallineConfig, String> {
    let path = root.join("crystalline.toml");
    fs::write(&path, text).unwrap();
    CrystallineConfig::load(&path)
}

fn walk(
    root: &Path,
    config: CrystallineConfig,
) -> Vec<Result<SourceFile, crystalline_lint::contracts::file_provider::SourceError>> {
    FileWalker::new(root.to_owned(), config).files().collect()
}

fn relative_paths(root: &Path, files: &[SourceFile]) -> Vec<PathBuf> {
    files
        .iter()
        .map(|file| file.path.strip_prefix(root).unwrap().to_owned())
        .collect()
}

#[test]
fn ambiguous_or_unknown_layer_mappings_are_rejected_independent_of_toml_order() {
    let cases = [
        ("duplicate-forward", "[layers]\nL1='shared'\nL2='shared'\n"),
        ("duplicate-reverse", "[layers]\nL2='shared'\nL1='shared'\n"),
        (
            "unknown-forward",
            "[layers]\nmystery='shared'\nL1='shared'\n",
        ),
        (
            "unknown-reverse",
            "[layers]\nL1='shared'\nmystery='shared'\n",
        ),
    ];
    let mut accepted = Vec::new();
    for (name, text) in cases {
        let root = tempfile::tempdir().unwrap();
        if load_config(root.path(), text).is_ok() {
            accepted.push(name);
        }
    }

    let root = tempfile::tempdir().unwrap();
    assert!(load_config(root.path(), "[layers]\nL1='one'\nL1='two'\n").is_err());
    assert!(
        accepted.is_empty(),
        "accepted ambiguous layer cases: {accepted:?}"
    );

    for invalid in ["", "/absolute", ".", "..", "one/two", "one\\two"] {
        let root = tempfile::tempdir().unwrap();
        let text = format!("[layers]\nL1={invalid:?}\n");
        assert!(
            load_config(root.path(), &text).is_err(),
            "accepted invalid layer path {invalid:?}"
        );
    }

    for alias in ["lab", "Lab"] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("research")).unwrap();
        fs::write(root.path().join("research/probe.rs"), "fn probe() {}\n").unwrap();
        let config = load_config(root.path(), &format!("[layers]\n{alias}='research'\n")).unwrap();
        let files: Vec<_> = walk(root.path(), config)
            .into_iter()
            .map(Result::unwrap)
            .collect();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].layer,
            Layer::Lab,
            "alias {alias} did not map to Lab"
        );
    }

    let root = tempfile::tempdir().unwrap();
    assert!(
        load_config(root.path(), "[layers]\nlab='one'\nLab='two'\n").is_err(),
        "accepted simultaneous lab and Lab aliases"
    );
}

#[test]
fn unreadable_eligible_source_is_an_error_and_does_not_hide_readable_files() {
    // LIMITATION(P0073): FileProvider exposes no deterministic injection point for a
    // WalkDir traversal error; permissions are not reliable when the gate runs as root.
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    fs::write(root.path().join("01_core/good.rs"), b"fn good() {}\n").unwrap();
    fs::write(root.path().join("01_core/invalid.rs"), [0xff, 0xfe, 0xfd]).unwrap();
    let results = walk(root.path(), CrystallineConfig::default());
    let oks: Vec<_> = results
        .iter()
        .filter_map(|result| result.as_ref().ok())
        .collect();
    let errors: Vec<_> = results
        .iter()
        .filter_map(|result| result.as_ref().err())
        .collect();
    assert_eq!(oks.len(), 1);
    assert!(oks[0].path.ends_with("01_core/good.rs"));
    assert_eq!(errors.len(), 1);
    assert!(errors[0].path().ends_with("01_core/invalid.rs"));
}

fn build_order_fixture(root: &Path, reverse: bool) {
    let entries = [
        ("01_core/z.rs", "fn z() {}\n"),
        ("01_core/a.rs", "fn a() {}\n"),
        ("02_shell/m.ts", "export function m() {}\n"),
        ("nested/c.py", "def c(): pass\n"),
    ];
    let indexes: Vec<_> = if reverse {
        (0..entries.len()).rev().collect()
    } else {
        (0..entries.len()).collect()
    };
    for index in indexes {
        let (path, content) = entries[index];
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        fs::write(root.join(path), content).unwrap();
    }
}

#[test]
fn enumeration_order_is_canonical_across_opposite_creation_orders() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    build_order_fixture(first.path(), false);
    build_order_fixture(second.path(), true);
    let a: Vec<_> = walk(first.path(), CrystallineConfig::default())
        .into_iter()
        .map(Result::unwrap)
        .collect();
    let b: Vec<_> = walk(second.path(), CrystallineConfig::default())
        .into_iter()
        .map(Result::unwrap)
        .collect();
    let a_paths = relative_paths(first.path(), &a);
    let b_paths = relative_paths(second.path(), &b);
    assert_eq!(a_paths, b_paths);
    let mut sorted = a_paths.clone();
    sorted.sort();
    assert_eq!(a_paths, sorted);
}

#[cfg(unix)]
#[test]
fn symlinks_never_escape_root_or_count_as_adjacent_test_coverage() {
    use std::os::unix::fs::symlink;
    use std::process::Command;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.rs"), "fn secret() {}\n").unwrap();
    fs::create_dir(root.path().join("01_core")).unwrap();
    for source in ["external.rs", "internal.rs", "fifo.rs"] {
        fs::write(
            root.path().join("01_core").join(source),
            format!("fn {}() {{}}\n", source.trim_end_matches(".rs")),
        )
        .unwrap();
    }
    fs::write(root.path().join("internal-target.txt"), "not a source\n").unwrap();
    symlink(
        outside.path().join("secret.rs"),
        root.path().join("link.rs"),
    )
    .unwrap();
    symlink(outside.path(), root.path().join("linked_dir")).unwrap();
    symlink(
        outside.path().join("secret.rs"),
        root.path().join("01_core/external_test.rs"),
    )
    .unwrap();
    symlink(
        root.path().join("internal-target.txt"),
        root.path().join("01_core/internal_test.rs"),
    )
    .unwrap();
    symlink(
        outside.path().join("missing.rs"),
        root.path().join("01_core/broken_test.rs"),
    )
    .unwrap();
    symlink(root.path(), root.path().join("loop")).unwrap();
    let fifo = root.path().join("01_core/fifo_test.rs");
    let status = Command::new("mkfifo").arg(&fifo).status().unwrap();
    assert!(
        status.success(),
        "mkfifo failed for deterministic FIFO fixture"
    );

    let files: Vec<_> = walk(root.path(), CrystallineConfig::default())
        .into_iter()
        .map(Result::unwrap)
        .collect();
    assert_eq!(files.len(), 3, "symlink or FIFO was enumerated: {files:?}");
    let mut false_coverage = Vec::new();
    for source in ["external.rs", "internal.rs", "fifo.rs"] {
        let file = files
            .iter()
            .find(|file| file.path.ends_with(Path::new("01_core").join(source)))
            .unwrap();
        if file.has_adjacent_test {
            false_coverage.push(source);
        }
    }
    assert!(
        false_coverage.is_empty(),
        "non-regular adjacent candidates counted as coverage: {false_coverage:?}"
    );
}

#[test]
fn exclusions_are_component_and_exact_path_based_without_prefix_leakage() {
    let root = tempfile::tempdir().unwrap();
    for path in [
        "target/a.rs",
        "not-target/a.rs",
        "targeted/a.rs",
        "dir/lib.rs",
        "dir/lib.rs.bak.rs",
        "other/dir/lib.rs",
        "dir2/lib.rs",
    ] {
        fs::create_dir_all(root.path().join(path).parent().unwrap()).unwrap();
        fs::write(root.path().join(path), "fn value() {}\n").unwrap();
    }
    let config = load_config(
        root.path(),
        "[excluded]\nbuild='target'\n[excluded_files]\none='dir/lib.rs'\n",
    )
    .unwrap();
    let files: Vec<_> = walk(root.path(), config)
        .into_iter()
        .map(Result::unwrap)
        .collect();
    let paths = relative_paths(root.path(), &files);
    assert!(!paths.contains(&PathBuf::from("target/a.rs")));
    assert!(!paths.contains(&PathBuf::from("dir/lib.rs")));
    for expected in [
        "not-target/a.rs",
        "targeted/a.rs",
        "dir/lib.rs.bak.rs",
        "other/dir/lib.rs",
        "dir2/lib.rs",
    ] {
        assert!(
            paths.contains(&PathBuf::from(expected)),
            "missing non-excluded path {expected}: {paths:?}"
        );
    }
    assert!(files
        .iter()
        .any(|file| { file.path.ends_with("not-target/a.rs") && file.layer == Layer::Unknown }));

    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        let first = root
            .path()
            .join("dir")
            .join(OsString::from_vec(b"\xff.rs".to_vec()));
        let second = root
            .path()
            .join("dir")
            .join(OsString::from_vec(b"\xfe.rs".to_vec()));
        fs::write(&first, "fn first() {}\n").unwrap();
        fs::write(&second, "fn second() {}\n").unwrap();
        let config = load_config(root.path(), "[excluded_files]\none='dir/lib.rs'\n").unwrap();
        let files: Vec<_> = walk(root.path(), config)
            .into_iter()
            .map(Result::unwrap)
            .collect();
        assert!(files.iter().any(|file| file.path == first));
        assert!(files.iter().any(|file| file.path == second));
    }
}

#[test]
fn adjacent_tests_require_regular_files_and_self_tests_are_not_coverage() {
    let root = tempfile::tempdir().unwrap();
    let cases = [
        (
            "01_core/rust/foo.rs",
            "01_core/rust/foo_test.rs",
            Language::Rust,
        ),
        (
            "01_core/ts/foo.ts",
            "01_core/ts/foo.test.ts",
            Language::TypeScript,
        ),
        (
            "01_core/python/foo.py",
            "01_core/python/test_foo.py",
            Language::Python,
        ),
    ];
    for (source, test, _) in &cases {
        fs::create_dir_all(root.path().join(source).parent().unwrap()).unwrap();
        fs::write(root.path().join(source), "source\n").unwrap();
        fs::write(root.path().join(test), "test\n").unwrap();
    }
    fs::create_dir_all(root.path().join("01_core/dircase/bar_test.rs")).unwrap();
    fs::write(root.path().join("01_core/dircase/bar.rs"), "fn bar() {}\n").unwrap();

    let files: Vec<_> = walk(root.path(), CrystallineConfig::default())
        .into_iter()
        .map(Result::unwrap)
        .collect();
    for (source, test, language) in &cases {
        let source_file = files
            .iter()
            .find(|file| file.path.ends_with(source))
            .unwrap();
        let test_file = files.iter().find(|file| file.path.ends_with(test)).unwrap();
        assert_eq!(&source_file.language, language);
        assert!(source_file.has_adjacent_test);
        assert!(!test_file.has_adjacent_test);
    }
    let directory_case = files
        .iter()
        .find(|file| file.path.ends_with("01_core/dircase/bar.rs"))
        .unwrap();
    assert!(!directory_case.has_adjacent_test);
    assert_eq!(directory_case.layer, Layer::L1);
}
