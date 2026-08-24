use crystalline_lint::contracts::prompt_provider::PromptProvider;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::infra::hash_writer::{compute_source_hash, write_hash, write_prompt_meta};
use crystalline_lint::infra::prompt_reader::FsPromptReader;
use crystalline_lint::infra::prompt_snapshot_reader::FsPromptSnapshotReader;
use crystalline_lint::infra::prompt_walker::FsPromptWalker;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const SNAPSHOT: &str = "## Interface Snapshot\n<!-- GENERATED — não edite manualmente -->\n<!-- crystalline-snapshot: {\"functions\":[],\"types\":[],\"reexports\":[]} -->\n";

fn reader(root: &Path) -> FsPromptReader {
    FsPromptReader {
        nucleo_root: root.to_owned(),
    }
}

fn snapshot_reader(root: &Path) -> FsPromptSnapshotReader {
    FsPromptSnapshotReader {
        nucleo_root: root.to_owned(),
    }
}

#[test]
#[ignore = "RED congelado: readers escapam da raiz e exists aceita não-arquivos"]
fn readers_confine_paths_and_exists_means_local_regular_file() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("prompts")).unwrap();
    fs::write(root.path().join("prompts/in.md"), SNAPSHOT).unwrap();
    fs::create_dir(root.path().join("prompts/dir.md")).unwrap();
    fs::write(outside.path().join("out.md"), SNAPSHOT).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(
            outside.path().join("out.md"),
            root.path().join("prompts/file-link.md"),
        )
        .unwrap();
        symlink(outside.path(), root.path().join("prompts/dir-link")).unwrap();
    }

    let hashes = reader(root.path());
    let snapshots = snapshot_reader(root.path());
    assert!(hashes.read_hash("prompts/in.md").is_some());
    assert!(snapshots.read_snapshot("prompts/in.md").is_some());
    assert!(hashes.exists("prompts/in.md"));

    let absolute = outside.path().join("out.md").to_string_lossy().into_owned();
    let mut invalid = vec![
        "".to_string(),
        ".".to_string(),
        "../outside/out.md".to_string(),
        absolute,
        "prompts/../prompts/in.md".to_string(),
        "prompts/dir.md".to_string(),
    ];
    #[cfg(unix)]
    invalid.extend([
        "prompts/file-link.md".to_string(),
        "prompts/dir-link/out.md".to_string(),
    ]);

    let violations: Vec<_> = invalid
        .iter()
        .filter(|path| {
            hashes.exists(path)
                || hashes.read_hash(path).is_some()
                || snapshots.read_snapshot(path).is_some()
        })
        .cloned()
        .collect();
    assert!(
        violations.is_empty(),
        "accepted unsafe paths: {violations:?}"
    );
}

#[test]
#[ignore = "RED congelado: hash normaliza bytes não autorizados"]
fn source_hash_is_byte_sensitive_except_for_one_canonical_header_meta_line() {
    fn hash(bytes: &[u8]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("source.rs");
        fs::write(&path, bytes).unwrap();
        compute_source_hash(&path).expect("fixture must be hashable")
    }

    let mut ignored = Vec::new();
    for (label, left, right) in [
        (
            "final-newline",
            b"fn x() {}".as_slice(),
            b"fn x() {}\n".as_slice(),
        ),
        ("crlf", b"a\nb\n".as_slice(), b"a\r\nb\r\n".as_slice()),
        ("trailing-space", b"a\n".as_slice(), b"a \n".as_slice()),
        ("bom", b"a\n".as_slice(), b"\xef\xbb\xbfa\n".as_slice()),
        (
            "body-decoy",
            b"const S: &str = \"@prompt-hash one\";\n".as_slice(),
            b"const S: &str = \"@prompt-hash two\";\n".as_slice(),
        ),
    ] {
        if hash(left) == hash(right) {
            ignored.push(label);
        }
    }
    assert_eq!(
        hash(b"//! @prompt-hash 11111111\nfn x() {}\n"),
        hash(b"//! @prompt-hash 22222222\nfn x() {}\n"),
        "canonical header meta must be outside the hash domain"
    );
    assert!(ignored.is_empty(), "ignored byte mutations: {ignored:?}");
}

