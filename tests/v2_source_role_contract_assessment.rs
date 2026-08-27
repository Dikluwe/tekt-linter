//! Oráculo independente P0118 — tabela pura de papel estrutural e V2.

use crystalline_lint::contracts::file_provider::SourceRole;
use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::rule_traits::HasCoverage;
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::test_file;
use std::path::Path;

struct Fixture {
    layer: Layer,
    role: SourceRole,
    covered: bool,
}

impl HasCoverage<'static> for Fixture {
    fn layer(&self) -> &Layer {
        &self.layer
    }

    fn source_role(&self) -> SourceRole {
        self.role
    }

    fn has_test_coverage(&self) -> bool {
        self.covered
    }

    fn path(&self) -> &'static Path {
        Path::new("01_core/catalogo.py")
    }
}

fn findings(layer: Layer, role: SourceRole, covered: bool) -> Vec<String> {
    test_file::check(&Fixture {
        layer,
        role,
        covered,
    })
    .into_iter()
    .map(|finding| finding.rule_id)
    .collect()
}

#[test]
fn production_l1_requires_coverage_exactly_once() {
    let fixture = Fixture {
        layer: Layer::L1,
        role: SourceRole::Production,
        covered: false,
    };
    let findings = test_file::check(&fixture);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "V2");
    assert_eq!(findings[0].level, ViolationLevel::Error);
    assert_eq!(findings[0].location.path.as_ref(), fixture.path());
    assert_eq!(
        (findings[0].location.line, findings[0].location.column),
        (1, 0)
    );
}

#[test]
fn production_l1_with_coverage_is_green() {
    assert!(findings(Layer::L1, SourceRole::Production, true).is_empty());
}

#[test]
fn test_role_is_green_independently_of_coverage_boolean() {
    for covered in [false, true] {
        assert!(
            findings(Layer::L1, SourceRole::Test, covered).is_empty(),
            "Test must not recursively require another test; covered={covered}"
        );
    }
}

#[test]
fn v2_remains_l1_only() {
    for layer in [
        Layer::L0,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        assert!(
            findings(layer.clone(), SourceRole::Production, false).is_empty(),
            "V2 unexpectedly applied to {layer:?}"
        );
    }
}

#[test]
fn unknown_role_is_not_reinterpreted_as_production_or_coverage() {
    assert!(findings(Layer::L1, SourceRole::Unknown, false).is_empty());
    assert!(findings(Layer::L1, SourceRole::Unknown, true).is_empty());
}
