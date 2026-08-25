use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::shell::update_snapshot::{
    execute, SnapshotEntry, SnapshotResult, SnapshotRewriter, SnapshotUnreadable,
};

#[derive(Default)]
struct WriteSpy {
    calls: RefCell<Vec<(String, String)>>,
    outcomes: RefCell<VecDeque<Result<(), String>>>,
}

impl WriteSpy {
    fn with_outcomes(outcomes: impl IntoIterator<Item = Result<(), String>>) -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
            outcomes: RefCell::new(outcomes.into_iter().collect()),
        }
    }
}

impl SnapshotRewriter for WriteSpy {
    fn serialize_snapshot(&self, _interface: &PublicInterface<'_>) -> String {
        panic!("execute must not serialize snapshots")
    }

    fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String> {
        self.calls
            .borrow_mut()
            .push((prompt_path.to_owned(), snapshot.to_owned()));
        self.outcomes
            .borrow_mut()
            .pop_front()
            .expect("one configured outcome per expected write")
    }
}

#[test]
fn dry_run_preserves_every_entry_in_order_without_writing() {
    let entries = vec![
        SnapshotEntry::Unreadable {
            source_path: PathBuf::from("src/unreadable.rs"),
            reason: SnapshotUnreadable::MissingPromptHeader,
        },
        SnapshotEntry::Ready {
            source_path: PathBuf::from("src/first.rs"),
            prompt_path: "prompts/first.md".to_owned(),
            snapshot: "snapshot-first".to_owned(),
        },
        SnapshotEntry::Ready {
            source_path: PathBuf::from("src/first.rs"),
            prompt_path: "prompts/first.md".to_owned(),
            snapshot: "snapshot-first".to_owned(),
        },
    ];
    let spy = WriteSpy::default();

    let results = execute(&entries, &spy, true);

    assert_eq!(
        results,
        vec![
            SnapshotResult::Unreadable {
                source_path: PathBuf::from("src/unreadable.rs"),
                reason: SnapshotUnreadable::MissingPromptHeader,
            },
            SnapshotResult::DryRun {
                source_path: PathBuf::from("src/first.rs"),
                prompt_path: "prompts/first.md".to_owned(),
                snapshot: "snapshot-first".to_owned(),
            },
            SnapshotResult::DryRun {
                source_path: PathBuf::from("src/first.rs"),
                prompt_path: "prompts/first.md".to_owned(),
                snapshot: "snapshot-first".to_owned(),
            },
        ]
    );
    assert!(spy.calls.borrow().is_empty());
}

#[test]
fn real_execution_writes_each_ready_once_and_continues_after_exact_error() {
    let entries = vec![
        SnapshotEntry::Ready {
            source_path: PathBuf::from("src/first.rs"),
            prompt_path: "prompts/first.md".to_owned(),
            snapshot: "snapshot-first".to_owned(),
        },
        SnapshotEntry::Unreadable {
            source_path: PathBuf::from("src/unreadable.rs"),
            reason: SnapshotUnreadable::MissingParsedFile,
        },
        SnapshotEntry::Ready {
            source_path: PathBuf::from("src/failing.rs"),
            prompt_path: "prompts/failing.md".to_owned(),
            snapshot: "snapshot-failing".to_owned(),
        },
        SnapshotEntry::Ready {
            source_path: PathBuf::from("src/first.rs"),
            prompt_path: "prompts/first.md".to_owned(),
            snapshot: "snapshot-first".to_owned(),
        },
    ];
    let exact_error = "writer rejected bytes: code=E17".to_owned();
    let spy = WriteSpy::with_outcomes([Ok(()), Err(exact_error.clone()), Ok(())]);

    let results = execute(&entries, &spy, false);

    assert_eq!(
        results,
        vec![
            SnapshotResult::Written {
                source_path: PathBuf::from("src/first.rs"),
                prompt_path: "prompts/first.md".to_owned(),
            },
            SnapshotResult::Unreadable {
                source_path: PathBuf::from("src/unreadable.rs"),
                reason: SnapshotUnreadable::MissingParsedFile,
            },
            SnapshotResult::WriteFailed {
                source_path: PathBuf::from("src/failing.rs"),
                prompt_path: "prompts/failing.md".to_owned(),
                reason: exact_error,
            },
            SnapshotResult::Written {
                source_path: PathBuf::from("src/first.rs"),
                prompt_path: "prompts/first.md".to_owned(),
            },
        ]
    );
    assert_eq!(
        *spy.calls.borrow(),
        vec![
            ("prompts/first.md".to_owned(), "snapshot-first".to_owned()),
            (
                "prompts/failing.md".to_owned(),
                "snapshot-failing".to_owned(),
            ),
            ("prompts/first.md".to_owned(), "snapshot-first".to_owned()),
        ]
    );
    assert!(spy.outcomes.borrow().is_empty());
}
