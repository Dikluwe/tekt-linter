//! Gate B2 segregado — adapter L3 de frescura de citações V21.
//! Assessment 0017; identidade verifier/v21-l3/0017.

use crystalline_lint::contracts::citation_freshness::{
    CitationFreshness, CitationFreshnessResolver, CitationStaleReason, CitationUnknownReason,
};
use crystalline_lint::infra::citation_freshness::FsCitationFreshnessResolver;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(label: &str) -> Self {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "tekt-v21-b2-{label}-{}-{stamp}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("criar sandbox B2");
        Self { root }
    }

    fn resolver(&self, max_bytes: u64) -> FsCitationFreshnessResolver {
        FsCitationFreshnessResolver::new(self.root.clone(), max_bytes)
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("criar diretório de fixture");
        }
        fs::write(path, bytes).expect("escrever fixture B2");
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_state(
    resolver: &FsCitationFreshnessResolver,
    path: &str,
    line: usize,
    expected: CitationFreshness,
) {
    assert_eq!(resolver.resolve(path, line), expected, "{path}:{line}");
}

#[test]
fn existing_nonempty_lines_are_valid_including_unicode_crlf_and_final_line() {
    let sandbox = Sandbox::new("valid");
    sandbox.write("nested/oracle.rs", "α\r\nβeta\r\núltima".as_bytes());
    let resolver = sandbox.resolver(1024);
    for line in 1..=3 {
        assert_state(
            &resolver,
            "nested/oracle.rs",
            line,
            CitationFreshness::Valid,
        );
    }
    assert_state(&resolver, "nested/./oracle.rs", 2, CitationFreshness::Valid);
}

#[test]
fn missing_and_invalid_or_empty_lines_are_stale_with_exact_reason() {
    let sandbox = Sandbox::new("stale");
    sandbox.write("oracle.rs", b"first\n \t\r\nthird\n");
    let resolver = sandbox.resolver(1024);
    assert_state(
        &resolver,
        "absent.rs",
        1,
        CitationFreshness::Stale(CitationStaleReason::MissingFile),
    );
    for line in [0, 4, usize::MAX] {
        assert_state(
            &resolver,
            "oracle.rs",
            line,
            CitationFreshness::Stale(CitationStaleReason::InvalidLine),
        );
    }
    assert_state(
        &resolver,
        "oracle.rs",
        2,
        CitationFreshness::Stale(CitationStaleReason::EmptyLine),
    );
}

#[test]
fn lexical_escape_absolute_and_empty_paths_are_outside_root() {
    let sandbox = Sandbox::new("confinement");
    sandbox.write("inside.rs", b"safe\n");
    let resolver = sandbox.resolver(1024);
    for path in ["", "..", "../escape.rs", "nested/../../escape.rs"] {
        assert_state(
            &resolver,
            path,
            1,
            CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot),
        );
    }
    let absolute = sandbox.root.join("inside.rs");
    assert_state(
        &resolver,
        absolute.to_str().unwrap(),
        1,
        CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot),
    );
}

#[cfg(unix)]
#[test]
fn root_and_component_symlinks_are_rejected_even_when_they_point_inside() {
    use std::os::unix::fs::symlink;
    let sandbox = Sandbox::new("symlink-component");
    sandbox.write("real/oracle.rs", b"source\n");
    symlink(sandbox.root.join("real"), sandbox.root.join("alias")).unwrap();
    assert_state(
        &sandbox.resolver(1024),
        "alias/oracle.rs",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::Symlink),
    );

    let parent = Sandbox::new("symlink-root");
    let real_root = parent.root.join("real-root");
    fs::create_dir(&real_root).unwrap();
    fs::write(real_root.join("oracle.rs"), b"source\n").unwrap();
    let linked_root = parent.root.join("linked-root");
    symlink(&real_root, &linked_root).unwrap();
    let resolver = FsCitationFreshnessResolver::new(linked_root, 1024);
    assert_state(
        &resolver,
        "oracle.rs",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::Symlink),
    );
}

