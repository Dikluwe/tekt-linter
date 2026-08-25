//! P0101/B3 — independent gate for the productive `refine-revisions` route.
//!
//! The gate intentionally enters through the published command. Exit-code policy is
//! reserved to F09 and is therefore never used as an oracle here.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const BEFORE_OID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const AFTER_OID: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BLOB_OID: &str = "cccccccccccccccccccccccccccccccccccccccc";
const SOURCE: &str = "enum Tone { Dark }\n";

struct Fixture {
    temp: TempDir,
    repository: PathBuf,
    contract: PathBuf,
    transcript: PathBuf,
    fake_bin: PathBuf,
}

impl Fixture {
    fn new(valid_framing: bool) -> Self {
        let temp = tempfile::tempdir().expect("temporary fixture");
        let repository = temp.path().join("repository");
        fs::create_dir_all(repository.join(".git/objects/info")).unwrap();
        fs::create_dir_all(repository.join(".git/objects/pack")).unwrap();
        fs::write(repository.join(".git/config"), b"[core]\n\tbare = false\n").unwrap();

        let contract = temp.path().join("contract.toml");
        fs::write(
            &contract,
            r#"id = "productive-route"

[[observable]]
key = "tone"
language = "rust"
file = "sample.rs"
query = '''
(enum_item name: (type_identifier) @value)
'''
capture = "value"
cardinality = "one"
on_missing = "unknown"

[[relation]]
kind = "preserve"
source = "tone"
target = "tone"
"#,
        )
        .unwrap();

        let transcript = repository.join(".git/productive-route.transcript");
        let fake_bin = temp.path().join("bin");
        fs::create_dir(&fake_bin).unwrap();
        let git = fake_bin.join("git");
        let rev_reply = if valid_framing {
            format!(
                "case \"$last\" in before^\\{{commit\\}}) printf '%s\\n' '{BEFORE_OID}' ;; after^\\{{commit\\}}) printf '%s\\n' '{AFTER_OID}' ;; *) exit 41 ;; esac"
            )
        } else {
            "printf 'not-an-oid\\n'".to_owned()
        };
        let transcript_literal = transcript
            .to_str()
            .expect("UTF-8 fixture path")
            .replace('\'', "'\\''");
        let script = format!(
            r#"#!/bin/sh
log='{transcript_literal}'
printf 'CALL' >> "$log"
last=''
for arg in "$@"; do
  printf '\t%s' "$arg" >> "$log"
  last="$arg"
done
printf '\n' >> "$log"
case " $* " in
  *' rev-parse '*) {rev_reply} ;;
  *' ls-tree '*) printf '100644 blob {BLOB_OID}\tsample.rs\000' ;;
  *' cat-file '*)
    while IFS= read -r request; do
      printf 'STDIN\t%s\n' "$request" >> "$log"
      case "$request" in
        'contents {BLOB_OID}') printf '{BLOB_OID} blob {source_len}\n{source}\n' ;;
        flush) : ;;
        *) exit 42 ;;
      esac
    done
    ;;
  *) exit 43 ;;
esac
"#,
            source_len = SOURCE.as_bytes().len(),
            source = SOURCE,
        );
        fs::write(&git, script).unwrap();
        let mut permissions = fs::metadata(&git).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&git, permissions).unwrap();

        Self {
            temp,
            repository,
            contract,
            transcript,
            fake_bin,
        }
    }

    fn run_productive(&self) -> Output {
        let _keep_alive = &self.temp;
        Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
            .arg("refine-revisions")
            .arg(&self.repository)
            .args([
                "--before-ref",
                "before",
                "--after-ref",
                "after",
                "--contract",
            ])
            .arg(&self.contract)
            .env("PATH", &self.fake_bin)
            .output()
            .expect("run productive consumer")
    }

    fn transcript(&self) -> String {
        fs::read_to_string(&self.transcript).unwrap_or_default()
    }
}

fn semantic_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn expected_prefix() -> Vec<&'static str> {
    vec![
        "-c",
        "protocol.allow=never",
        "-c",
        "core.hooksPath=",
        "-c",
        "core.fsmonitor=false",
        "-c",
        "credential.helper=",
        "-c",
        "diff.external=",
        "-c",
        "filter.lfs.process=",
        "-c",
        "filter.lfs.smudge=",
        "-c",
        "filter.lfs.clean=",
    ]
}

fn calls(transcript: &str) -> Vec<Vec<&str>> {
    transcript
        .lines()
        .filter_map(|line| line.strip_prefix("CALL\t"))
        .map(|line| line.split('\t').collect())
        .collect()
}

fn expected_call(suffix: &[&str]) -> Vec<String> {
    expected_prefix()
        .into_iter()
        .chain(suffix.iter().copied())
        .map(str::to_owned)
        .collect()
}

