use crystalline_lint::contracts::prompt_provider::PromptProvider;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::infra::hash_writer::{compute_source_hash, write_hash, write_prompt_meta};
use crystalline_lint::infra::prompt_reader::{CachedPromptReader, FsPromptReader};
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
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let mut accepted_intermediate_symlink = Vec::new();
        if hashes.exists("prompts/dir-link/out.md") {
            accepted_intermediate_symlink.push("exists");
        }
        if hashes.read_hash("prompts/dir-link/out.md").is_some() {
            accepted_intermediate_symlink.push("reader");
        }
        if snapshots.read_snapshot("prompts/dir-link/out.md").is_some() {
            accepted_intermediate_symlink.push("snapshot");
        }
        assert!(
            accepted_intermediate_symlink.is_empty(),
            "intermediate path symlink accepted by: {accepted_intermediate_symlink:?}"
        );

        let real_parent = tempfile::tempdir().unwrap();
        fs::create_dir_all(real_parent.path().join("nucleo/prompts")).unwrap();
        fs::write(real_parent.path().join("nucleo/prompts/in.md"), SNAPSHOT).unwrap();
        let ancestor_holder = tempfile::tempdir().unwrap();
        symlink(real_parent.path(), ancestor_holder.path().join("link")).unwrap();
        let root_with_symlink_ancestor = ancestor_holder.path().join("link/nucleo");
        let ancestor_hashes = reader(&root_with_symlink_ancestor);
        let ancestor_snapshots = snapshot_reader(&root_with_symlink_ancestor);
        let mut accepted_root_ancestor_symlink = Vec::new();
        if ancestor_hashes.exists("prompts/in.md") {
            accepted_root_ancestor_symlink.push("exists");
        }
        if ancestor_hashes.read_hash("prompts/in.md").is_some() {
            accepted_root_ancestor_symlink.push("reader");
        }
        if ancestor_snapshots.read_snapshot("prompts/in.md").is_some() {
            accepted_root_ancestor_symlink.push("snapshot");
        }
        assert!(
            accepted_root_ancestor_symlink.is_empty(),
            "symlink ancestor of nucleo_root accepted by: {accepted_root_ancestor_symlink:?}"
        );

        let holder = tempfile::tempdir().unwrap();
        let linked_root = holder.path().join("linked-root");
        symlink(root.path(), &linked_root).unwrap();
        let linked_hashes = reader(&linked_root);
        let linked_snapshots = snapshot_reader(&linked_root);
        let mut accepted_root_symlink = Vec::new();
        if linked_hashes.exists("prompts/in.md") {
            accepted_root_symlink.push("exists");
        }
        if linked_hashes.read_hash("prompts/in.md").is_some() {
            accepted_root_symlink.push("reader");
        }
        if linked_snapshots.read_snapshot("prompts/in.md").is_some() {
            accepted_root_symlink.push("snapshot");
        }
        assert!(
            accepted_root_symlink.is_empty(),
            "root symlink accepted by: {accepted_root_symlink:?}"
        );
    }
}

