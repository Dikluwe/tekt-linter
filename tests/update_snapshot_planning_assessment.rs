use std::borrow::Cow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::{
    FunctionSignature, ParsedFile, PromptHeader, PublicInterface,
};
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::shell::update_snapshot::{
    plan, SnapshotEntry, SnapshotRewriter, SnapshotUnreadable,
};

#[derive(Default)]
struct SerializationSpy {
    calls: RefCell<Vec<(Vec<String>, Vec<String>)>>,
}

impl SnapshotRewriter for SerializationSpy {
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String {
        let functions = interface
            .functions
            .iter()
            .map(|function| function.name.to_owned())
            .collect::<Vec<_>>();
        let reexports = interface
            .reexports
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<Vec<_>>();
        self.calls
            .borrow_mut()
            .push((functions.clone(), reexports.clone()));
        format!("spy:{functions:?}:{reexports:?}")
    }

    fn write_snapshot(&self, _: &str, _: &str) -> Result<(), String> {
        panic!("planning must not write")
    }
}

fn interface<'a>(function: &'a str, reexport: &'a str) -> PublicInterface<'a> {
    PublicInterface {
        functions: vec![FunctionSignature {
            name: function,
            params: vec!["input: &str"],
            return_type: Some("bool"),
        }],
        types: vec![],
        reexports: vec![reexport],
    }
}

fn parsed<'a>(
    path: &'a Path,
    prompt_path: Option<&'a str>,
    public_interface: PublicInterface<'a>,
) -> ParsedFile<'a> {
    ParsedFile {
        path,
        layer: Layer::L2,
        language: Language::Rust,
        prompt_header: prompt_path.map(|prompt_path| PromptHeader {
            prompt_path,
            prompt_hash: None,
            current_hash: None,
            layer: Layer::L2,
            updated: None,
        }),
        prompt_file_exists: true,
        prompt_refs: vec![],
        has_test_coverage: true,
        imports: vec![],
        tokens: vec![],
        public_interface,
        prompt_snapshot: None,
        declarations: vec![],
        declared_traits: vec![],
        implemented_traits: vec![],
        blanket_impl_traits: vec![],
        static_declarations: vec![],
        module_decls: vec![],
        decision_exprs: vec![],
        constants: vec![],
        semantic_observations: vec![],
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
fn filters_exact_v6_and_preserves_occurrence_order_and_duplicates() {
    let alpha = Path::new("02_shell/alpha.rs");
    let beta = Path::new("02_shell/beta.rs");
    let parsed_files = vec![
        parsed(alpha, Some("prompts/alpha.md"), interface("alpha", "Alpha")),
        parsed(beta, Some("prompts/beta.md"), interface("beta", "Beta")),
    ];
    let violations = vec![
        violation("V5", beta),
        violation("V6", beta),
        violation("v6", alpha),
        violation("V60", alpha),
        violation("V6", alpha),
        violation("V6", beta),
    ];
    let spy = SerializationSpy::default();

    let entries = plan(&violations, &parsed_files, &spy);

    assert_eq!(
        entries,
        vec![
            SnapshotEntry::Ready {
                source_path: PathBuf::from("02_shell/beta.rs"),
                prompt_path: "prompts/beta.md".to_owned(),
                snapshot: "spy:[\"beta\"]:[\"Beta\"]".to_owned(),
            },
            SnapshotEntry::Ready {
                source_path: PathBuf::from("02_shell/alpha.rs"),
                prompt_path: "prompts/alpha.md".to_owned(),
                snapshot: "spy:[\"alpha\"]:[\"Alpha\"]".to_owned(),
            },
            SnapshotEntry::Ready {
                source_path: PathBuf::from("02_shell/beta.rs"),
                prompt_path: "prompts/beta.md".to_owned(),
                snapshot: "spy:[\"beta\"]:[\"Beta\"]".to_owned(),
            },
        ]
    );
    assert_eq!(spy.calls.borrow().len(), 3);
}

#[test]
fn path_matching_is_integral_hostile_and_uses_first_duplicate_parsed_file() {
    let requested = Path::new("odd/../target.rs");
    let normalized_lookalike = Path::new("target.rs");
    let basename_lookalike = Path::new("elsewhere/target.rs");
    let parsed_files = vec![
        parsed(
            normalized_lookalike,
            Some("wrong/normalized.md"),
            interface("wrong_normalized", "WrongNormalized"),
        ),
        parsed(
            basename_lookalike,
            Some("wrong/basename.md"),
            interface("wrong_basename", "WrongBasename"),
        ),
        parsed(
            requested,
            Some("prompts/first exact ! #.md"),
            interface("first_exact", "FirstExact"),
        ),
        parsed(
            requested,
            Some("prompts/second.md"),
            interface("second_exact", "SecondExact"),
        ),
    ];
    let spy = SerializationSpy::default();

    let entries = plan(&[violation("V6", requested)], &parsed_files, &spy);

    assert_eq!(
        entries,
        vec![SnapshotEntry::Ready {
            source_path: requested.to_path_buf(),
            prompt_path: "prompts/first exact ! #.md".to_owned(),
            snapshot: "spy:[\"first_exact\"]:[\"FirstExact\"]".to_owned(),
        }]
    );
    assert_eq!(
        &*spy.calls.borrow(),
        &[(
            vec!["first_exact".to_owned()],
            vec!["FirstExact".to_owned()]
        )]
    );
}

#[test]
fn missing_parsed_and_missing_header_are_distinct_and_never_serialize() {
    let absent = Path::new("missing/source.rs");
    let headerless = Path::new("02_shell/headerless.rs");
    let parsed_files = vec![parsed(headerless, None, interface("unused", "Unused"))];
    let violations = vec![violation("V6", absent), violation("V6", headerless)];
    let spy = SerializationSpy::default();

    let entries = plan(&violations, &parsed_files, &spy);

    assert_eq!(
        entries,
        vec![
            SnapshotEntry::Unreadable {
                source_path: absent.to_path_buf(),
                reason: SnapshotUnreadable::MissingParsedFile,
            },
            SnapshotEntry::Unreadable {
                source_path: headerless.to_path_buf(),
                reason: SnapshotUnreadable::MissingPromptHeader,
            },
        ]
    );
    assert!(spy.calls.borrow().is_empty());
}
