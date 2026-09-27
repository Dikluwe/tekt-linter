use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg(root)
        .args(["--checks", "v0"])
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn absent_default_is_allowed_but_absent_explicit_config_fails() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(root.path(), &[]).status.success());
    for quiet in [false, true] {
        let mut args = vec!["--config", "missing.toml"];
        if quiet {
            args.push("--quiet");
        }
        let out = run(root.path(), &args);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert_eq!(out.stderr.is_empty(), quiet);
    }
}
#[test]
fn existing_invalid_configuration_never_falls_back_and_respects_quiet() {
    for directory in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("crystalline.toml");
        if directory {
            fs::create_dir(&path).unwrap();
        } else {
            fs::write(&path, "invalid [toml").unwrap();
        }
        for quiet in [false, true] {
            let out = run(root.path(), if quiet { &["--quiet"] } else { &[] });
            assert_eq!(out.status.code(), Some(1));
            assert!(out.stdout.is_empty());
            assert_eq!(out.stderr.is_empty(), quiet);
        }
    }
}
#[cfg(unix)]
#[test]
fn dangling_default_config_is_a_read_error_not_an_absent_config() {
    let root = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink("missing.toml", root.path().join("crystalline.toml")).unwrap();
    for quiet in [false, true] {
        let out = run(root.path(), if quiet { &["--quiet"] } else { &[] });
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        if quiet {
            assert!(out.stderr.is_empty());
        } else {
            assert!(String::from_utf8_lossy(&out.stderr).contains("config error"));
        }
    }
}
#[cfg(unix)]
#[test]
fn valid_config_symlink_is_still_loaded_and_validated() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("settings.toml");
    std::os::unix::fs::symlink("settings.toml", root.path().join("crystalline.toml")).unwrap();
    fs::write(&target, "[layers]\nL1 = '01_core'\n").unwrap();
    assert!(run(root.path(), &[]).status.success());
    fs::write(&target, "[rules]\nV99 = { level = 'error' }\n").unwrap();
    let out = run(root.path(), &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("V99"));
}
