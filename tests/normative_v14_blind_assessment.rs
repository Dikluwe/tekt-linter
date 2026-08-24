use std::collections::{HashMap, HashSet};
use std::path::Path;

use crystalline_lint::entities::l1_allowed_external::L1AllowedExternal;
use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::{Import, ImportKind};
use crystalline_lint::entities::rule_traits::HasImports;
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::external_type_in_contract::check;

struct ImportFile<'a> {
    layer: Layer,
    path: &'a Path,
    imports: Vec<Import<'a>>,
}

impl<'a> HasImports<'a> for ImportFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }

    fn imports(&self) -> &[Import<'a>] {
        &self.imports
    }

    fn path(&self) -> &'a Path {
        self.path
    }
}

fn import(path: &'static str, line: usize, target_layer: Layer) -> Import<'static> {
    Import {
        path,
        line,
        kind: ImportKind::Direct,
        target_layer,
        target_subdir: None,
        is_test_origin: false,
    }
}

fn file(layer: Layer, imports: Vec<Import<'static>>) -> ImportFile<'static> {
    ImportFile {
        layer,
        path: Path::new("01_core/entities/order.rs"),
        imports,
    }
}

fn rust_allowed(packages: &[&str]) -> L1AllowedExternal {
    let allowed = packages
        .iter()
        .map(|package| ((*package).to_string(), HashSet::new()))
        .collect::<HashMap<_, _>>();
    L1AllowedExternal::for_rust(allowed)
}

fn rust_allowed_items(package: &str, items: &[&str]) -> L1AllowedExternal {
    let item_set = items.iter().map(|item| (*item).to_string()).collect();
    L1AllowedExternal::for_rust(HashMap::from([(package.to_string(), item_set)]))
}

#[test]
fn deny_by_default_and_violation_shape_are_normative() {
    let subject = file(
        Layer::L1,
        vec![import("comemo::Tracked", 17, Layer::Unknown)],
    );
    let violations = check(&subject, &rust_allowed(&[]), false);

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];
    assert_eq!(violation.rule_id, "V14");
    assert_eq!(violation.level, ViolationLevel::Error);
    assert_eq!(
        violation.message,
        "Dependência externa não autorizada em L1: 'comemo' não está em [l1_allowed_external]. Adicionar ao crystalline.toml se necessário, ou mover a dependência para L3."
    );
    assert_eq!(
        violation.location.path.as_ref(),
        Path::new("01_core/entities/order.rs")
    );
    assert_eq!(violation.location.line, 17);
    assert_eq!(violation.location.column, 0);
}

#[test]
fn whitelist_and_rust_stdlib_exemptions_allow_imports() {
    let subject = file(
        Layer::L1,
        vec![
            import("thiserror::Error", 1, Layer::Unknown),
            import("serde::Serialize", 2, Layer::Unknown),
            import("std::collections::HashMap", 3, Layer::Unknown),
            import("core::fmt::Display", 4, Layer::Unknown),
            import("alloc::string::String", 5, Layer::Unknown),
        ],
    );

    assert!(check(&subject, &rust_allowed(&["thiserror", "serde"]), false).is_empty());
}

#[test]
fn only_unknown_targets_are_external_and_only_l1_is_in_scope() {
    let internal = file(Layer::L1, vec![import("rayon::prelude", 3, Layer::L3)]);
    assert!(check(&internal, &rust_allowed(&[]), false).is_empty());

    for layer in [
        Layer::L0,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        let outside = file(layer, vec![import("rayon::prelude", 4, Layer::Unknown)]);
        assert!(check(&outside, &rust_allowed(&[]), false).is_empty());
    }
}

#[test]
fn package_is_the_first_rust_segment_and_preserves_scoped_npm_name() {
    let subject = file(
        Layer::L1,
        vec![
            import("serde::de::DeserializeOwned", 1, Layer::Unknown),
            import("@scope/pkg", 2, Layer::Unknown),
        ],
    );

    assert!(check(&subject, &rust_allowed(&["serde", "@scope/pkg"]), false).is_empty());
}

#[test]
fn type_level_whitelist_allows_only_named_items() {
    let allowed = rust_allowed_items("ecow", &["EcoString"]);
    let subject = file(
        Layer::L1,
        vec![
            import("ecow::EcoString", 1, Layer::Unknown),
            import("ecow::EcoMap", 2, Layer::Unknown),
        ],
    );
    let violations = check(&subject, &allowed, false);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].location.line, 2);
    assert!(violations[0].message.contains("'ecow'"));
}

#[test]
fn intra_crate_qualifiers_are_exempt() {
    let subject = file(
        Layer::L1,
        vec![
            import("crate::entities::Value", 1, Layer::Unknown),
            import("super::policy::Rule", 2, Layer::Unknown),
        ],
    );
    assert!(check(&subject, &rust_allowed(&[]), false).is_empty());
}

#[test]
fn preserves_import_order_and_one_violation_per_disallowed_import() {
    let subject = file(
        Layer::L1,
        vec![
            import("tokio::sync::Mutex", 29, Layer::Unknown),
            import("comemo::Tracked", 7, Layer::Unknown),
            import("tokio::runtime::Runtime", 31, Layer::Unknown),
        ],
    );
    let violations = check(&subject, &rust_allowed(&["serde"]), false);

    assert_eq!(violations.len(), 3);
    assert_eq!(
        violations
            .iter()
            .map(|v| v.location.line)
            .collect::<Vec<_>>(),
        [29, 7, 31]
    );
    assert!(violations[0].message.contains("'tokio'"));
    assert!(violations[1].message.contains("'comemo'"));
    assert!(violations[2].message.contains("'tokio'"));
}

#[test]
fn test_origin_is_skipped_by_default_and_checked_when_enabled() {
    let mut test_import = import("dev_dependency::Helper", 41, Layer::Unknown);
    test_import.is_test_origin = true;
    let subject = file(Layer::L1, vec![test_import]);

    assert!(check(&subject, &rust_allowed(&[]), false).is_empty());
    let enabled = check(&subject, &rust_allowed(&[]), true);
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].location.line, 41);
}

#[test]
fn empty_import_collection_is_allowed() {
    assert!(check(&file(Layer::L1, vec![]), &rust_allowed(&[]), false).is_empty());
}
