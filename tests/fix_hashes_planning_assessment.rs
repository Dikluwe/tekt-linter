use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::shell::fix_hashes::{plan, FixEntry, FixUnavailable, HashRewriter};

#[derive(Debug)]
struct HeaderAnswer {
    source_path: PathBuf,
    answer: Option<(String, String)>,
}

#[derive(Debug)]
struct HashAnswer {
    path: PathBuf,
    answer: Option<String>,
}

#[derive(Debug, Default)]
struct Spy {
    headers: RefCell<VecDeque<HeaderAnswer>>,
    prompt_hashes: RefCell<VecDeque<HashAnswer>>,
    source_hashes: RefCell<VecDeque<HashAnswer>>,
    calls: RefCell<Vec<String>>,
}

impl Spy {
    fn with_header(self, source_path: impl Into<PathBuf>, answer: Option<(&str, &str)>) -> Self {
        self.headers.borrow_mut().push_back(HeaderAnswer {
            source_path: source_path.into(),
            answer: answer.map(|(prompt, old)| (prompt.to_owned(), old.to_owned())),
        });
        self
    }

    fn with_prompt_hash(self, prompt_path: impl Into<PathBuf>, answer: Option<&str>) -> Self {
        self.prompt_hashes.borrow_mut().push_back(HashAnswer {
            path: prompt_path.into(),
            answer: answer.map(str::to_owned),
        });
        self
    }

    fn with_source_hash(self, source_path: impl Into<PathBuf>, answer: Option<&str>) -> Self {
        self.source_hashes.borrow_mut().push_back(HashAnswer {
            path: source_path.into(),
            answer: answer.map(str::to_owned),
        });
        self
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl HashRewriter for Spy {
    fn read_header(&self, source_path: &Path) -> Option<(String, String)> {
        self.calls
            .borrow_mut()
            .push(format!("header:{}", source_path.display()));
        let answer = self
            .headers
            .borrow_mut()
            .pop_front()
            .expect("unexpected read_header");
        assert_eq!(answer.source_path, source_path);
        answer.answer
    }

    fn compute_hash(&self, prompt_path: &str) -> Option<String> {
        self.calls
            .borrow_mut()
            .push(format!("prompt:{prompt_path}"));
        let answer = self
            .prompt_hashes
            .borrow_mut()
            .pop_front()
            .expect("unexpected compute_hash");
        assert_eq!(answer.path, Path::new(prompt_path));
        answer.answer
    }

    fn compute_source_hash(&self, source_path: &Path) -> Option<String> {
        self.calls
            .borrow_mut()
            .push(format!("source:{}", source_path.display()));
        let answer = self
            .source_hashes
            .borrow_mut()
            .pop_front()
            .expect("unexpected compute_source_hash");
        assert_eq!(answer.path, source_path);
        answer.answer
    }

    fn write_hash(&self, _: &Path, _: &str) -> Result<(), String> {
        panic!("planning must not write source files")
    }

    fn write_prompt_meta(&self, _: &str, _: &str) -> Result<(), String> {
        panic!("planning must not write prompt metadata")
    }
}

fn violation<'a>(rule_id: &str, path: &'a Path) -> Violation<'a> {
    Violation {
        rule_id: rule_id.to_owned(),
        level: ViolationLevel::Warning,
        message: "assessment fixture".to_owned(),
        location: Location {
            path: Cow::Borrowed(path),
            line: 1,
            column: 1,
        },
    }
}

