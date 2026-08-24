use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::{Import, ImportKind};
use crystalline_lint::entities::project_index::ProjectIndex;
use crystalline_lint::entities::rule_traits::{HasCoverage, HasImports};
use crystalline_lint::entities::violation::{Violation, ViolationLevel};
use crystalline_lint::rules::{alien_file, dangling_contract, quarantine_leak, test_file};
use std::path::Path;

struct CoverageFixture<'a> {
    layer: Layer,
    covered: bool,
    path: &'a Path,
}

impl<'a> HasCoverage<'a> for CoverageFixture<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn has_test_coverage(&self) -> bool {
        self.covered
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

#[test]
fn v2_truth_table_is_exact_and_preserves_public_location() {
    let path = Path::new("01_core/área/Δ.rs");
    for layer in [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        for covered in [false, true] {
            let violations = test_file::check(&CoverageFixture {
                layer: layer.clone(),
                covered,
                path,
            });
            let expected = usize::from(layer == Layer::L1 && !covered);
            assert_eq!(
                violations.len(),
                expected,
                "V2 mismatch for {layer:?}, covered={covered}"
            );
            if let Some(v) = violations.first() {
                assert_eq!(v.rule_id, "V2");
                assert_eq!(v.level, ViolationLevel::Error);
                assert_eq!(v.location.path.as_ref(), path);
                assert_eq!((v.location.line, v.location.column), (1, 0));
            }
        }
    }
}

#[test]
fn v8_is_one_fatal_per_alien_in_received_identity_order_and_empty_is_identity() {
    let paths = [
        Path::new("fora/β.rs"),
        Path::new("fora/./β.rs"),
        Path::new("z/終.rs"),
    ];
    let mut empty = ProjectIndex::new();
    assert!(alien_file::check_aliens(&empty).is_empty());
    empty.alien_files = paths.to_vec();
    let violations = alien_file::check_aliens(&empty);
    assert_eq!(violations.len(), paths.len());
    for (violation, expected) in violations.iter().zip(paths) {
        assert_eq!(violation.rule_id, "V8");
        assert_eq!(violation.level, ViolationLevel::Fatal);
        assert_eq!(violation.location.path.as_ref(), expected);
        assert_eq!((violation.location.line, violation.location.column), (0, 0));
        assert!(violation
            .message
            .contains(&expected.to_string_lossy().into_owned()));
    }
}

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

fn import<'a>(path: &'a str, line: usize, target_layer: Layer) -> Import<'a> {
    Import {
        path,
        line,
        kind: ImportKind::Direct,
        target_layer,
        target_subdir: None,
        is_test_origin: false,
    }
}

#[test]
fn v10_truth_table_depends_only_on_production_origin_and_lab_target() {
    for origin in [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        for target in [
            Layer::L0,
            Layer::L1,
            Layer::L2,
            Layer::L3,
            Layer::L4,
            Layer::Lab,
            Layer::Unknown,
        ] {
            let fixture = ImportFixture {
                layer: origin.clone(),
                imports: vec![import("crate::target", 7, target.clone())],
                path: Path::new("origin.rs"),
            };
            let violations = quarantine_leak::check(&fixture);
            let production = matches!(origin, Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4);
            assert_eq!(
                violations.len(),
                usize::from(production && target == Layer::Lab),
                "V10 mismatch for {origin:?} -> {target:?}"
            );
            if let Some(v) = violations.first() {
                assert_eq!(v.rule_id, "V10");
                assert_eq!(v.level, ViolationLevel::Fatal);
            }
        }
    }
}

