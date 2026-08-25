use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crystalline_lint::shell::fix_hashes::{
    execute, FixEntry, FixResult, FixUnavailable, HashRewriter,
};

#[derive(Default)]
struct Spy {
    calls: RefCell<Vec<String>>,
}

impl Spy {
    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl HashRewriter for Spy {
    fn read_header(&self, _path: &Path) -> Option<(String, String)> {
        panic!("execution must not read headers")
    }

    fn compute_hash(&self, _prompt_path: &str) -> Option<String> {
        panic!("execution must not compute prompt hashes")
    }

    fn compute_source_hash(&self, _source_path: &Path) -> Option<String> {
        panic!("execution must not compute source hashes")
    }

    fn write_hash(&self, source_path: &Path, new_hash: &str) -> Result<(), String> {
        self.calls
            .borrow_mut()
            .push(format!("code:{}:{new_hash}", source_path.display()));
        if new_hash == "fail-code" {
            Err("code: denied / exact".into())
        } else {
            Ok(())
        }
    }

    fn write_prompt_meta(&self, prompt_path: &str, source_hash: &str) -> Result<(), String> {
        self.calls
            .borrow_mut()
            .push(format!("prompt:{prompt_path}:{source_hash}"));
        if source_hash == "fail-prompt" {
            Err("prompt: denied / exact".into())
        } else {
            Ok(())
        }
    }
}

fn ready(source: &str, prompt: &str, old: &str, new: &str, source_hash: &str) -> FixEntry {
    FixEntry::Ready {
        source_path: PathBuf::from(source),
        prompt_path: prompt.into(),
        old_hash: old.into(),
        new_hash: new.into(),
        source_hash: source_hash.into(),
    }
}

#[test]
fn unavailable_and_dry_run_each_produce_one_result_and_never_write() {
    let unavailable = FixEntry::Unavailable {
        source_path: PathBuf::from("src/unreadable.rs"),
        reason: FixUnavailable::HeaderUnreadable,
    };
    let actionable = ready(
        "src/dry.rs",
        "prompts/dry.md",
        "old-hash",
        "hash-a",
        "hash-b",
    );
    let spy = Spy::default();

    let results = execute(&[unavailable.clone(), actionable], &spy, true);

    assert_eq!(
        results,
        vec![
            FixResult::Unavailable {
                source_path: PathBuf::from("src/unreadable.rs"),
                reason: FixUnavailable::HeaderUnreadable,
            },
            FixResult::DryRun {
                source_path: PathBuf::from("src/dry.rs"),
                prompt_path: "prompts/dry.md".into(),
                old_hash: "old-hash".into(),
                new_hash: "hash-a".into(),
                source_hash: "hash-b".into(),
            },
        ]
    );
    assert!(spy.calls().is_empty());
}

#[test]
fn successful_execution_writes_code_then_prompt_and_reports_applied() {
    let spy = Spy::default();
    let entries = [ready("src/a.rs", "prompts/a.md", "old", "hash-a", "hash-b")];

    let results = execute(&entries, &spy, false);

    assert_eq!(
        spy.calls(),
        vec!["code:src/a.rs:hash-a", "prompt:prompts/a.md:hash-b"]
    );
    assert_eq!(
        results,
        vec![FixResult::Applied {
            source_path: PathBuf::from("src/a.rs"),
            prompt_path: "prompts/a.md".into(),
            new_hash: "hash-a".into(),
            source_hash: "hash-b".into(),
        }]
    );
}

#[test]
fn code_failure_blocks_phase_two_but_does_not_block_later_entries() {
    let spy = Spy::default();
    let entries = [
        ready(
            "src/bad.rs",
            "prompts/bad.md",
            "old-1",
            "fail-code",
            "unused-b",
        ),
        ready(
            "src/next.rs",
            "prompts/next.md",
            "old-2",
            "next-a",
            "next-b",
        ),
    ];

    let results = execute(&entries, &spy, false);

    assert_eq!(
        spy.calls(),
        vec![
            "code:src/bad.rs:fail-code",
            "code:src/next.rs:next-a",
            "prompt:prompts/next.md:next-b",
        ]
    );
    assert_eq!(
        results,
        vec![
            FixResult::CodeWriteFailed {
                source_path: PathBuf::from("src/bad.rs"),
                prompt_path: "prompts/bad.md".into(),
                new_hash: "fail-code".into(),
                source_hash: "unused-b".into(),
                reason: "code: denied / exact".into(),
            },
            FixResult::Applied {
                source_path: PathBuf::from("src/next.rs"),
                prompt_path: "prompts/next.md".into(),
                new_hash: "next-a".into(),
                source_hash: "next-b".into(),
            },
        ]
    );
}

#[test]
fn prompt_failure_is_exact_partial_write_and_duplicates_are_preserved() {
    let spy = Spy::default();
    let duplicate = ready(
        "src/duplicate.rs",
        "prompts/duplicate.md",
        "old",
        "applied-a",
        "fail-prompt",
    );

    let results = execute(&[duplicate.clone(), duplicate], &spy, false);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0], results[1]);
    assert_eq!(
        results[0],
        FixResult::PartialWrite {
            source_path: PathBuf::from("src/duplicate.rs"),
            prompt_path: "prompts/duplicate.md".into(),
            applied_new_hash: "applied-a".into(),
            rejected_source_hash: "fail-prompt".into(),
            reason: "prompt: denied / exact".into(),
        }
    );
    assert_eq!(
        spy.calls(),
        vec![
            "code:src/duplicate.rs:applied-a",
            "prompt:prompts/duplicate.md:fail-prompt",
            "code:src/duplicate.rs:applied-a",
            "prompt:prompts/duplicate.md:fail-prompt",
        ]
    );
}
