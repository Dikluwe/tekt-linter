use crystalline_lint::contracts::prompt_provider::{AllPrompts, PromptEntry};
use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::{
    FunctionSignature, PromptHeader, PublicInterface, TypeKind, TypeSignature,
};
use crystalline_lint::entities::project_index::ProjectIndex;
use crystalline_lint::entities::rule_traits::{HasHashes, HasPublicInterface};
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::{orphan_prompt, prompt_drift, prompt_stale};
use std::path::Path;

struct HashFixture<'a> {
    header: Option<PromptHeader<'a>>,
    path: &'a Path,
}

impl<'a> HasHashes<'a> for HashFixture<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.header.as_ref()
    }

    fn path(&self) -> &'a Path {
        self.path
    }
}

fn header<'a>(declared: Option<&'a str>, current: Option<&str>) -> PromptHeader<'a> {
    PromptHeader {
        prompt_path: "00_nucleo/prompts/source.md",
        prompt_hash: declared,
        current_hash: current.map(str::to_string),
        layer: Layer::L1,
        updated: None,
    }
}

#[test]
fn v5_emits_exactly_once_iff_both_hashes_exist_and_differ() {
    let path = Path::new("01_core/source.rs");
    let cases = [
        ("no-header", None, 0),
        ("no-declared", Some(header(None, Some("aaaaaaaa"))), 0),
        ("no-current", Some(header(Some("aaaaaaaa"), None)), 0),
        ("equal", Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))), 0),
        (
            "different",
            Some(header(Some("aaaaaaaa"), Some("bbbbbbbb"))),
            1,
        ),
    ];
    for (name, header, expected) in cases {
        let violations = prompt_drift::check(&HashFixture { header, path });
        assert_eq!(violations.len(), expected, "V5 truth-table case {name}");
        if expected == 1 {
            assert_eq!(violations[0].rule_id, "V5");
        }
    }
}

#[test]
fn v5_preserves_path_values_and_representation_without_normalizing() {
    let path = Path::new("odd/../identity.rs");
    let fixture = HashFixture {
        header: Some(header(Some("ABCDEF12"), Some("abcdef12"))),
        path,
    };
    let violations = prompt_drift::check(&fixture);
    assert_eq!(violations.len(), 1, "case differences were normalized");
    let violation = &violations[0];
    assert_eq!(violation.location.path.as_ref(), path);
    assert!(violation.message.contains("ABCDEF12"));
    assert!(violation.message.contains("abcdef12"));

    let whitespace = HashFixture {
        header: Some(header(Some("abcdef12 "), Some("abcdef12"))),
        path,
    };
    assert_eq!(prompt_drift::check(&whitespace).len(), 1);
}

struct InterfaceFixture<'a> {
    header: Option<PromptHeader<'a>>,
    current: PublicInterface<'a>,
    snapshot: Option<PublicInterface<'a>>,
    path: &'a Path,
}

impl<'a> HasPublicInterface<'a> for InterfaceFixture<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.header.as_ref()
    }

    fn public_interface(&self) -> &PublicInterface<'a> {
        &self.current
    }

    fn prompt_snapshot(&self) -> Option<&PublicInterface<'a>> {
        self.snapshot.as_ref()
    }

    fn path(&self) -> &'a Path {
        self.path
    }
}

fn function<'a>(name: &'a str, param: &'a str, ret: Option<&'a str>) -> FunctionSignature<'a> {
    FunctionSignature {
        name,
        params: vec![param],
        return_type: ret,
    }
}

fn interface<'a>(reverse: bool) -> PublicInterface<'a> {
    let mut functions = vec![
        function("alpha", "u8", Some("bool")),
        function("beta", "u16", None),
    ];
    let mut types = vec![
        TypeSignature {
            name: "Choice",
            kind: TypeKind::Enum,
            members: vec!["A", "B"],
        },
        TypeSignature {
            name: "Value",
            kind: TypeKind::Struct,
            members: vec!["field: u8"],
        },
    ];
    let mut reexports = vec!["crate::a", "crate::b"];
    if reverse {
        functions.reverse();
        types.reverse();
        reexports.reverse();
    }
    PublicInterface {
        functions,
        types,
        reexports,
    }
}

#[test]
fn v6_is_invariant_to_permutations_of_the_same_interface() {
    let fixture = InterfaceFixture {
        header: Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))),
        current: interface(false),
        snapshot: Some(interface(true)),
        path: Path::new("01_core/api.rs"),
    };
    assert!(prompt_stale::check(&fixture).is_empty());
    assert!(
        prompt_stale::compute_delta(&fixture.current, fixture.snapshot.as_ref().unwrap())
            .is_empty()
    );
}

