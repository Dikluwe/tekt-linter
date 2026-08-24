use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/segregated_refinement")
        .join(name)
}

fn hash(path: &Path) -> String {
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

fn git(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

struct Repository {
    dir: tempfile::TempDir,
    baseline: String,
    preserved: String,
    violated: String,
    unknown: String,
}

impl Repository {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(git(root, &["init", "-q"]).status.success());
        assert!(
            git(root, &["config", "user.email", "fixture@example.invalid"])
                .status
                .success()
        );
        assert!(git(root, &["config", "user.name", "Fixture"])
            .status
            .success());
        fs::create_dir_all(root.join("00_nucleo/refinement/contracts")).unwrap();
        fs::create_dir_all(root.join("00_nucleo/prompts")).unwrap();
        fs::copy(
            fixture("prompt.md"),
            root.join("00_nucleo/prompts/prompt.md"),
        )
        .unwrap();
        fs::copy(
            fixture("contract.toml"),
            root.join("00_nucleo/refinement/contracts/contract.toml"),
        )
        .unwrap();
        fs::copy(fixture("baseline.rs"), root.join("sample.rs")).unwrap();
        fs::write(
            root.join(".gitattributes"),
            "sample.rs filter=hostile\nlfs.bin filter=lfs\n",
        )
        .unwrap();
        fs::write(root.join("lfs.bin"), b"not an actual LFS pointer\n").unwrap();
        assert!(git(root, &["add", "."]).status.success());
        assert!(git(root, &["commit", "-qm", "baseline"]).status.success());
        let baseline = Self::oid(root);

        let preserved = Self::commit_state(root, "preserved.rs", "preserved");
        let violated = Self::commit_state(root, "violated.rs", "violated");
        let unknown = Self::commit_state(root, "unknown.rs", "unknown");
        Self {
            dir,
            baseline,
            preserved,
            violated,
            unknown,
        }
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn oid(root: &Path) -> String {
        String::from_utf8(git(root, &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned()
    }

    fn commit_state(root: &Path, source: &str, message: &str) -> String {
        fs::copy(fixture(source), root.join("sample.rs")).unwrap();
        Self::commit_sample(root, message)
    }

    fn commit_sample(root: &Path, message: &str) -> String {
        assert!(git(root, &["add", "sample.rs"]).status.success());
        assert!(git(root, &["commit", "-qm", message]).status.success());
        Self::oid(root)
    }

    fn manifest(&self, oracles: &[(&str, &str, &str, &str)], producers: [&str; 3]) -> PathBuf {
        let path = self.root().join("run.toml");
        let prompt = self.root().join("00_nucleo/prompts/prompt.md");
        let contract = self
            .root()
            .join("00_nucleo/refinement/contracts/contract.toml");
        let mut text = format!(
            "protocol_version = 1\nprompt = '00_nucleo/prompts/prompt.md'\nprompt_sha256 = '{}'\nbaseline_oid = '{}'\ncontract = '00_nucleo/refinement/contracts/contract.toml'\ncontract_sha256 = '{}'\ncontract_producer = '{}'\nimplementation_producer = '{}'\nverifier_producer = '{}'\nunknown_policy = 'block'\n",
            hash(&prompt), self.baseline, hash(&contract), producers[0], producers[1], producers[2]
        );
        for (id, kind, before, after) in oracles {
            text.push_str(&format!(
                "\n[[oracle]]\nid = '{id}'\nkind = '{kind}'\nbefore_ref = '{before}'\nafter_ref = '{after}'\n"
            ));
        }
        fs::write(&path, text).unwrap();
        path
    }

    fn valid_manifest(&self) -> PathBuf {
        self.manifest(
            &[
                ("positive", "positive", &self.baseline, &self.preserved),
                ("negative", "negative", &self.baseline, &self.violated),
                ("opaque", "unknown", &self.baseline, &self.unknown),
            ],
            ["contract-agent/a", "implementation-agent/b", "verifier/c"],
        )
    }

    fn seal(&self, manifest: &Path, output: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .args([
                "seal-refinement",
                self.root().to_str().unwrap(),
                "--manifest",
                manifest.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    }
}

fn assert_blocked(result: Output, output: &Path) {
    assert_eq!(
        result.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("unexpected argument"),
        "seal-refinement must be a recognized command before its validation can count as blocking"
    );
    assert!(!output.exists(), "a failed seal must not be published");
}

#[test]
fn missing_any_oracle_category_blocks() {
    let repo = Repository::new();
    let cases: &[(&str, &[(&str, &str, &str, &str)])] = &[
        ("all", &[]),
        (
            "positive",
            &[
                ("negative", "negative", &repo.baseline, &repo.violated),
                ("opaque", "unknown", &repo.baseline, &repo.unknown),
            ],
        ),
        (
            "negative",
            &[
                ("positive", "positive", &repo.baseline, &repo.preserved),
                ("opaque", "unknown", &repo.baseline, &repo.unknown),
            ],
        ),
        (
            "unknown",
            &[
                ("positive", "positive", &repo.baseline, &repo.preserved),
                ("negative", "negative", &repo.baseline, &repo.violated),
            ],
        ),
    ];
    let mut incorrectly_sealed = Vec::new();
    for (missing, oracles) in cases {
        let manifest = repo.manifest(oracles, ["a", "b", "c"]);
        let output = repo.root().join(format!("missing-category-{missing}.json"));
        let result = repo.seal(&manifest, &output);
        if result.status.code() == Some(0) {
            incorrectly_sealed.push(*missing);
        } else {
            assert_blocked(result, &output);
        }
    }
    assert!(
        incorrectly_sealed.is_empty(),
        "manifests missing these oracle categories were sealed: {incorrectly_sealed:?}"
    );
}

#[test]
fn negative_with_witness_decoy_and_inconclusive_blocks() {
    let mut repo = Repository::new();
    let contract = repo
        .root()
        .join("00_nucleo/refinement/contracts/contract.toml");
    fs::copy(fixture("decoy-contract.toml"), &contract).unwrap();

    fs::write(
        repo.root().join("sample.rs"),
        "enum Stable { Alpha, Beta }\nenum Required { Present }\n",
    )
    .unwrap();
    assert!(git(
        repo.root(),
        &["add", "sample.rs", contract.to_str().unwrap()]
    )
    .status
    .success());
    assert!(git(repo.root(), &["commit", "-qm", "decoy baseline"])
        .status
        .success());
    repo.baseline = Repository::oid(repo.root());

    fs::write(
        repo.root().join("sample.rs"),
        "// external rewrite\nenum Stable { Alpha, Beta }\nenum Required { Present }\n",
    )
    .unwrap();
    repo.preserved = Repository::commit_sample(repo.root(), "decoy positive");

    fs::write(
        repo.root().join("sample.rs"),
        "enum Stable { Alpha }\n// Required is missing: inconclusive alongside the decoy witness.\n",
    )
    .unwrap();
    repo.violated = Repository::commit_sample(repo.root(), "decoy negative");

    fs::write(
        repo.root().join("sample.rs"),
        "fn construct_dynamically() {}\n",
    )
    .unwrap();
    repo.unknown = Repository::commit_sample(repo.root(), "decoy unknown");

    let manifest = repo.valid_manifest();
    let output = repo.root().join("decoy-seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn valid_package_with_all_oracle_kinds_produces_seal() {
    let repo = Repository::new();
    let manifest = repo.valid_manifest();
    let output = repo.root().join("seal.json");
    let result = repo.seal(&manifest, &output);
    assert_eq!(
        result.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let seal: serde_json::Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
    assert_eq!(seal["protocol_version"], 1);
    assert_eq!(seal["baseline_oid"], repo.baseline);
    assert_eq!(seal["sealed"], true);
    assert_eq!(seal["mutation_score"], "1.0");
    assert_eq!(seal["counts"]["positive"], 1);
    assert_eq!(seal["counts"]["negative"], 1);
    assert_eq!(seal["counts"]["unknown"], 1);
}

#[test]
fn negative_preserved_blocks() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("negative", "negative", &repo.baseline, &repo.preserved)],
        ["a", "b", "c"],
    );
    let output = repo.root().join("seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn negative_unknown_blocks() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("negative", "negative", &repo.baseline, &repo.unknown)],
        ["a", "b", "c"],
    );
    let output = repo.root().join("seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn positive_violated_blocks() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("positive", "positive", &repo.baseline, &repo.violated)],
        ["a", "b", "c"],
    );
    let output = repo.root().join("seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn unknown_preserved_blocks() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("opaque", "unknown", &repo.baseline, &repo.preserved)],
        ["a", "b", "c"],
    );
    let output = repo.root().join("seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn divergent_prompt_or_contract_hash_blocks() {
    for field in ["prompt_sha256", "contract_sha256"] {
        let repo = Repository::new();
        let manifest = repo.valid_manifest();
        let text = fs::read_to_string(&manifest).unwrap();
        let needle = format!("{field} = '");
        let start = text.find(&needle).unwrap() + needle.len();
        let mut bytes = text.into_bytes();
        bytes[start] = if bytes[start] == b'0' { b'1' } else { b'0' };
        fs::write(&manifest, bytes).unwrap();
        let output = repo.root().join("seal.json");
        assert_blocked(repo.seal(&manifest, &output), &output);
    }
}

#[test]
fn repeated_producers_block() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("positive", "positive", &repo.baseline, &repo.preserved)],
        ["same", "same", "other"],
    );
    let output = repo.root().join("seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
}

#[test]
fn symbolic_or_mismatched_baseline_blocks() {
    for bad in [
        "HEAD".to_owned(),
        "0000000000000000000000000000000000000000".to_owned(),
    ] {
        let repo = Repository::new();
        let manifest = repo.valid_manifest();
        let text = fs::read_to_string(&manifest).unwrap().replace(
            &format!("baseline_oid = '{}'", repo.baseline),
            &format!("baseline_oid = '{bad}'"),
        );
        fs::write(&manifest, text).unwrap();
        let output = repo.root().join("seal.json");
        assert_blocked(repo.seal(&manifest, &output), &output);
    }
}

#[test]
fn manifest_order_does_not_change_seal_bytes() {
    let repo = Repository::new();
    let first = repo.valid_manifest();
    let output_a = repo.root().join("a.json");
    assert_eq!(repo.seal(&first, &output_a).status.code(), Some(0));
    let second = repo.manifest(
        &[
            ("opaque", "unknown", &repo.baseline, &repo.unknown),
            ("negative", "negative", &repo.baseline, &repo.violated),
            ("positive", "positive", &repo.baseline, &repo.preserved),
        ],
        ["contract-agent/a", "implementation-agent/b", "verifier/c"],
    );
    let output_b = repo.root().join("b.json");
    assert_eq!(repo.seal(&second, &output_b).status.code(), Some(0));
    assert_eq!(fs::read(output_a).unwrap(), fs::read(output_b).unwrap());
}

#[test]
fn failure_leaves_no_partial_or_temporary_output() {
    let repo = Repository::new();
    let manifest = repo.manifest(
        &[("negative", "negative", &repo.baseline, &repo.preserved)],
        ["a", "b", "c"],
    );
    let output = repo.root().join("nested/seal.json");
    assert_blocked(repo.seal(&manifest, &output), &output);
    if repo.root().join("nested").exists() {
        assert!(fs::read_dir(repo.root().join("nested"))
            .unwrap()
            .next()
            .is_none());
    }
    assert!(!fs::read_dir(repo.root()).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains("seal.json")));
}

#[test]
fn dirty_worktree_remains_byte_identical() {
    let repo = Repository::new();
    let manifest = repo.valid_manifest();
    fs::write(repo.root().join("dirty.txt"), b"do not touch\n").unwrap();
    fs::write(repo.root().join("sample.rs"), b"dirty tracked bytes\n").unwrap();
    let before_status = git(repo.root(), &["status", "--porcelain=v1", "-z"]).stdout;
    let before_sample = fs::read(repo.root().join("sample.rs")).unwrap();
    let output = repo.root().join("seal.json");
    assert_eq!(repo.seal(&manifest, &output).status.code(), Some(0));
    assert_eq!(
        fs::read(repo.root().join("sample.rs")).unwrap(),
        before_sample
    );
    fs::remove_file(output).unwrap();
    let after_status = git(repo.root(), &["status", "--porcelain=v1", "-z"]).stdout;
    assert_eq!(after_status, before_status);
    assert_eq!(
        fs::read(repo.root().join("dirty.txt")).unwrap(),
        b"do not touch\n"
    );
}

#[cfg(unix)]
#[test]
fn sealing_does_not_execute_checkout_hooks_filters_lfs_or_submodules() {
    use std::os::unix::fs::PermissionsExt;
    let repo = Repository::new();
    let marker = repo.root().join("host-code-executed");
    let hook = repo.root().join(".git/hooks/post-checkout");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).unwrap();
    let mut permissions = fs::metadata(&hook).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&hook, permissions).unwrap();
    assert!(git(
        repo.root(),
        &[
            "config",
            "filter.lfs.smudge",
            &format!("touch {}", marker.display())
        ]
    )
    .status
    .success());
    assert!(git(
        repo.root(),
        &[
            "config",
            "filter.hostile.smudge",
            &format!("touch {}", marker.display())
        ]
    )
    .status
    .success());
    assert!(git(
        repo.root(),
        &[
            "config",
            "submodule.hostile.url",
            "https://example.invalid/must-not-connect"
        ]
    )
    .status
    .success());
    let manifest = repo.valid_manifest();
    let output = repo.root().join("seal.json");
    assert_eq!(repo.seal(&manifest, &output).status.code(), Some(0));
    assert!(!marker.exists());
}
