#![cfg(unix)]

use crystalline_lint::infra::git_refinement::{
    load_revision_with_git, GitPathContent, GitRevisionError,
};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const SENTINEL: &[u8] = b"P0102-external-object-sentinel\n";

#[derive(Clone, Copy)]
enum RaceKind {
    Loose,
    Fanout,
    PackPair,
}

struct Fixture {
    _temp: TempDir,
    repository: PathBuf,
    wrapper: PathBuf,
    swapped: PathBuf,
    restored: PathBuf,
}

fn run(cwd: &Path, program: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new(program)
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("fixture command must start");
    assert!(
        output.status.success(),
        "fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

fn make_writable(path: &Path) {
    let mut permissions = fs::metadata(path).expect("object metadata").permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(path, permissions).expect("make fixture object writable");
}

fn fixture(kind: RaceKind, race: bool) -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let repository = temp.path().join("repository");
    let external = temp.path().join("external-object-store");
    fs::create_dir_all(&repository).expect("repository directory");
    fs::create_dir_all(&external).expect("external directory");

    let git = PathBuf::from(
        String::from_utf8(run(
            Path::new("/"),
            Path::new("sh"),
            &["-c", "command -v git"],
        ))
        .expect("git path utf8")
        .trim(),
    );
    run(&repository, &git, &["init", "-q"]);
    run(
        &repository,
        &git,
        &["config", "user.email", "p0102@example.invalid"],
    );
    run(&repository, &git, &["config", "user.name", "P0102 fixture"]);
    fs::write(repository.join("artifact.txt"), SENTINEL).expect("fixture artifact");
    run(&repository, &git, &["add", "artifact.txt"]);
    run(&repository, &git, &["commit", "-qm", "fixture"]);

    let oid = String::from_utf8(run(&repository, &git, &["rev-parse", "HEAD:artifact.txt"]))
        .expect("oid utf8")
        .trim()
        .to_owned();
    let objects = repository.join(".git/objects");

    let (first, second) = match kind {
        RaceKind::Loose | RaceKind::Fanout => {
            let (fanout, suffix) = oid.split_at(2);
            let internal_object = objects.join(fanout).join(suffix);
            let external_fanout = external.join(fanout);
            fs::create_dir_all(&external_fanout).expect("external fanout");
            fs::copy(&internal_object, external_fanout.join(suffix)).expect("copy loose object");
            if race {
                make_writable(&internal_object);
                fs::write(&internal_object, b"regular-but-invalid-loose-object")
                    .expect("corrupt internal loose object");
            }
            match kind {
                RaceKind::Loose => (internal_object, external_fanout.join(suffix)),
                RaceKind::Fanout => (objects.join(fanout), external_fanout),
                RaceKind::PackPair => unreachable!(),
            }
        }
        RaceKind::PackPair => {
            run(&repository, &git, &["gc", "--prune=now", "-q"]);
            let pack_dir = objects.join("pack");
            let pack = fs::read_dir(&pack_dir)
                .expect("pack directory")
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| path.extension() == Some(OsStr::new("pack")))
                .expect("pack file");
            let idx = pack.with_extension("idx");
            let external_pack = external.join(pack.file_name().expect("pack basename"));
            let external_idx = external.join(idx.file_name().expect("idx basename"));
            fs::copy(&pack, &external_pack).expect("copy pack");
            fs::copy(&idx, &external_idx).expect("copy idx");
            (pack, idx)
        }
    };

    let swapped = temp.path().join("swap-observed");
    let restored = temp.path().join("restore-observed");
    let wrapper = temp.path().join("git-wrapper.sh");
    let script = if !race {
        format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote(&git))
    } else {
        let swap = match kind {
            RaceKind::Loose => format!(
                "mv {a} {a}.saved\nln -s {b} {a}\n",
                a = shell_quote(&first),
                b = shell_quote(&second)
            ),
            RaceKind::Fanout => format!(
                "mv {a} {a}.saved\nln -s {b} {a}\n",
                a = shell_quote(&first),
                b = shell_quote(&second)
            ),
            RaceKind::PackPair => {
                let external_pack = external.join(first.file_name().expect("pack basename"));
                let external_idx = external.join(second.file_name().expect("idx basename"));
                format!(
                    "mv {a} {a}.saved\nmv {b} {b}.saved\nln -s {ea} {a}\nln -s {eb} {b}\n",
                    a = shell_quote(&first),
                    b = shell_quote(&second),
                    ea = shell_quote(&external_pack),
                    eb = shell_quote(&external_idx)
                )
            }
        };
        let restore = match kind {
            RaceKind::Loose | RaceKind::Fanout => {
                format!("rm {a}\nmv {a}.saved {a}\n", a = shell_quote(&first))
            }
            RaceKind::PackPair => format!(
                "rm {a} {b}\nmv {a}.saved {a}\nmv {b}.saved {b}\n",
                a = shell_quote(&first),
                b = shell_quote(&second)
            ),
        };
        format!(
            "#!/bin/sh\ncase \" $* \" in\n  *' cat-file '*)\n{swap}touch {swapped}\n{git} \"$@\"\nstatus=$?\n{restore}touch {restored}\nexit $status\n;;\n  *) exec {git} \"$@\";;\nesac\n",
            swap = swap,
            restore = restore,
            swapped = shell_quote(&swapped),
            restored = shell_quote(&restored),
            git = shell_quote(&git),
        )
    };
    fs::write(&wrapper, script).expect("write wrapper");
    let mut permissions = fs::metadata(&wrapper)
        .expect("wrapper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&wrapper, permissions).expect("wrapper permissions");

    Fixture {
        _temp: temp,
        repository,
        wrapper,
        swapped,
        restored,
    }
}

fn confront(kind: RaceKind) {
    let fixture = fixture(kind, true);
    let result = load_revision_with_git(
        &fixture.wrapper,
        &fixture.repository,
        OsStr::new("HEAD"),
        &[PathBuf::from("artifact.txt")],
    );
    assert!(
        fixture.swapped.is_file(),
        "fixture must prove the transient swap"
    );
    assert!(
        fixture.restored.is_file(),
        "fixture must prove restoration before adapter return"
    );
    if let Ok(content) = &result {
        assert_eq!(
            content.paths.get(Path::new("artifact.txt")),
            Some(&GitPathContent::Blob(SENTINEL.to_vec())),
            "an accepted race must demonstrably publish the external sentinel"
        );
    }
    assert_eq!(result, Err(GitRevisionError::ContainmentFailure));
}

fn positive_control(kind: RaceKind) {
    let fixture = fixture(kind, false);
    let result = load_revision_with_git(
        &fixture.wrapper,
        &fixture.repository,
        OsStr::new("HEAD"),
        &[PathBuf::from("artifact.txt")],
    );
    let debug = format!("{result:?}");
    assert!(
        result.is_ok(),
        "regular internal object must remain readable: {debug}"
    );
}

#[test]
fn transient_loose_object_symlink_is_rejected() {
    confront(RaceKind::Loose);
}

#[test]
fn transient_fanout_symlink_is_rejected() {
    confront(RaceKind::Fanout);
}

#[test]
fn transient_pack_and_index_symlinks_are_rejected() {
    confront(RaceKind::PackPair);
}

#[test]
fn regular_internal_loose_object_is_accepted() {
    positive_control(RaceKind::Loose);
}

#[test]
fn regular_internal_fanout_is_accepted() {
    positive_control(RaceKind::Fanout);
}

#[test]
fn regular_internal_pack_and_index_are_accepted() {
    positive_control(RaceKind::PackPair);
}
