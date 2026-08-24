use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::rule_traits::{
    Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
};
use crystalline_lint::infra::crate_registry::{parse_manifest, CrateRegistry, MemberCrate};
use crystalline_lint::rules::provenance_inventory::check_inventory;
use crystalline_lint::rules::unsourced_constant::V21RuleConfig;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn member(name: &str, dir: &str, layer: Layer) -> MemberCrate {
    MemberCrate {
        name: name.to_owned(),
        dir: PathBuf::from(dir),
        layer,
        deps: HashSet::new(),
        renames: HashMap::new(),
    }
}

fn permutations<T: Clone>(values: &[T]) -> Vec<Vec<T>> {
    fn visit<T: Clone>(prefix: &mut Vec<T>, rest: &mut Vec<T>, out: &mut Vec<Vec<T>>) {
        if rest.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..rest.len() {
            let value = rest.remove(index);
            prefix.push(value.clone());
            visit(prefix, rest, out);
            prefix.pop();
            rest.insert(index, value);
        }
    }
    let mut out = Vec::new();
    visit(&mut Vec::new(), &mut values.to_vec(), &mut out);
    out
}

#[test]
fn registry_lookups_are_invariant_under_permutation_and_duplicates() {
    let members = [
        member("same", "a", Layer::L1),
        member("same", "b", Layer::L3),
        member("unique", "c", Layer::L4),
    ];
    for permutation in permutations(&members) {
        assert!(CrateRegistry::from_members(permutation).is_err());
    }

    let identical = member("same", "a", Layer::L1);
    let registry = CrateRegistry::from_members(vec![identical.clone(), identical]).unwrap();
    assert_eq!(registry.member_layer("same"), Some(Layer::L1));
}

#[test]
fn owner_is_deepest_and_ties_are_permutation_invariant() {
    let shallow = member("shallow", "ws/crates", Layer::L1);
    let deep = member("deep", "ws/crates/a", Layer::L3);
    for order in [
        vec![shallow.clone(), deep.clone()],
        vec![deep.clone(), shallow],
    ] {
        let registry = CrateRegistry::from_members(order).unwrap();
        let owner = registry
            .owner_of(Path::new("ws/crates/a/src/lib.rs"))
            .unwrap();
        assert_eq!(
            (&owner.name, &owner.dir, &owner.layer),
            (
                &"deep".to_owned(),
                &PathBuf::from("ws/crates/a"),
                &Layer::L3
            )
        );
        assert!(registry.owner_of(Path::new("outside/file.rs")).is_none());
    }

    let tied = [
        member("tie-a", "ws/tied", Layer::L1),
        member("tie-b", "ws/tied", Layer::L4),
    ];
    for order in permutations(&tied) {
        assert!(CrateRegistry::from_members(order).is_err());
    }
}

#[test]
fn normalized_package_dependency_and_rename_collisions_are_order_invariant() {
    let first = r#"
[package]
name = "foo-bar"
version = "0.1.0"
[dependencies]
foo-bar = "1"
foo_bar = "1"
dep-x = { package = "real-a", version = "1" }
dep_x = { package = "real-b", version = "1" }
"#;
    let second = r#"
[package]
name = "foo-bar"
version = "0.1.0"
[dependencies]
dep_x = { package = "real-b", version = "1" }
dep-x = { package = "real-a", version = "1" }
foo_bar = "1"
foo-bar = "1"
"#;
    assert!(parse_manifest(first).is_err());
    assert!(parse_manifest(second).is_err());
}

#[test]
fn each_normalized_collision_class_is_rejected() {
    assert!(CrateRegistry::from_members(vec![
        member("foo-bar", "a", Layer::L1),
        member("foo_bar", "b", Layer::L1),
    ])
    .is_err());

    let dependency_collision = r#"
[package]
name = "owner"
version = "0.1.0"
[dependencies]
foo-bar = "1"
foo_bar = "2"
"#;
    assert!(parse_manifest(dependency_collision).is_err());

    let rename_collision = r#"
[package]
name = "owner"
version = "0.1.0"
[dependencies]
dep-x = { package = "real-a", version = "1" }
dep_x = { package = "real-b", version = "1" }
"#;
    assert!(parse_manifest(rename_collision).is_err());
}