#[test]
fn prompt_reader_enforces_the_ten_mib_capture_limit() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("prompts")).unwrap();
    fs::write(
        root.path().join("prompts/exact.md"),
        vec![b'a'; 10 * 1024 * 1024],
    )
    .unwrap();
    fs::write(
        root.path().join("prompts/over.md"),
        vec![b'a'; 10 * 1024 * 1024 + 1],
    )
    .unwrap();
    let reader = reader(root.path());
    assert!(reader.read_hash("prompts/exact.md").is_some());
    assert_eq!(reader.read_hash("prompts/over.md"), None);

    // SPEC-GAP(P3): cache freshness is not specified, and the public API exposes no
    // deterministic barrier between metadata, read and hash for a TOCTOU assertion.
}

fn build_prompt_tree(root: &Path, reverse: bool) {
    let entries = [
        "00_nucleo/prompts/a.md",
        "00_nucleo/prompts/nested/a.md",
        "00_nucleo/prompts/a.md.bak",
        "00_nucleo/prompts/exact.md",
    ];
    let order: Vec<_> = if reverse {
        entries.iter().rev().collect()
    } else {
        entries.iter().collect()
    };
    for relative in order {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "prompt\n").unwrap();
    }
}

fn scan_paths(root: &Path) -> HashSet<String> {
    FsPromptWalker::new(
        root.to_owned(),
        HashSet::from(["00_nucleo/prompts/exact.md".to_string()]),
    )
    .scan()
    .unwrap()
    .entries
    .iter()
    .map(|entry| entry.relative_path.to_string())
    .collect()
}

#[test]
#[ignore = "RED congelado: prompt walker aceita raiz que é arquivo"]
fn prompt_walker_has_exact_deterministic_local_set_and_rejects_invalid_root() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    build_prompt_tree(first.path(), false);
    build_prompt_tree(second.path(), true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("outside.md"), "outside\n").unwrap();
        symlink(
            outside.path().join("outside.md"),
            first.path().join("00_nucleo/prompts/external.md"),
        )
        .unwrap();
        symlink(
            first.path().join("00_nucleo/prompts/a.md"),
            first.path().join("00_nucleo/prompts/internal.md"),
        )
        .unwrap();
    }
    let expected = HashSet::from([
        "00_nucleo/prompts/a.md".to_string(),
        "00_nucleo/prompts/nested/a.md".to_string(),
    ]);
    assert_eq!(scan_paths(first.path()), expected);
    assert_eq!(scan_paths(second.path()), expected);

    let invalid = tempfile::tempdir().unwrap();
    fs::create_dir(invalid.path().join("00_nucleo")).unwrap();
    fs::write(invalid.path().join("00_nucleo/prompts"), "not a directory").unwrap();
    assert!(
        FsPromptWalker::new(invalid.path().to_owned(), HashSet::new())
            .scan()
            .is_err(),
        "regular file used as prompts root produced a complete scan"
    );
}

