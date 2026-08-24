use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::{Import, ImportKind};
use crystalline_lint::entities::rule_traits::{HasImports, HasPubLeak};
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::{forbidden_import, pub_leak};
use std::collections::HashSet;
use std::path::Path;

struct ImportFixture<'a> {
    layer: Layer,
    imports: Vec<Import<'a>>,
    path: &'a Path,
}

impl<'a> HasImports<'a> for ImportFixture<'a> {
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

impl<'a> HasPubLeak<'a> for ImportFixture<'a> {
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

fn import<'a>(
    path: &'a str,
    line: usize,
    kind: ImportKind,
    target_layer: Layer,
    target_subdir: Option<&'a str>,
    is_test_origin: bool,
) -> Import<'a> {
    Import {
        path,
        line,
        kind,
        target_layer,
        target_subdir,
        is_test_origin,
    }
}

fn v3_forbidden(origin: &Layer, target: &Layer) -> bool {
    matches!(
        (origin, target),
        (Layer::L1, Layer::L2 | Layer::L3 | Layer::L4 | Layer::Lab)
            | (Layer::L2, Layer::L3 | Layer::L4 | Layer::Lab)
            | (Layer::L3, Layer::L2 | Layer::L4 | Layer::Lab)
            | (Layer::L4, Layer::Lab)
    )
}

fn layers() -> [Layer; 7] {
    [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ]
}

#[test]
fn v3_matches_the_complete_seven_by_seven_matrix() {
    for origin in layers() {
        for target in layers() {
            let fixture = ImportFixture {
                layer: origin.clone(),
                imports: vec![import(
                    "target",
                    4,
                    ImportKind::Direct,
                    target.clone(),
                    None,
                    false,
                )],
                path: Path::new("origin.rs"),
            };
            let violations = forbidden_import::check(&fixture, false);
            assert_eq!(
                violations.len(),
                usize::from(v3_forbidden(&origin, &target)),
                "V3 mismatch for {origin:?} -> {target:?}"
            );
        }
    }
}

#[test]
fn v3_test_guard_only_filters_test_origin_and_all_import_kinds_are_equivalent() {
    for kind in [
        ImportKind::Direct,
        ImportKind::Glob,
        ImportKind::Alias,
        ImportKind::Named,
    ] {
        for is_test_origin in [false, true] {
            let fixture = ImportFixture {
                layer: Layer::L1,
                imports: vec![import(
                    "crate::forbidden",
                    8,
                    kind.clone(),
                    Layer::L4,
                    None,
                    is_test_origin,
                )],
                path: Path::new("01_core/value.rs"),
            };
            assert_eq!(
                forbidden_import::check(&fixture, false).len(),
                usize::from(!is_test_origin)
            );
            assert_eq!(forbidden_import::check(&fixture, true).len(), 1);
        }
    }
}

#[test]
fn v3_preserves_order_multiplicity_path_line_layers_and_unicode_import_evidence() {
    let source = Path::new("01_core/área/源.rs");
    let imports = vec![
        import("crate::β", 9, ImportKind::Direct, Layer::L2, None, false),
        import("crate::β", 9, ImportKind::Alias, Layer::L2, None, false),
        import("crate::B", 2, ImportKind::Glob, Layer::L3, None, false),
    ];
    let violations = forbidden_import::check(
        &ImportFixture {
            layer: Layer::L1,
            imports,
            path: source,
        },
        false,
    );
    assert_eq!(violations.len(), 3);
    assert_eq!(
        violations
            .iter()
            .map(|v| v.location.line)
            .collect::<Vec<_>>(),
        vec![9, 9, 2]
    );
    for violation in &violations {
        assert_eq!(violation.rule_id, "V3");
        assert_eq!(violation.level, ViolationLevel::Error);
        assert_eq!(violation.location.path.as_ref(), source);
        assert!(violation.message.contains("L1"));
    }
    assert!(violations[0].message.contains("crate::β"));
    assert!(violations[1].message.contains("crate::β"));
    assert!(violations[2].message.contains("crate::B"));
}

fn ports(values: &[&str]) -> pub_leak::L1Ports {
    pub_leak::L1Ports::new(
        values
            .iter()
            .map(|value| value.to_string())
            .collect::<HashSet<_>>(),
    )
}

#[test]
fn v9_matches_origin_destination_subdir_and_ports_product() {
    let configured = ports(&["contracts"]);
    for origin in layers() {
        for target in layers() {
            for subdir in [None, Some("contracts"), Some("internal")] {
                let fixture = ImportFixture {
                    layer: origin.clone(),
                    imports: vec![import(
                        "crate::x",
                        3,
                        ImportKind::Direct,
                        target.clone(),
                        subdir,
                        false,
                    )],
                    path: Path::new("source.rs"),
                };
                let expected = matches!(origin, Layer::L2 | Layer::L3)
                    && target == Layer::L1
                    && subdir == Some("internal");
                assert_eq!(
                    pub_leak::check(&fixture, &configured, false).len(),
                    usize::from(expected),
                    "V9 mismatch: {origin:?}->{target:?}, subdir={subdir:?}"
                );
            }
        }
    }
}