#[test]
fn from_members_rejects_internal_dependency_and_rename_collisions() {
    let mut dependency_collision = member("owner", "owner", Layer::L1);
    dependency_collision.deps = HashSet::from(["foo-bar".to_owned(), "foo_bar".to_owned()]);
    let dependency_rejected = CrateRegistry::from_members(vec![dependency_collision]).is_err();

    let mut rename_collision = member("owner", "owner", Layer::L1);
    rename_collision.renames = HashMap::from([
        ("dep-x".to_owned(), "real-a".to_owned()),
        ("dep_x".to_owned(), "real-b".to_owned()),
    ]);
    let rename_rejected = CrateRegistry::from_members(vec![rename_collision]).is_err();
    assert!(
        dependency_rejected && rename_rejected,
        "from_members collision rejection: dependencies={dependency_rejected}, renames={rename_rejected}"
    );
}

#[test]
fn present_but_invalid_workspace_manifest_is_an_infrastructure_error() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("00_nucleo/prompts")).unwrap();
    std::fs::write(root.path().join("Cargo.toml"), b"[workspace\nmembers = [\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .arg(root.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    assert!(stderr.contains("infrastructure"));
    assert!(stderr.contains("manifest"));
}

#[derive(Clone)]
struct File {
    path: &'static Path,
    language: Language,
    constants: Vec<SourceConstant<'static>>,
}

impl HasConstants<'static> for File {
    fn layer(&self) -> &Layer {
        &Layer::L1
    }
    fn constants(&self) -> &[SourceConstant<'static>] {
        &self.constants
    }
    fn path(&self) -> &'static Path {
        self.path
    }
    fn language(&self) -> &Language {
        &self.language
    }
}

fn constant(snippet: &'static str, cited: bool) -> SourceConstant<'static> {
    SourceConstant {
        kind: ConstantKind::FunctionNumberLiteral,
        snippet,
        line: 10,
        column: 2,
        citation: cited.then_some(Citation {
            kind: CitationKind::Rationale("fixture"),
            raw: "// rationale: fixture",
            line: 9,
        }),
        is_test_origin: false,
        function_return_type: None,
        is_in_binary_scaling: false,
        context_var: None,
        geometric_sink: None,
        is_in_data_table: false,
    }
}

#[test]
fn inventory_is_permutation_invariant_and_applies_all_filters() {
    let eligible_z = File {
        path: Path::new("01_core/alpha/z.rs"),
        language: Language::Rust,
        constants: vec![constant("0.6", true)],
    };
    let eligible_a = File {
        path: Path::new("01_core/alpha/a.rs"),
        language: Language::Rust,
        constants: vec![constant("0.7", false)],
    };
    let mut test = File {
        path: Path::new("01_core/alpha/test.rs"),
        language: Language::Rust,
        constants: vec![constant("9.1", false)],
    };
    test.constants[0].is_test_origin = true;
    let mut table = File {
        path: Path::new("01_core/alpha/table.rs"),
        language: Language::Rust,
        constants: vec![constant("9.2", false)],
    };
    table.constants[0].is_in_data_table = true;
    let trivial = File {
        path: Path::new("01_core/alpha/trivial.rs"),
        language: Language::Rust,
        constants: vec![constant("1", false)],
    };
    let non_rust = File {
        path: Path::new("01_core/alpha/foreign.ts"),
        language: Language::TypeScript,
        constants: vec![constant("9.3", false)],
    };
    let format = File {
        path: Path::new("export/pdf/format.rs"),
        language: Language::Rust,
        constants: vec![constant("9.4", false)],
    };
    let files = vec![
        eligible_z, eligible_a, test, table, trivial, non_rust, format,
    ];
    let mut config = V21RuleConfig::default();
    config.format_syntax_modules.push("export/pdf".to_owned());
    config.trivial_literals.insert("1".to_owned());
    let forward = check_inventory(&files, &config);
    let reverse = check_inventory(&files.iter().cloned().rev().collect::<Vec<_>>(), &config);
    assert_eq!(
        forward, reverse,
        "file permutation changed inventory diagnostics"
    );
    assert_eq!(forward.len(), 1);
    assert!(forward[0].message.contains("1/2"));
    assert_eq!(forward[0].location.path, Path::new("01_core/alpha/a.rs"));
}
