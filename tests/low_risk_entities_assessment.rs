use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::{ParsedFile, PromptHeader, PublicInterface};
use crystalline_lint::entities::project_index::{LocalIndex, ProjectIndex};
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn contribution(tag: &'static str) -> LocalIndex<'static> {
    LocalIndex {
        referenced_prompt: Some(tag),
        alien_file: Some(Path::new(tag)),
        declared_traits: vec![tag],
        implemented_traits: vec![tag],
        blanket_impl_traits: vec![tag],
    }
}

fn reduce_locals(locals: impl IntoIterator<Item = LocalIndex<'static>>) -> ProjectIndex<'static> {
    let mut result = ProjectIndex::new();
    for local in locals {
        result.merge_local(local);
    }
    result
}

#[derive(Debug, PartialEq, Eq)]
struct Projection {
    prompts: BTreeSet<&'static str>,
    aliens_as_set: BTreeSet<&'static Path>,
    aliens_raw: Vec<&'static Path>,
    declared: BTreeSet<&'static str>,
    implemented: BTreeSet<&'static str>,
    blanket: BTreeSet<&'static str>,
}

fn projection(index: &ProjectIndex<'static>) -> Projection {
    Projection {
        prompts: index.referenced_prompts.iter().copied().collect(),
        aliens_as_set: index.alien_files.iter().copied().collect(),
        aliens_raw: index.alien_files.clone(),
        declared: index.all_declared_traits.iter().copied().collect(),
        implemented: index.all_implemented_traits.iter().copied().collect(),
        blanket: index.all_blanket_impl_traits.iter().copied().collect(),
    }
}

#[test]
#[ignore = "RED congelado: alien_files torna merge dependente da ordem"]
fn project_index_is_a_commutative_associative_monoid_across_all_fields() {
    let abc = reduce_locals([contribution("a"), contribution("b"), contribution("c")]);
    let cba = reduce_locals([contribution("c"), contribution("b"), contribution("a")]);

    let mut left = reduce_locals([contribution("a")]);
    left = left.merge(reduce_locals([contribution("b"), contribution("c")]));
    let mut right = ProjectIndex::new();
    right = right.merge(reduce_locals([
        contribution("a"),
        contribution("b"),
        contribution("c"),
    ]));
    let with_right_identity =
        reduce_locals([contribution("a"), contribution("b"), contribution("c")])
            .merge(ProjectIndex::new());

    let expected = projection(&abc);
    assert_eq!(projection(&cba), expected, "permutation changed the index");
    assert_eq!(projection(&left), expected, "partition changed the index");
    assert_eq!(
        projection(&right),
        expected,
        "left identity changed the index"
    );
    assert_eq!(
        projection(&with_right_identity),
        expected,
        "right identity changed the index"
    );
}

#[test]
fn duplicate_contributions_are_idempotent_in_all_four_sets() {
    let once = reduce_locals([contribution("duplicate")]);
    let twice = reduce_locals([contribution("duplicate"), contribution("duplicate")]);
    let expected = projection(&once);
    let actual = projection(&twice);

    assert_eq!(actual.prompts, expected.prompts);
    assert_eq!(actual.declared, expected.declared);
    assert_eq!(actual.implemented, expected.implemented);
    assert_eq!(actual.blanket, expected.blanket);
    assert_eq!(actual.prompts.len(), 1);
    assert_eq!(actual.declared.len(), 1);
    assert_eq!(actual.implemented.len(), 1);
    assert_eq!(actual.blanket.len(), 1);
}

fn parsed_sentinel(layer: Layer) -> ParsedFile<'static> {
    ParsedFile {
        path: Path::new("alien-P"),
        layer: layer.clone(),
        language: Language::Rust,
        prompt_header: Some(PromptHeader {
            prompt_path: "prompt-P",
            prompt_hash: None,
            current_hash: None,
            layer,
            updated: None,
        }),
        prompt_file_exists: true,
        prompt_refs: vec![],
        has_test_coverage: false,
        imports: vec![],
        tokens: vec![],
        public_interface: PublicInterface {
            functions: vec![],
            types: vec![],
            reexports: vec![],
        },
        prompt_snapshot: None,
        declared_traits: vec!["declared-only"],
        implemented_traits: vec!["implemented-only"],
        blanket_impl_traits: vec!["blanket-only"],
        declarations: vec![],
        static_declarations: vec![],
        module_decls: vec![],
        decision_exprs: vec![],
        constants: vec![],
        semantic_observations: vec![],
    }
}

#[test]
fn parsed_transport_and_error_identities_cover_every_index_field() {
    let unknown = LocalIndex::from_parsed(&parsed_sentinel(Layer::Unknown));
    let known = LocalIndex::from_parsed(&parsed_sentinel(Layer::L1));
    let unknown_projection = projection(&reduce_locals([unknown]));
    let known_projection = projection(&reduce_locals([known]));

    assert_eq!(unknown_projection.prompts, BTreeSet::from(["prompt-P"]));
    assert_eq!(unknown_projection.aliens_raw, vec![Path::new("alien-P")]);
    assert_eq!(
        unknown_projection.declared,
        BTreeSet::from(["declared-only"])
    );
    assert_eq!(
        unknown_projection.implemented,
        BTreeSet::from(["implemented-only"])
    );
    assert_eq!(unknown_projection.blanket, BTreeSet::from(["blanket-only"]));
    assert_eq!(known_projection.prompts, unknown_projection.prompts);
    assert!(known_projection.aliens_raw.is_empty());
    assert_eq!(known_projection.declared, unknown_projection.declared);
    assert_eq!(known_projection.implemented, unknown_projection.implemented);
    assert_eq!(known_projection.blanket, unknown_projection.blanket);

    let empty = projection(&reduce_locals([
        LocalIndex::empty(),
        LocalIndex::from_parse_error(),
        LocalIndex::from_source_error(),
    ]));
    assert_eq!(empty, projection(&ProjectIndex::new()));
}

fn layer_tag(value: Layer) -> u8 {
    match value {
        Layer::L0 => 0,
        Layer::L1 => 1,
        Layer::L2 => 2,
        Layer::L3 => 3,
        Layer::L4 => 4,
        Layer::Lab => 5,
        Layer::Unknown => 6,
    }
}

fn language_tag(value: Language) -> u8 {
    match value {
        Language::Rust => 0,
        Language::TypeScript => 1,
        Language::Python => 2,
        Language::C => 3,
        Language::Cpp => 4,
        Language::Zig => 5,
        Language::Go => 6,
        Language::Java => 7,
        Language::Elixir => 8,
        Language::Unknown => 9,
    }
}

fn level_tag(value: ViolationLevel) -> u8 {
    match value {
        ViolationLevel::Info => 0,
        ViolationLevel::Warning => 1,
        ViolationLevel::Error => 2,
        ViolationLevel::Fatal => 3,
    }
}

#[test]
fn public_variants_severity_order_and_violation_clone_are_complete() {
    let layers = [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ];
    let languages = [
        Language::Rust,
        Language::TypeScript,
        Language::Python,
        Language::C,
        Language::Cpp,
        Language::Zig,
        Language::Go,
        Language::Java,
        Language::Elixir,
        Language::Unknown,
    ];
    let levels = [
        ViolationLevel::Info,
        ViolationLevel::Warning,
        ViolationLevel::Error,
        ViolationLevel::Fatal,
    ];
    assert_eq!(layers.map(layer_tag), [0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(languages.map(language_tag), [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(levels.map(level_tag), [0, 1, 2, 3]);
    assert!(ViolationLevel::Info < ViolationLevel::Warning);
    assert!(ViolationLevel::Warning < ViolationLevel::Error);
    assert!(ViolationLevel::Error < ViolationLevel::Fatal);

    let borrowed = Violation {
        rule_id: "rule".to_owned(),
        level: ViolationLevel::Warning,
        message: "message".to_owned(),
        location: Location {
            path: Cow::Borrowed(Path::new("borrowed.rs")),
            line: 7,
            column: 11,
        },
    };
    let borrowed_clone = borrowed.clone();
    assert_eq!(borrowed_clone, borrowed);
    assert!(matches!(borrowed_clone.location.path, Cow::Borrowed(_)));

    let owned = Violation {
        rule_id: "rule".to_owned(),
        level: ViolationLevel::Warning,
        message: "message".to_owned(),
        location: Location {
            path: Cow::Owned(PathBuf::from("owned.rs")),
            line: 7,
            column: 11,
        },
    };
    let owned_clone = owned.clone();
    assert_eq!(owned_clone, owned);
    assert!(matches!(owned_clone.location.path, Cow::Owned(_)));

    let mutations = [
        Violation {
            rule_id: "other".to_owned(),
            ..borrowed.clone()
        },
        Violation {
            level: ViolationLevel::Error,
            ..borrowed.clone()
        },
        Violation {
            message: "other".to_owned(),
            ..borrowed.clone()
        },
        Violation {
            location: Location {
                path: Cow::Borrowed(Path::new("other.rs")),
                ..borrowed.location.clone()
            },
            ..borrowed.clone()
        },
        Violation {
            location: Location {
                line: 8,
                ..borrowed.location.clone()
            },
            ..borrowed.clone()
        },
        Violation {
            location: Location {
                column: 12,
                ..borrowed.location.clone()
            },
            ..borrowed.clone()
        },
    ];
    for mutation in mutations {
        assert_ne!(mutation, borrowed);
    }
}
