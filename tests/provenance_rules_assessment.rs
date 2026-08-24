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
    let mut current = interface(false);
    current.functions.push(current.functions[0].clone());
    current.types.push(current.types[0].clone());
    current.reexports.push(current.reexports[0]);
    let mut snapshot = current.clone();
    snapshot.functions.reverse();
    snapshot.types.reverse();
    snapshot.reexports.reverse();
    let fixture = InterfaceFixture {
        header: Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))),
        current,
        snapshot: Some(snapshot),
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

    let one_function = function("f", "u8", Some("bool"));
    let one_type = TypeSignature {
        name: "T",
        kind: TypeKind::Struct,
        members: vec!["x: u8"],
    };
    for current_has_extra in [true, false] {
        let (current_count, snapshot_count) = if current_has_extra { (2, 1) } else { (1, 2) };
        for family in ["function", "type", "reexport"] {
            let mut current = PublicInterface::empty();
            let mut snapshot = PublicInterface::empty();
            match family {
                "function" => {
                    current.functions = vec![one_function.clone(); current_count];
                    snapshot.functions = vec![one_function.clone(); snapshot_count];
                }
                "type" => {
                    current.types = vec![one_type.clone(); current_count];
                    snapshot.types = vec![one_type.clone(); snapshot_count];
                }
                "reexport" => {
                    current.reexports = vec!["crate::same"; current_count];
                    snapshot.reexports = vec!["crate::same"; snapshot_count];
                }
                _ => unreachable!(),
            }
            let delta = prompt_stale::compute_delta(&current, &snapshot);
            let added =
                delta.added_functions.len() + delta.added_types.len() + delta.added_reexports.len();
            let removed = delta.removed_functions.len()
                + delta.removed_types.len()
                + delta.removed_reexports.len();
            assert_eq!(
                (added, removed),
                if current_has_extra { (1, 0) } else { (0, 1) },
                "wrong multiset delta for {family}, current_has_extra={current_has_extra}"
            );
        }
    }
}

#[test]
fn v6_delta_description_is_complete_and_deterministic_under_permutation() {
    let snapshot = PublicInterface {
        functions: vec![
            FunctionSignature {
                name: "same",
                params: vec!["z", "a"],
                return_type: Some("z"),
            },
            FunctionSignature {
                name: "same",
                params: vec!["a", "z"],
                return_type: None,
            },
            FunctionSignature {
                name: "same",
                params: vec!["a", "z"],
                return_type: None,
            },
        ],
        types: vec![
            TypeSignature {
                name: "Same",
                kind: TypeKind::Trait,
                members: vec!["z", "a"],
            },
            TypeSignature {
                name: "Same",
                kind: TypeKind::Enum,
                members: vec!["a", "z"],
            },
        ],
        reexports: vec!["crate::same", "crate::same", "crate::old"],
    };
    let forward = PublicInterface {
        functions: vec![
            FunctionSignature {
                name: "same",
                params: vec!["a", "z"],
                return_type: Some("a"),
            },
            FunctionSignature {
                name: "same",
                params: vec!["a", "z"],
                return_type: Some("z"),
            },
            FunctionSignature {
                name: "same",
                params: vec!["a", "z"],
                return_type: Some("a"),
            },
        ],
        types: vec![
            TypeSignature {
                name: "Same",
                kind: TypeKind::Interface,
                members: vec!["a", "z"],
            },
            TypeSignature {
                name: "Same",
                kind: TypeKind::Struct,
                members: vec!["z", "a"],
            },
            TypeSignature {
                name: "Same",
                kind: TypeKind::Interface,
                members: vec!["a", "z"],
            },
        ],
        reexports: vec!["crate::z", "crate::a", "crate::a"],
    };
    let mut reverse = forward.clone();
    reverse.functions.reverse();
    reverse.types.reverse();
    reverse.reexports.reverse();
    let mut reverse_snapshot = snapshot.clone();
    reverse_snapshot.functions.reverse();
    reverse_snapshot.types.reverse();
    reverse_snapshot.reexports.reverse();
    let make_fixture = |current, snapshot| InterfaceFixture {
        header: Some(header(Some("aaaaaaaa"), Some("aaaaaaaa"))),
        current,
        snapshot: Some(snapshot),
        path: Path::new("01_core/api.rs"),
    };
    let first_fixture = make_fixture(forward, snapshot);
    let second_fixture = make_fixture(reverse, reverse_snapshot);
    let first_delta = prompt_stale::compute_delta(
        &first_fixture.current,
        first_fixture.snapshot.as_ref().unwrap(),
    );
    let second_delta = prompt_stale::compute_delta(
        &second_fixture.current,
        second_fixture.snapshot.as_ref().unwrap(),
    );
    assert_eq!(
        first_delta, second_delta,
        "canonical delta vectors require all-field tie-breaking"
    );
    let first = prompt_stale::check(&first_fixture);
    let second = prompt_stale::check(&second_fixture);
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(
        first[0].message, second[0].message,
        "V6 delta description depends on extraction order"
    );
    for symbol in ["same", "Same", "crate::a", "crate::z", "crate::old"] {
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
