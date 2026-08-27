use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MINIMAL: &[u8] = b"[project]\nroot = \".\"\n\n[layers]\nL0 = \"00_nucleo\"\nL1 = \"01_core\"\n\n[excluded]\nbuild = \"target\"\ndeps = \"node_modules\"\nvcs = \".git\"\nvenv = \".venv\"\npycache = \"__pycache__\"\n\n[analysis.lineage]\nstrict_directories = [\"00_nucleo\", \"01_core\"]\n";

const COMPLETE: &[u8] = b"[project]\nroot = \".\"\n\n[layers]\nL0 = \"00_nucleo\"\nL1 = \"01_core\"\nL2 = \"02_shell\"\nL3 = \"03_infra\"\nL4 = \"04_wiring\"\nlab = \"lab\"\n\n[excluded]\nbuild = \"target\"\ndeps = \"node_modules\"\nvcs = \".git\"\nvenv = \".venv\"\npycache = \"__pycache__\"\n\n[analysis.lineage]\nstrict_directories = [\"00_nucleo\", \"01_core\", \"02_shell\", \"03_infra\", \"04_wiring\", \"lab\"]\n";

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_crystalline-lint")
}

fn run(args: &[&std::ffi::OsStr], cwd: Option<&Path>) -> Output {
    let mut command = Command::new(bin());
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.output().unwrap()
}

fn init(root: &Path) -> Output {
    run(&["init".as_ref(), root.as_os_str()], None)
}

fn mkdirs(root: &Path, names: &[&str]) {
    for name in names {
        fs::create_dir(root.join(name)).unwrap();
    }
}

fn assert_domain(output: &Output, code: i32, stdout: &[u8], stderr: &[u8]) {
    assert_eq!(output.status.code(), Some(code), "output: {output:?}");
    assert_eq!(output.stdout, stdout);
    assert_eq!(output.stderr, stderr);
}

fn created_line(root: &Path) -> Vec<u8> {
    format!("CREATED {}/crystalline.toml\n", root.display()).into_bytes()
}

#[test]
fn minimum_and_complete_goldens_are_literal_and_loader_accepted() {
    for (names, golden) in [
        (&["00_nucleo", "01_core"][..], MINIMAL),
        (
            &[
                "00_nucleo",
                "01_core",
                "02_shell",
                "03_infra",
                "04_wiring",
                "lab",
            ][..],
            COMPLETE,
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        mkdirs(root.path(), names);
        let output = init(root.path());
        assert_domain(&output, 0, &created_line(root.path()), b"");
        let path = root.path().join("crystalline.toml");
        assert_eq!(fs::read(&path).unwrap(), golden);
        crystalline_lint::infra::config::CrystallineConfig::load(&path).unwrap();
    }
}

#[test]
fn every_optional_subset_emits_only_present_layers_in_fixed_order() {
    let optional = ["02_shell", "03_infra", "04_wiring", "lab"];
    for mask in 0u8..16 {
        let root = tempfile::tempdir().unwrap();
        mkdirs(root.path(), &["00_nucleo", "01_core"]);
        for (bit, name) in optional.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                fs::create_dir(root.path().join(name)).unwrap();
            }
        }
        assert_eq!(init(root.path()).status.code(), Some(0));
        let bytes = fs::read(root.path().join("crystalline.toml")).unwrap();
        for (bit, name) in optional.iter().enumerate() {
            let mapping = if *name == "lab" {
                "lab = \"lab\"".to_string()
            } else {
                format!("L{} = \"{}\"", bit + 2, name)
            };
            assert_eq!(
                String::from_utf8_lossy(&bytes).contains(&mapping),
                mask & (1 << bit) != 0,
                "mask={mask:#06b}, mapping={mapping}"
            );
        }
    }
}

#[test]
fn creation_order_and_absolute_or_relative_spelling_do_not_change_bytes() {
    let parent = tempfile::tempdir().unwrap();
    let a = parent.path().join("normal");
    let b = parent.path().join("reverse");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    let names = [
        "00_nucleo",
        "01_core",
        "02_shell",
        "03_infra",
        "04_wiring",
        "lab",
    ];
    mkdirs(&a, &names);
    for name in names.iter().rev() {
        fs::create_dir(b.join(name)).unwrap();
    }
    assert_eq!(init(&a).status.code(), Some(0));
    let relative = PathBuf::from("reverse");
    let output = run(
        &["init".as_ref(), relative.as_os_str()],
        Some(parent.path()),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::read(a.join("crystalline.toml")).unwrap(), COMPLETE);
    assert_eq!(fs::read(b.join("crystalline.toml")).unwrap(), COMPLETE);
}

#[test]
fn present_valid_or_invalid_is_immutable_and_precedes_missing_topology() {
    for sentinel in [
        b"[project]\nroot='.'\n".as_slice(),
        b"not toml\n".as_slice(),
    ] {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("crystalline.toml");
        fs::write(&destination, sentinel).unwrap();
        let output = init(root.path());
        let stderr = format!(
            "crystalline-lint: config already exists: {}/crystalline.toml\n",
            root.path().display()
        );
        assert_domain(&output, 1, b"", stderr.as_bytes());
        assert_eq!(fs::read(destination).unwrap(), sentinel);
    }
}

#[test]
fn unknown_missing_and_case_ambiguity_are_exact_and_never_write() {
    let cases: &[(&[&str], &str)] = &[
        (&[], "required directories missing: 00_nucleo, 01_core"),
        (&["00_nucleo"], "required directories missing: 01_core"),
        (&["01_core"], "required directories missing: 00_nucleo"),
        (
            &["01_CORE", "00_NUCLEO"],
            "case-ambiguous layer directories: 00_NUCLEO, 01_CORE",
        ),
    ];
    for (names, reason) in cases {
        let root = tempfile::tempdir().unwrap();
        mkdirs(root.path(), names);
        let output = init(root.path());
        let stderr = format!("crystalline-lint: cannot infer Tekt topology: {reason}\n");
        assert_domain(&output, 2, b"", stderr.as_bytes());
        assert!(!root.path().join("crystalline.toml").exists());
    }
}