#[test]
fn v6_detects_every_field_change_and_preserves_observable_multiplicity() {
    let baseline = PublicInterface {
        functions: vec![function("f", "u8", Some("bool"))],
        types: vec![TypeSignature {
            name: "T",
            kind: TypeKind::Struct,
            members: vec!["x: u8"],
        }],
        reexports: vec!["crate::one"],
    };
    let mutations = [
        PublicInterface {
            functions: vec![function("g", "u8", Some("bool"))],
            ..baseline.clone()
        },
        PublicInterface {
            functions: vec![function("f", "u16", Some("bool"))],
            ..baseline.clone()
        },
        PublicInterface {
            functions: vec![function("f", "u8", None)],
            ..baseline.clone()
        },
        PublicInterface {
            types: vec![TypeSignature {
                name: "U",
                kind: TypeKind::Struct,
                members: vec!["x: u8"],
            }],
            ..baseline.clone()
        },
        PublicInterface {
            types: vec![TypeSignature {
                name: "T",
                kind: TypeKind::Enum,
                members: vec!["x: u8"],
            }],
            ..baseline.clone()
        },
        PublicInterface {
            types: vec![TypeSignature {
                name: "T",
                kind: TypeKind::Struct,
                members: vec!["y: u8"],
            }],
            ..baseline.clone()
        },
        PublicInterface {
            reexports: vec!["crate::two"],
            ..baseline.clone()
        },
    ];
    for (index, current) in mutations.into_iter().enumerate() {
        let fixture = InterfaceFixture {
            header: Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))),
            current,
            snapshot: Some(baseline.clone()),
            path: Path::new("01_core/api.rs"),
        };
        let violations = prompt_stale::check(&fixture);
        assert_eq!(violations.len(), 1, "field mutation {index} was lost");
        assert_eq!(violations[0].rule_id, "V6");
    }

    let duplicate_current = PublicInterface {
        functions: vec![
            function("f", "u8", Some("bool")),
            function("f", "u8", Some("bool")),
        ],
        ..PublicInterface::empty()
    };
    let single_snapshot = PublicInterface {
        functions: vec![function("f", "u8", Some("bool"))],
        ..PublicInterface::empty()
    };
    let delta = prompt_stale::compute_delta(&duplicate_current, &single_snapshot);
    assert!(
        !delta.is_empty(),
        "V6 lost observable duplicate multiplicity"
    );
}

#[test]
fn v6_delta_description_is_complete_and_deterministic_under_permutation() {
    let snapshot = PublicInterface::empty();
    let forward = PublicInterface {
        functions: vec![function("zeta", "u8", None), function("alpha", "u8", None)],
        types: vec![
            TypeSignature {
                name: "Zulu",
                kind: TypeKind::Struct,
                members: vec![],
            },
            TypeSignature {
                name: "Alpha",
                kind: TypeKind::Enum,
                members: vec![],
            },
        ],
        reexports: vec!["crate::z", "crate::a"],
    };
    let mut reverse = forward.clone();
    reverse.functions.reverse();
    reverse.types.reverse();
    reverse.reexports.reverse();
    let make_fixture = |current| InterfaceFixture {
        header: Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))),
        current,
        snapshot: Some(snapshot.clone()),
        path: Path::new("01_core/api.rs"),
    };
    let first = prompt_stale::check(&make_fixture(forward));
    let second = prompt_stale::check(&make_fixture(reverse));
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(
        first[0].message, second[0].message,
        "V6 delta description depends on extraction order"
    );
    for symbol in ["alpha", "zeta", "Alpha", "Zulu", "crate::a", "crate::z"] {
        assert!(first[0].message.contains(symbol), "delta omitted {symbol}");
    }
}

fn all_prompts<'a>(paths: &'a [&'a str], reverse: bool) -> AllPrompts<'a> {
    let iter: Box<dyn Iterator<Item = &&str>> = if reverse {
        Box::new(paths.iter().rev())
    } else {
        Box::new(paths.iter())
    };
    AllPrompts {
        entries: iter
            .map(|path| PromptEntry {
                relative_path: path,
            })
            .collect(),
    }
}

#[test]
fn v7_is_exact_deterministic_and_preserves_injected_level_and_path_representation() {
    let paths = [
        "00_nucleo/prompts/a.md",
        "00_nucleo/prompts/./a.md",
        "00_nucleo/prompts/b.md",
    ];
    let mut index = ProjectIndex::new();
    index.referenced_prompts.insert(paths[2]);
    let first =
        orphan_prompt::check_orphans(&index, &all_prompts(&paths, false), ViolationLevel::Info);
    let second =
        orphan_prompt::check_orphans(&index, &all_prompts(&paths, true), ViolationLevel::Info);
    assert_eq!(
        first, second,
        "V7 output depends on inventory construction order"
    );
    assert_eq!(first.len(), 2);
    assert!(first
        .iter()
        .all(|v| v.rule_id == "V7" && v.level == ViolationLevel::Info));
    let emitted: Vec<_> = first
        .iter()
        .map(|v| v.location.path.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        emitted,
        vec![paths[1].to_string(), paths[0].to_string()],
        "V7 normalized identities or emitted non-canonical order"
    );
}