#[test]
fn filters_exact_v5_and_preserves_order_duplicates_and_hostile_paths() {
    let hostile = "../odd/./name\nwith-newline.rs";
    let spy = Spy::default()
        .with_header(hostile, Some(("prompts/a.md", "old-a")))
        .with_prompt_hash("prompts/a.md", Some("new-a"))
        .with_source_hash(hostile, Some("source-a"))
        .with_header(hostile, None);
    let violations = [
        violation("V05", Path::new("ignored-prefix.rs")),
        violation("V5", Path::new(hostile)),
        violation("v5", Path::new("ignored-case.rs")),
        violation("V5", Path::new(hostile)),
        violation("V5 ", Path::new("ignored-suffix.rs")),
    ];

    let entries = plan(&violations, &spy);

    assert_eq!(
        entries,
        vec![
            FixEntry::Ready {
                source_path: PathBuf::from(hostile),
                prompt_path: "prompts/a.md".into(),
                old_hash: "old-a".into(),
                new_hash: "new-a".into(),
                source_hash: "source-a".into(),
            },
            FixEntry::Unavailable {
                source_path: PathBuf::from(hostile),
                reason: FixUnavailable::HeaderUnreadable,
            },
        ]
    );
    assert_eq!(
        spy.calls(),
        vec![
            format!("header:{hostile}"),
            "prompt:prompts/a.md".into(),
            format!("source:{hostile}"),
            format!("header:{hostile}"),
        ]
    );
}

#[test]
fn unreadable_header_calls_no_hash_calculation() {
    let spy = Spy::default().with_header("unreadable.rs", None);

    let entries = plan(&[violation("V5", Path::new("unreadable.rs"))], &spy);

    assert_eq!(
        entries,
        vec![FixEntry::Unavailable {
            source_path: "unreadable.rs".into(),
            reason: FixUnavailable::HeaderUnreadable,
        }]
    );
    assert_eq!(spy.calls(), vec!["header:unreadable.rs"]);
}

#[test]
fn readable_header_computes_a_and_b_once_and_materializes_all_four_combinations() {
    struct Case {
        source: &'static str,
        prompt: &'static str,
        old: &'static str,
        hash_a: Option<&'static str>,
        hash_b: Option<&'static str>,
        expected: FixEntry,
    }

    let cases = vec![
        Case {
            source: "ready.rs",
            prompt: "ready.md",
            old: "old-r",
            hash_a: Some("new-r"),
            hash_b: Some("source-r"),
            expected: FixEntry::Ready {
                source_path: "ready.rs".into(),
                prompt_path: "ready.md".into(),
                old_hash: "old-r".into(),
                new_hash: "new-r".into(),
                source_hash: "source-r".into(),
            },
        },
        Case {
            source: "no-a.rs",
            prompt: "no-a.md",
            old: "old-a",
            hash_a: None,
            hash_b: Some("source-a"),
            expected: FixEntry::Unavailable {
                source_path: "no-a.rs".into(),
                reason: FixUnavailable::PromptHashUnavailable {
                    prompt_path: "no-a.md".into(),
                    old_hash: "old-a".into(),
                    source_hash: "source-a".into(),
                },
            },
        },
        Case {
            source: "no-b.rs",
            prompt: "no-b.md",
            old: "old-b",
            hash_a: Some("new-b"),
            hash_b: None,
            expected: FixEntry::Unavailable {
                source_path: "no-b.rs".into(),
                reason: FixUnavailable::SourceHashUnavailable {
                    prompt_path: "no-b.md".into(),
                    old_hash: "old-b".into(),
                    new_hash: "new-b".into(),
                },
            },
        },
        Case {
            source: "neither.rs",
            prompt: "neither.md",
            old: "old-n",
            hash_a: None,
            hash_b: None,
            expected: FixEntry::Unavailable {
                source_path: "neither.rs".into(),
                reason: FixUnavailable::BothHashesUnavailable {
                    prompt_path: "neither.md".into(),
                    old_hash: "old-n".into(),
                },
            },
        },
    ];

    for case in cases {
        let spy = Spy::default()
            .with_header(case.source, Some((case.prompt, case.old)))
            .with_prompt_hash(case.prompt, case.hash_a)
            .with_source_hash(case.source, case.hash_b);

        assert_eq!(
            plan(&[violation("V5", Path::new(case.source))], &spy),
            vec![case.expected]
        );
        assert_eq!(
            spy.calls(),
            vec![
                format!("header:{}", case.source),
                format!("prompt:{}", case.prompt),
                format!("source:{}", case.source),
            ],
            "each readable header must cause exactly one Hash A and one Hash B calculation"
        );
    }
}