#[test]
fn source_hash_is_byte_sensitive_except_for_one_canonical_header_meta_line() {
    fn hash_full(bytes: &[u8]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("source.rs");
        fs::write(&path, bytes).unwrap();
        compute_source_hash(&path).expect("fixture must be hashable")
    }
    fn hash_body(bytes: &[u8]) -> String {
        let mut source = b"//! @prompt-hash deadbeef\n".to_vec();
        source.extend_from_slice(bytes);
        hash_full(&source)
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
        if hash_body(left) == hash_body(right) {
            ignored.push(label);
        }
    }
    assert_eq!(
        hash_full(b"//! @prompt-hash 11111111\nfn x() {}\n"),
        hash_full(b"//! @prompt-hash 22222222\nfn x() {}\n"),
        "canonical header meta must be outside the hash domain"
    );
    let duplicate_dir = tempfile::tempdir().unwrap();
    let duplicate_path = duplicate_dir.path().join("duplicate.rs");
    fs::write(
        &duplicate_path,
        b"//! @prompt-hash 11111111\n//! @prompt-hash 22222222\nfn x() {}\n",
    )
    .unwrap();
    let mut invalid_meta_accepted = Vec::new();
    if compute_source_hash(&duplicate_path).is_some() {
        invalid_meta_accepted.push("duplicate");
    }
    for (name, malformed) in [
        ("short", b"//! @prompt-hash 1234567\nfn x() {}\n".as_slice()),
        (
            "uppercase",
            b"//! @prompt-hash ABCDEF12\nfn x() {}\n".as_slice(),
        ),
        (
            "non-hex",
            b"//! @prompt-hash zzzzzzzz\nfn x() {}\n".as_slice(),
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("malformed.rs");
        fs::write(&path, malformed).unwrap();
        if compute_source_hash(&path).is_some() {
            invalid_meta_accepted.push(name);
        }
    }
    let outside_header = tempfile::tempdir().unwrap();
    let outside_header_path = outside_header.path().join("outside-header.rs");
    fs::write(
        &outside_header_path,
        b"fn before() {}\n//! @prompt-hash deadbeef\n",
    )
    .unwrap();
    if compute_source_hash(&outside_header_path).is_some() {
        invalid_meta_accepted.push("outside-leading-header");
    }
    let prompt_root = tempfile::tempdir().unwrap();
    fs::create_dir(prompt_root.path().join("prompts")).unwrap();
    fs::write(
        prompt_root.path().join("prompts/fenced.md"),
        b"```text\nHash do C\xc3\xb3digo: deadbeef\n```\nbody\n",
    )
    .unwrap();
    let prompt_reader = reader(prompt_root.path());
    let fenced_before = prompt_reader.read_hash("prompts/fenced.md");
    fs::write(
        prompt_root.path().join("prompts/fenced.md"),
        b"```text\nHash do C\xc3\xb3digo: cafebabe\n```\nbody\n",
    )
    .unwrap();
    let fenced_after = prompt_reader.read_hash("prompts/fenced.md");
    if fenced_before.is_none() || fenced_after.is_none() || fenced_before == fenced_after {
        ignored.push("prompt-meta-inside-fence");
    }
    assert!(
        ignored.is_empty() && invalid_meta_accepted.is_empty(),
        "ignored byte mutations: {ignored:?}; accepted invalid meta: {invalid_meta_accepted:?}"
    );
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
    let fs_reader = reader(root.path());
    assert!(fs_reader.read_hash("prompts/exact.md").is_some());
    assert_eq!(fs_reader.read_hash("prompts/over.md"), None);

    let missing_path = "prompts/later.md";
    let cached_missing = CachedPromptReader::new(reader(root.path()));
    assert_eq!(cached_missing.read_hash(missing_path), None);
    fs::write(root.path().join(missing_path), b"created later\n").unwrap();
    assert_eq!(cached_missing.read_hash(missing_path), None);
    assert!(CachedPromptReader::new(reader(root.path()))
        .read_hash(missing_path)
        .is_some());

    let cached_value = CachedPromptReader::new(reader(root.path()));
    let first = cached_value.read_hash("prompts/exact.md").unwrap();
    fs::write(root.path().join("prompts/exact.md"), b"replacement\n").unwrap();
    assert_eq!(
        cached_value.read_hash("prompts/exact.md"),
        Some(first.clone())
    );
    assert_ne!(
        CachedPromptReader::new(reader(root.path())).read_hash("prompts/exact.md"),
        Some(first)
    );

    // SPEC-GAP(P3): the public API exposes no deterministic barrier between metadata,
    // read and hash for a TOCTOU assertion.
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
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let holder = tempfile::tempdir().unwrap();
        let linked_root = holder.path().join("linked-project-root");
        symlink(first.path(), &linked_root).unwrap();
        assert!(
            FsPromptWalker::new(linked_root, HashSet::new())
                .scan()
                .is_err(),
            "walker accepted a symlink project root"
        );
    }
}