#[test]
#[ignore = "RED congelado: snapshot aceita marcadores e schema ambíguos"]
fn snapshot_requires_one_canonical_marker_and_closed_complete_schema() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("prompts")).unwrap();
    let reader = snapshot_reader(root.path());
    fs::write(root.path().join("prompts/value.md"), SNAPSHOT).unwrap();
    assert!(reader.read_snapshot("prompts/value.md").is_some());

    let valid_json = "{\"functions\":[],\"types\":[],\"reexports\":[]}";
    let cases = [
        ("paragraph-decoy", format!("text crystalline-snapshot: {valid_json}\n")),
        ("fenced-decoy", format!("```\n<!-- crystalline-snapshot: {valid_json} -->\n```\n")),
        ("duplicate", format!("{SNAPSHOT}<!-- crystalline-snapshot: {valid_json} -->\n")),
        ("missing-field", "## Interface Snapshot\n<!-- crystalline-snapshot: {\"functions\":[],\"types\":[]} -->\n".to_string()),
        ("unknown-field", "## Interface Snapshot\n<!-- crystalline-snapshot: {\"functions\":[],\"types\":[],\"reexports\":[],\"alien\":true} -->\n".to_string()),
        ("truncated", "## Interface Snapshot\n<!-- crystalline-snapshot: {\"functions\":[] -->\n".to_string()),
    ];
    let mut accepted = Vec::new();
    for (name, content) in cases {
        fs::write(root.path().join("prompts/value.md"), content).unwrap();
        if reader.read_snapshot("prompts/value.md").is_some() {
            accepted.push(name);
        }
    }
    assert!(
        accepted.is_empty(),
        "accepted invalid snapshots: {accepted:?}"
    );
}

#[cfg(unix)]
#[test]
#[ignore = "RED congelado: writers aceitam digest inválido e alteram bytes/mode"]
fn writers_validate_digest_and_preserve_unauthorized_bytes_mode_and_directory_state() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source.rs");
    let original = b"//! @prompt-hash deadbeef\r\nconst S: &str = \"@prompt-hash decoy\";\r\n";
    fs::write(&source, original).unwrap();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o640)).unwrap();
    let before_mode = fs::metadata(&source).unwrap().mode() & 0o777;
    let before_entries: HashSet<PathBuf> = fs::read_dir(root.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut failures = Vec::new();
    write_hash(&source, "0123abcd").unwrap();
    if fs::read(&source).unwrap()
        != b"//! @prompt-hash 0123abcd\r\nconst S: &str = \"@prompt-hash decoy\";\r\n"
    {
        failures.push("write_hash changed unauthorized bytes".to_string());
    }
    if fs::metadata(&source).unwrap().mode() & 0o777 != before_mode {
        failures.push("write_hash changed mode".to_string());
    }
    let after_entries: HashSet<PathBuf> = fs::read_dir(root.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    if after_entries != before_entries {
        failures.push("write_hash left temporary residue".to_string());
    }

    for digest in [
        "",
        "1234567",
        "123456789",
        "zzzzzzzz",
        "1234 678",
        "1234567\n",
    ] {
        fs::write(&source, original).unwrap();
        let stable = fs::read(&source).unwrap();
        if write_hash(&source, digest).is_ok() {
            failures.push(format!("write_hash accepted digest {digest:?}"));
        }
        if fs::read(&source).unwrap() != stable {
            failures.push(format!("invalid digest {digest:?} changed destination"));
        }
    }

    let prompt = root.path().join("prompt.md");
    fs::write(
        &prompt,
        b"Header\r\nHash do C\xc3\xb3digo: deadbeef\r\nBody Hash do C\xc3\xb3digo: decoy",
    )
    .unwrap();
    fs::set_permissions(&prompt, fs::Permissions::from_mode(0o640)).unwrap();
    let prompt_mode = fs::metadata(&prompt).unwrap().mode() & 0o777;
    write_prompt_meta(&prompt, "89abcdef").unwrap();
    if fs::read(&prompt).unwrap()
        != b"Header\r\nHash do C\xc3\xb3digo: 89abcdef\r\nBody Hash do C\xc3\xb3digo: decoy"
    {
        failures.push("write_prompt_meta changed unauthorized bytes".to_string());
    }
    if fs::metadata(&prompt).unwrap().mode() & 0o777 != prompt_mode {
        failures.push("write_prompt_meta changed mode".to_string());
    }

    // SPEC-GAP(P6): deterministic concurrency/TOCTOU requires a public barrier or
    // injectable filesystem; racing scheduler timing would make this gate flaky.
    assert!(
        failures.is_empty(),
        "writer contract failures: {failures:?}"
    );
}