#[test]
fn v9_port_membership_is_exact_for_case_unicode_normalization_and_prefixes() {
    let configured = ports(&["Port", "Á", "A\u{301}", "core"]);
    let cases = [
        ("Port", false),
        ("port", true),
        ("PORT", true),
        ("Á", false),
        ("A\u{301}", false),
        ("á", true),
        ("core", false),
        ("corex", true),
        ("core/internal", true),
        ("", true),
    ];
    for (subdir, should_violate) in cases {
        let fixture = ImportFixture {
            layer: Layer::L2,
            imports: vec![import(
                "crate::value",
                5,
                ImportKind::Direct,
                Layer::L1,
                Some(subdir),
                false,
            )],
            path: Path::new("02_shell/value.rs"),
        };
        assert_eq!(
            pub_leak::check(&fixture, &configured, false).len(),
            usize::from(should_violate),
            "V9 normalized or prefix-matched {subdir:?}"
        );
    }
}

#[test]
fn v9_guard_kinds_multiplicity_order_and_evidence_match_v3_guarantees() {
    let source = Path::new("03_infra/á/源.rs");
    let imports = vec![
        import(
            "crate::same",
            7,
            ImportKind::Direct,
            Layer::L1,
            Some("内部"),
            false,
        ),
        import(
            "crate::same",
            7,
            ImportKind::Glob,
            Layer::L1,
            Some("Á"),
            false,
        ),
        import(
            "crate::same",
            7,
            ImportKind::Alias,
            Layer::L1,
            Some("A\u{301}"),
            false,
        ),
        import(
            "crate::same",
            7,
            ImportKind::Named,
            Layer::L1,
            Some(""),
            false,
        ),
        import(
            "crate::test",
            2,
            ImportKind::Direct,
            Layer::L1,
            Some("internal"),
            true,
        ),
        import(
            "crate::none",
            13,
            ImportKind::Direct,
            Layer::L1,
            None,
            false,
        ),
        import(
            "crate::safe",
            11,
            ImportKind::Named,
            Layer::L1,
            Some("public"),
            false,
        ),
    ];
    let configured = ports(&["public"]);
    let fixture = ImportFixture {
        layer: Layer::L3,
        imports: imports.clone(),
        path: source,
    };
    let production = pub_leak::check(&fixture, &configured, false);
    assert_eq!(production.len(), 4);
    assert_eq!(
        production
            .iter()
            .map(|v| v.location.line)
            .collect::<Vec<_>>(),
        vec![7, 7, 7, 7]
    );
    let all = pub_leak::check(&fixture, &configured, true);
    assert_eq!(all.len(), 5);
    assert_eq!(
        all.iter().map(|v| v.location.line).collect::<Vec<_>>(),
        vec![7, 7, 7, 7, 2]
    );
    for violation in &all {
        assert_eq!(violation.rule_id, "V9");
        assert_eq!(violation.level, ViolationLevel::Error);
        assert_eq!(violation.location.path.as_ref(), source);
    }
    let expected_evidence = [
        ("crate::same", "内部"),
        ("crate::same", "Á"),
        ("crate::same", "A\u{301}"),
        ("crate::same", ""),
        ("crate::test", "internal"),
    ];
    let missing_evidence: Vec<_> = all
        .iter()
        .zip(expected_evidence)
        .filter_map(|(violation, (import_path, subdir))| {
            (!(violation.message.contains(import_path) && violation.message.contains(subdir)))
                .then(|| (import_path, subdir, violation.message.clone()))
        })
        .collect();
    let same_identity_messages: HashSet<_> = all[..4]
        .iter()
        .map(|violation| violation.message.as_str())
        .collect();
    assert!(
        missing_evidence.is_empty() && same_identity_messages.len() == 4,
        "V9 evidence failures: missing={missing_evidence:?}, distinct_messages={}/4",
        same_identity_messages.len()
    );

    let mut reversed = imports;
    reversed.reverse();
    let reverse = pub_leak::check(
        &ImportFixture {
            layer: Layer::L3,
            imports: reversed,
            path: source,
        },
        &configured,
        true,
    );
    let mut forward_multiset: Vec<_> = all
        .into_iter()
        .map(|v| (v.location.line, v.message))
        .collect();
    let mut reverse_multiset: Vec<_> = reverse
        .into_iter()
        .map(|v| (v.location.line, v.message))
        .collect();
    forward_multiset.sort();
    reverse_multiset.sort();
    assert_eq!(forward_multiset, reverse_multiset);
}