#[test]
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
        ("tilde-fenced-decoy", format!("## Interface Snapshot\n~~~html\n<!-- crystalline-snapshot: {valid_json} -->\n~~~\n")),
        ("missing-section", format!("<!-- crystalline-snapshot: {valid_json} -->\n")),
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

    for (name, bytes) in [
        ("absent", b"fn no_meta() {}\r\n".as_slice()),
        (
            "duplicate",
            b"//! @prompt-hash deadbeef\r\n//! @prompt-hash cafebabe\r\n".as_slice(),
        ),
        (
            "malformed",
            b"//! @prompt-hash NOTHEX!!\r\nfn malformed() {}\r\n".as_slice(),
        ),
    ] {
        let path = root.path().join(format!("{name}.rs"));
        fs::write(&path, bytes).unwrap();
        let before = fs::read(&path).unwrap();
        if write_hash(&path, "0123abcd").is_ok() {
            failures.push(format!("write_hash accepted {name} source meta"));
        }
        if fs::read(&path).unwrap() != before {
            failures.push(format!("write_hash changed {name} source meta fixture"));
        }
    }

    let substring_only = root.path().join("substring-only.md");
    let substring_bytes =
        b"Header\r\nBody mentions Hash do C\xc3\xb3digo: but has no canonical meta\r\n";
    fs::write(&substring_only, substring_bytes).unwrap();
    fs::set_permissions(&substring_only, fs::Permissions::from_mode(0o640)).unwrap();
    if write_prompt_meta(&substring_only, "0123abcd").is_ok() {
        failures.push("write_prompt_meta accepted substring decoy without meta".to_string());
    }
    if fs::read(&substring_only).unwrap() != substring_bytes {
        failures.push("write_prompt_meta changed substring-only fixture".to_string());
    }
    if fs::metadata(&substring_only).unwrap().mode() & 0o777 != 0o640 {
        failures.push("write_prompt_meta changed substring-only mode".to_string());
    }

    let prompt = root.path().join("prompt.md");
    fs::write(
        &prompt,
        b"Header\r\nHash do C\xc3\xb3digo: deadbeef\r\nBody Hash do C\xc3\xb3digo: decoy",
    )
    .unwrap();
    fs::set_permissions(&prompt, fs::Permissions::from_mode(0o640)).unwrap();
    let prompt_mode = fs::metadata(&prompt).unwrap().mode() & 0o777;
    match write_prompt_meta(&prompt, "89abcdef") {
        Ok(()) => {
            if fs::read(&prompt).unwrap()
                != b"Header\r\nHash do C\xc3\xb3digo: 89abcdef\r\nBody Hash do C\xc3\xb3digo: decoy"
            {
                failures.push("write_prompt_meta changed unauthorized bytes".to_string());
            }
        }
        Err(error) => failures.push(format!(
            "write_prompt_meta rejected canonical meta with body decoy: {error}"
        )),
    }
    if fs::metadata(&prompt).unwrap().mode() & 0o777 != prompt_mode {
        failures.push("write_prompt_meta changed mode".to_string());
    }

    use std::sync::{Arc, Barrier};
    let concurrent = root.path().join("concurrent.rs");
    fs::write(&concurrent, original).unwrap();
    fs::set_permissions(&concurrent, fs::Permissions::from_mode(0o640)).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles: Vec<_> = ["11111111", "22222222"]
        .into_iter()
        .map(|digest| {
            let path = concurrent.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                write_hash(&path, digest)
            })
        })
        .collect();
    barrier.wait();
    let outcomes: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    let final_bytes = fs::read(&concurrent).unwrap();
    let valid_a = b"//! @prompt-hash 11111111\r\nconst S: &str = \"@prompt-hash decoy\";\r\n";
    let valid_b = b"//! @prompt-hash 22222222\r\nconst S: &str = \"@prompt-hash decoy\";\r\n";
    if final_bytes != valid_a && final_bytes != valid_b {
        failures.push(format!(
            "concurrent writers produced corrupt bytes: {outcomes:?}"
        ));
    }
    if fs::metadata(&concurrent).unwrap().mode() & 0o777 != 0o640 {
        failures.push("concurrent writers changed mode".to_string());
    }
    if fs::read_dir(root.path())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("tmp"))
    {
        failures.push("writer left temporary residue after concurrent writes".to_string());
    }

    // SPEC-GAP(P6): a destination-swap TOCTOU still requires a public filesystem seam.
    assert!(
        failures.is_empty(),
        "writer contract failures: {failures:?}"
    );
}