#[cfg(unix)]
#[test]
fn concurrent_directory_to_external_symlink_swap_never_becomes_valid() {
    use std::os::unix::fs::symlink;

    let sandbox = Sandbox::new("nofollow-race");
    let external = Sandbox::new("external-marker");
    external.write("oracle.rs", b"EXTERNAL_VALID_MARKER\n");

    let live = sandbox.root.join("switch");
    let parked = sandbox.root.join("parked");
    fs::create_dir(&live).unwrap();
    // O alvo interno nunca pode produzir Valid: quando existe, sua linha é vazia.
    fs::write(live.join("oracle.rs"), b" \n").unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let worker_live = live.clone();
    let worker_parked = parked.clone();
    let external_root = external.root.clone();
    let swapper = thread::spawn(move || {
        while !worker_stop.load(Ordering::Relaxed) {
            if fs::rename(&worker_live, &worker_parked).is_ok() {
                if symlink(&external_root, &worker_live).is_ok() {
                    thread::yield_now();
                    let _ = fs::remove_file(&worker_live);
                }
                let _ = fs::rename(&worker_parked, &worker_live);
            }
            thread::yield_now();
        }
        let _ = fs::remove_file(&worker_live);
        let _ = fs::rename(&worker_parked, &worker_live);
    });

    let resolver = sandbox.resolver(1024);
    for attempt in 0..20_000 {
        let state = resolver.resolve("switch/oracle.rs", 1);
        assert_ne!(
            state,
            CitationFreshness::Valid,
            "escape TOCTOU leu marcador externo na tentativa {attempt}"
        );
    }
    stop.store(true, Ordering::Relaxed);
    swapper.join().unwrap();
}

#[test]
fn concurrent_removal_of_an_empty_target_remains_fail_closed() {
    let sandbox = Sandbox::new("removal-race");
    let target = sandbox.root.join("volatile.rs");
    fs::write(&target, b"\n").unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let worker_target = target.clone();
    let remover = thread::spawn(move || {
        while !worker_stop.load(Ordering::Relaxed) {
            let _ = fs::remove_file(&worker_target);
            thread::yield_now();
            let _ = fs::write(&worker_target, b"\n");
        }
    });

    let resolver = sandbox.resolver(1024);
    for attempt in 0..10_000 {
        let state = resolver.resolve("volatile.rs", 1);
        assert_ne!(
            state,
            CitationFreshness::Valid,
            "remoção concorrente produziu validade na tentativa {attempt}"
        );
    }
    stop.store(true, Ordering::Relaxed);
    remover.join().unwrap();
}

#[test]
fn invalid_roots_and_directories_are_unknown_with_exact_reason() {
    let sandbox = Sandbox::new("invalid-root");
    let resolver = FsCitationFreshnessResolver::new(sandbox.root.join("missing-root"), 1024);
    assert_state(
        &resolver,
        "oracle.rs",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::InvalidRoot),
    );
    sandbox.write("root-file", b"not a directory");
    let resolver = FsCitationFreshnessResolver::new(sandbox.root.join("root-file"), 1024);
    assert_state(
        &resolver,
        "oracle.rs",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::InvalidRoot),
    );
    fs::create_dir(sandbox.root.join("directory-target")).unwrap();
    assert_state(
        &sandbox.resolver(1024),
        "directory-target",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::Io),
    );
}

#[test]
fn invalid_utf8_and_budget_limits_fail_closed() {
    let sandbox = Sandbox::new("bytes");
    sandbox.write("invalid.rs", &[0xff, b'\n']);
    sandbox.write("large.rs", b"12345\n");
    assert_state(
        &sandbox.resolver(1024),
        "invalid.rs",
        1,
        CitationFreshness::Unknown(CitationUnknownReason::InvalidUtf8),
    );
    for budget in [0, 5] {
        assert_state(
            &sandbox.resolver(budget),
            "large.rs",
            1,
            CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded),
        );
    }
    assert_state(
        &sandbox.resolver(6),
        "large.rs",
        1,
        CitationFreshness::Valid,
    );
}

#[test]
fn resolution_is_deterministic_and_does_not_mutate_project_bytes() {
    let sandbox = Sandbox::new("immutable");
    sandbox.write("oracle.rs", b"authority\nsecond\n");
    let path = sandbox.root.join("oracle.rs");
    let before = fingerprint(&path);
    let resolver = sandbox.resolver(1024);
    let first = resolver.resolve("oracle.rs", 2);
    for _ in 0..8 {
        assert_eq!(resolver.resolve("oracle.rs", 2), first);
    }
    assert_eq!(first, CitationFreshness::Valid);
    assert_eq!(fingerprint(&path), before);
}

fn fingerprint(path: &Path) -> (Vec<u8>, u64, Option<SystemTime>) {
    let metadata = fs::metadata(path).unwrap();
    (
        fs::read(path).unwrap(),
        metadata.len(),
        metadata.modified().ok(),
    )
}