#[test]
fn productive_route_uses_one_controlled_transcript_and_oids_after_resolution() {
    let fixture = Fixture::new(true);
    let output = fixture.run_productive();
    let transcript = fixture.transcript();
    let calls = calls(&transcript);

    assert_eq!(
        calls.len(),
        6,
        "two revisions must use exactly three seam operations each\ntranscript={transcript}\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let prefix = expected_prefix();
    for call in &calls {
        assert!(
            call.len() >= prefix.len(),
            "Git operation omitted the controlled seam prefix\n{transcript}"
        );
        assert_eq!(
            &call[..prefix.len()],
            prefix,
            "historical/uncontrolled Git route reached\n{transcript}"
        );
    }

    let expected = vec![
        expected_call(&[
            "rev-parse",
            "--verify",
            "--end-of-options",
            "before^{commit}",
        ]),
        expected_call(&[
            "ls-tree",
            "-rz",
            "--full-tree",
            BEFORE_OID,
            "--",
            ":(top,literal)sample.rs",
        ]),
        expected_call(&["cat-file", "--batch-command", "--buffer"]),
        expected_call(&[
            "rev-parse",
            "--verify",
            "--end-of-options",
            "after^{commit}",
        ]),
        expected_call(&[
            "ls-tree",
            "-rz",
            "--full-tree",
            AFTER_OID,
            "--",
            ":(top,literal)sample.rs",
        ]),
        expected_call(&["cat-file", "--batch-command", "--buffer"]),
    ];
    let observed: Vec<Vec<String>> = calls
        .iter()
        .map(|call| call.iter().map(|arg| (*arg).to_owned()).collect())
        .collect();
    assert_eq!(
        observed, expected,
        "productive argv diverged from the L3 seam"
    );

    let rev_calls: Vec<_> = calls.iter().filter(|c| c.contains(&"rev-parse")).collect();
    assert_eq!(
        rev_calls.len(),
        2,
        "each ref must be resolved exactly once\n{transcript}"
    );
    assert_eq!(
        rev_calls
            .iter()
            .filter(|c| c.contains(&"before^{commit}"))
            .count(),
        1
    );
    assert_eq!(
        rev_calls
            .iter()
            .filter(|c| c.contains(&"after^{commit}"))
            .count(),
        1
    );

    for call in calls.iter().filter(|c| !c.contains(&"rev-parse")) {
        assert!(
            !call.contains(&"before") && !call.contains(&"after"),
            "symbolic ref escaped resolution\n{transcript}"
        );
    }
    assert!(transcript.contains(&format!("STDIN\tcontents {BLOB_OID}")));
    assert_eq!(transcript.matches("STDIN\tflush").count(), 2);
}

#[test]
fn framing_failure_stops_before_extraction_and_comparison() {
    let fixture = Fixture::new(false);
    let output = fixture.run_productive();
    let transcript = fixture.transcript();

    assert_eq!(
        calls(&transcript).len(),
        1,
        "failure must stop before tree/blob extraction\ntranscript={transcript}\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = semantic_stdout(&output).to_ascii_uppercase();
    assert!(
        !stdout.contains("PRESERVED")
            && !stdout.contains("VIOLATED")
            && !stdout.contains("UNKNOWN"),
        "L3 framing failure reached the comparator/presenter: {stdout}"
    );
}

#[test]
fn equal_source_bytes_have_the_same_observable_result() {
    let fixture = Fixture::new(true);
    let productive = fixture.run_productive();
    let productive_stdout = semantic_stdout(&productive);

    let before_root = fixture.temp.path().join("before-tree");
    let after_root = fixture.temp.path().join("after-tree");
    fs::create_dir(&before_root).unwrap();
    fs::create_dir(&after_root).unwrap();
    fs::write(before_root.join("sample.rs"), SOURCE).unwrap();
    fs::write(after_root.join("sample.rs"), SOURCE).unwrap();
    let before_snapshot = fixture.temp.path().join("before.json");
    let after_snapshot = fixture.temp.path().join("after.json");

    snapshot(&before_root, &fixture.contract, "before", &before_snapshot);
    snapshot(&after_root, &fixture.contract, "after", &after_snapshot);
    let explicit = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg("refine")
        .args(["--before"])
        .arg(&before_snapshot)
        .args(["--after"])
        .arg(&after_snapshot)
        .args(["--contract"])
        .arg(&fixture.contract)
        .output()
        .expect("run explicit B1 route");

    assert!(
        !productive_stdout.is_empty(),
        "productive route published no semantic result; stderr={}",
        String::from_utf8_lossy(&productive.stderr)
    );
    assert_eq!(productive_stdout, semantic_stdout(&explicit));
}

fn snapshot(root: &Path, contract: &Path, artifact_id: &str, output: &Path) {
    let result = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg("snapshot")
        .arg(root)
        .args(["--contract"])
        .arg(contract)
        .args(["--artifact-id", artifact_id, "--output"])
        .arg(output)
        .output()
        .expect("run B1 snapshot route");
    assert!(
        output.is_file(),
        "snapshot route did not publish facts: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}
