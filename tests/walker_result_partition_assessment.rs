use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use crystalline_lint::contracts::file_provider::{collect_walker_results, SourceError, SourceFile};
use crystalline_lint::entities::layer::{Language, Layer};

#[derive(Default)]
struct Observations {
    next_calls: Cell<usize>,
    size_hint_calls: Cell<usize>,
    eof_seen: Cell<bool>,
}

struct HostileIterator {
    items: std::vec::IntoIter<Result<SourceFile, SourceError>>,
    observations: Rc<Observations>,
}

impl HostileIterator {
    fn new(items: Vec<Result<SourceFile, SourceError>>) -> (Self, Rc<Observations>) {
        let observations = Rc::new(Observations::default());
        (
            Self {
                items: items.into_iter(),
                observations: Rc::clone(&observations),
            },
            observations,
        )
    }
}

impl Iterator for HostileIterator {
    type Item = Result<SourceFile, SourceError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.observations.eof_seen.get() {
            panic!("next called after the first EOF");
        }

        self.observations
            .next_calls
            .set(self.observations.next_calls.get() + 1);
        let item = self.items.next();
        if item.is_none() {
            self.observations.eof_seen.set(true);
        }
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.observations
            .size_hint_calls
            .set(self.observations.size_hint_calls.get() + 1);
        panic!("size_hint must not be consulted")
    }
}

fn source(
    path: &str,
    content: &str,
    language: Language,
    layer: Layer,
    has_adjacent_test: bool,
) -> SourceFile {
    SourceFile {
        path: PathBuf::from(path),
        content: content.to_owned(),
        language,
        layer,
        has_adjacent_test,
    }
}

fn unreadable(path: &str, reason: &str) -> SourceError {
    SourceError::Unreadable {
        path: PathBuf::from(path),
        reason: reason.to_owned(),
    }
}

fn collect_checked(
    items: Vec<Result<SourceFile, SourceError>>,
) -> (Vec<SourceFile>, Vec<SourceError>) {
    let item_count = items.len();
    let (iterator, observations) = HostileIterator::new(items);
    let output = collect_walker_results(iterator);

    assert_eq!(observations.next_calls.get(), item_count + 1);
    assert_eq!(observations.size_hint_calls.get(), 0);
    assert!(observations.eof_seen.get());
    output
}

fn assert_source(
    actual: &SourceFile,
    path: &str,
    content: &str,
    language: Language,
    layer: Layer,
    has_adjacent_test: bool,
) {
    assert_eq!(actual.path, PathBuf::from(path));
    assert_eq!(actual.content, content);
    assert_eq!(actual.language, language);
    assert_eq!(actual.layer, layer);
    assert_eq!(actual.has_adjacent_test, has_adjacent_test);
}

#[test]
fn partitions_empty_only_ok_and_only_err_without_extra_iterator_observations() {
    let (files, errors) = collect_checked(vec![]);
    assert!(files.is_empty());
    assert!(errors.is_empty());

    let (files, errors) = collect_checked(vec![
        Ok(source("a.rs", "a", Language::Rust, Layer::L1, false)),
        Ok(source("b.ts", "b", Language::TypeScript, Layer::L2, true)),
    ]);
    assert_eq!(files.len(), 2);
    assert!(errors.is_empty());
    assert_source(&files[0], "a.rs", "a", Language::Rust, Layer::L1, false);
    assert_source(
        &files[1],
        "b.ts",
        "b",
        Language::TypeScript,
        Layer::L2,
        true,
    );

    let expected = vec![unreadable("x", "first"), unreadable("y", "second")];
    let (files, errors) = collect_checked(vec![
        Err(unreadable("x", "first")),
        Err(unreadable("y", "second")),
    ]);
    assert!(files.is_empty());
    assert_eq!(errors, expected);
}

#[test]
fn preserves_alternation_duplicates_hostile_unicode_and_items_after_errors() {
    let hostile_path = "diretório/../\u{1f4a5}\0arquivo.rs";
    let hostile_content = "\u{feff}linha\0\r\n🦀 é\n";
    let hostile_reason = "permissão negada: \"λ\"\0\nsem normalização";

    let input = vec![
        Ok(source(
            hostile_path,
            hostile_content,
            Language::Rust,
            Layer::Unknown,
            true,
        )),
        Err(unreadable("erro/α", hostile_reason)),
        Ok(source("dup.rs", "igual", Language::Rust, Layer::L1, false)),
        Err(unreadable("erro/igual", "mesma razão")),
        Ok(source("dup.rs", "igual", Language::Rust, Layer::L1, false)),
        Err(unreadable("erro/igual", "mesma razão")),
        Ok(source(
            "posterior.py",
            "print('depois do erro')",
            Language::Python,
            Layer::L3,
            false,
        )),
    ];

    let (files, errors) = collect_checked(input);

    assert_eq!(files.len(), 4);
    assert_source(
        &files[0],
        hostile_path,
        hostile_content,
        Language::Rust,
        Layer::Unknown,
        true,
    );
    assert_source(
        &files[1],
        "dup.rs",
        "igual",
        Language::Rust,
        Layer::L1,
        false,
    );
    assert_source(
        &files[2],
        "dup.rs",
        "igual",
        Language::Rust,
        Layer::L1,
        false,
    );
    assert_source(
        &files[3],
        "posterior.py",
        "print('depois do erro')",
        Language::Python,
        Layer::L3,
        false,
    );
    assert_eq!(
        errors,
        vec![
            unreadable("erro/α", hostile_reason),
            unreadable("erro/igual", "mesma razão"),
            unreadable("erro/igual", "mesma razão"),
        ]
    );
}