#[test]
fn unrelated_unicode_name_is_not_topology_evidence_or_ambiguity() {
    let root = tempfile::tempdir().unwrap();
    mkdirs(root.path(), &["00_nucleo", "01_core", "núcleo", "Λ-layer"]);
    let output = init(root.path());
    assert_domain(&output, 0, &created_line(root.path()), b"");
    assert_eq!(
        fs::read(root.path().join("crystalline.toml")).unwrap(),
        MINIMAL
    );
}

#[cfg(unix)]
#[test]
fn destination_symlinks_and_special_types_are_unsafe_and_untouched() {
    use std::os::unix::fs::symlink;
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("sentinel");
    fs::write(&target, b"outside-winner").unwrap();
    for mode in [
        "internal",
        "external",
        "dangling",
        "loop",
        "directory",
        "fifo",
    ] {
        let root = tempfile::tempdir().unwrap();
        mkdirs(root.path(), &["00_nucleo", "01_core"]);
        let destination = root.path().join("crystalline.toml");
        match mode {
            "internal" => {
                fs::write(root.path().join("target"), b"inside").unwrap();
                symlink("target", &destination).unwrap();
            }
            "external" => symlink(&target, &destination).unwrap(),
            "dangling" => symlink("missing", &destination).unwrap(),
            "loop" => symlink("crystalline.toml", &destination).unwrap(),
            "directory" => fs::create_dir(&destination).unwrap(),
            "fifo" => {
                assert!(Command::new("mkfifo")
                    .arg(&destination)
                    .status()
                    .unwrap()
                    .success());
            }
            _ => unreachable!(),
        }
        let output = init(root.path());
        assert_domain(&output, 2, b"", b"crystalline-lint: unsafe config initialization: config destination is not a regular file\n");
        assert_eq!(fs::read(&target).unwrap(), b"outside-winner");
        assert!(fs::symlink_metadata(&destination).is_ok());
    }
}

#[cfg(unix)]
#[test]
fn root_symlink_is_rejected_without_escape() {
    use std::os::unix::fs::symlink;
    let outside = tempfile::tempdir().unwrap();
    mkdirs(outside.path(), &["00_nucleo", "01_core"]);
    let holder = tempfile::tempdir().unwrap();
    let link = holder.path().join("project-link");
    symlink(outside.path(), &link).unwrap();
    let output = init(&link);
    assert_domain(&output, 2, b"", b"crystalline-lint: unsafe config initialization: project root is not a physical directory\n");
    assert!(!outside.path().join("crystalline.toml").exists());
}

#[test]
fn missing_root_and_non_directory_root_use_closed_unsafe_reasons() {
    let holder = tempfile::tempdir().unwrap();
    let missing = holder.path().join("missing");
    assert_domain(
        &init(&missing),
        2,
        b"",
        b"crystalline-lint: unsafe config initialization: project root does not exist\n",
    );
    let file = holder.path().join("file");
    fs::write(&file, b"sentinel").unwrap();
    assert_domain(&init(&file), 2, b"", b"crystalline-lint: unsafe config initialization: project root is not a physical directory\n");
    assert_eq!(fs::read(file).unwrap(), b"sentinel");
}

#[test]
fn lint_without_config_remains_read_only() {
    let root = tempfile::tempdir().unwrap();
    mkdirs(root.path(), &["00_nucleo", "01_core"]);
    let before: Vec<_> = fs::read_dir(root.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let _ = run(&[root.path().as_os_str()], None);
    let after: Vec<_> = fs::read_dir(root.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(before, after);
    assert!(!root.path().join("crystalline.toml").exists());
}

#[test]
fn every_lint_flag_is_rejected_after_init_without_writing() {
    let cases: &[&[&str]] = &[
        &["--config", "elsewhere.toml"],
        &["--fix-hashes"],
        &["--update-snapshot"],
        &["--dry-run"],
        &["--quiet"],
        &["--format", "json"],
        &["--checks", "V1"],
    ];
    for suffix in cases {
        let root = tempfile::tempdir().unwrap();
        mkdirs(root.path(), &["00_nucleo", "01_core"]);
        let mut owned = vec!["init".to_string(), root.path().display().to_string()];
        owned.extend(suffix.iter().map(|s| (*s).to_string()));
        let refs: Vec<_> = owned.iter().map(|s| s.as_ref()).collect();
        let output = run(&refs, None);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args={owned:?}, output={output:?}"
        );
        assert!(!root.path().join("crystalline.toml").exists());
    }
}

#[test]
fn init_is_reserved_but_dot_slash_init_remains_a_lint_path() {
    let parent = tempfile::tempdir().unwrap();
    let literal = parent.path().join("init");
    fs::create_dir(&literal).unwrap();
    mkdirs(&literal, &["00_nucleo", "01_core"]);
    let lint = run(&["./init".as_ref()], Some(parent.path()));
    assert_ne!(
        lint.status.code(),
        Some(2),
        "./init was parsed as the init subcommand: {lint:?}"
    );
    assert!(!literal.join("crystalline.toml").exists());
    let command = run(&["init".as_ref()], Some(literal.as_path()));
    assert_eq!(command.status.code(), Some(0));
    assert_eq!(fs::read(literal.join("crystalline.toml")).unwrap(), MINIMAL);
}