#[test]
fn v10_preserves_multiset_evidence_under_input_permutation() {
    let path = Path::new("02_shell/produção.rs");
    let imports = vec![
        import("lab::α", 3, Layer::Lab),
        import("lab::α", 3, Layer::Lab),
        import("lab::β", 11, Layer::Lab),
        import("crate::safe", 99, Layer::L1),
    ];
    let mut reversed = imports.clone();
    reversed.reverse();
    let diagnose = |imports| {
        quarantine_leak::check(&ImportFixture {
            layer: Layer::L2,
            imports,
            path,
        })
        .into_iter()
        .map(|v| {
            assert_eq!(v.location.path.as_ref(), path);
            assert_eq!(v.level, ViolationLevel::Fatal);
            (v.location.line, v.message)
        })
        .collect::<Vec<_>>()
    };
    let mut forward_diagnostics = diagnose(imports);
    let mut reverse_diagnostics = diagnose(reversed);
    assert_eq!(forward_diagnostics.len(), 3);
    forward_diagnostics.sort();
    reverse_diagnostics.sort();
    assert_eq!(forward_diagnostics, reverse_diagnostics);
    assert_eq!(
        forward_diagnostics
            .iter()
            .filter(|(line, _)| *line == 3)
            .count(),
        2
    );
    assert!(forward_diagnostics
        .iter()
        .any(|(_, message)| message.contains("lab::α")));
    assert!(forward_diagnostics
        .iter()
        .any(|(_, message)| message.contains("lab::β")));
}

fn dangling_for(
    declared: &[&'static str],
    concrete: &[&'static str],
    blanket: &[&'static str],
    level: ViolationLevel,
) -> Vec<Violation<'static>> {
    let mut index = ProjectIndex::new();
    for name in declared {
        index.all_declared_traits.insert(name);
        index.all_declared_traits.insert(name);
    }
    index
        .all_implemented_traits
        .extend(concrete.iter().copied());
    index
        .all_blanket_impl_traits
        .extend(blanket.iter().copied());
    dangling_contract::check_dangling_contracts(&index, level)
}

#[test]
fn v11_is_exact_set_difference_with_injected_level_and_canonical_order() {
    let pending = [
        "Trait",
        "TraitA",
        "TraitAA",
        "trait",
        "Árvore",
        "A\u{301}rvore",
        "Alpha",
    ];
    let satisfied = ["Concrete", "Blanket", "SatisfiedBoth"];
    let mut all_declared = pending.to_vec();
    all_declared.extend(satisfied);
    let concrete = ["Concrete", "SatisfiedBoth", "UnrelatedConcrete"];
    let blanket = ["Blanket", "SatisfiedBoth", "UnrelatedBlanket"];

    let reference = dangling_for(&all_declared, &concrete, &blanket, ViolationLevel::Info);
    for rotation in 0..all_declared.len() {
        let mut declared = all_declared.clone();
        declared.rotate_left(rotation);
        if rotation % 2 == 1 {
            declared.reverse();
        }
        let mut concrete_order = concrete;
        let concrete_shift = rotation % concrete_order.len();
        concrete_order.rotate_left(concrete_shift);
        let mut blanket_order = blanket;
        let blanket_shift = rotation % blanket_order.len();
        blanket_order.rotate_right(blanket_shift);
        let candidate = dangling_for(
            &declared,
            &concrete_order,
            &blanket_order,
            ViolationLevel::Info,
        );
        assert_eq!(
            candidate, reference,
            "full V11 vector changed for construction {rotation}"
        );
    }

    assert_eq!(reference.len(), pending.len());
    assert!(reference
        .iter()
        .all(|v| v.rule_id == "V11" && v.level == ViolationLevel::Info));
    assert!(reference.iter().all(|v| {
        v.location.path.as_ref() == Path::new("01_core/contracts")
            && (v.location.line, v.location.column) == (0, 0)
    }));

    let mut canonical_names = pending.to_vec();
    canonical_names.sort();
    for (violation, name) in reference.iter().zip(canonical_names) {
        assert!(
            violation.message.contains(name),
            "V11 order/message mismatch for {name:?}: {}",
            violation.message
        );
    }
}

#[test]
fn textual_representations_remain_distinct_and_complete_across_classifiers() {
    let mut index = ProjectIndex::new();
    index.alien_files = vec![Path::new("x/á.rs"), Path::new("x/a.rs")];
    assert_eq!(alien_file::check_aliens(&index).len(), 2);

    index
        .all_declared_traits
        .extend(["Trait", "trait", "Tráit"]);
    let dangling = dangling_contract::check_dangling_contracts(&index, ViolationLevel::Warning);
    assert_eq!(dangling.len(), 3);
    for spelling in ["Trait", "trait", "Tráit"] {
        assert!(dangling.iter().any(|v| v.message.contains(spelling)));
    }
}
