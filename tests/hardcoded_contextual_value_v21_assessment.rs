//! Gate cego B1 — Assessment 0017, identidade `verifier/v21-l1/0017`.
//! Derivado somente dos sete insumos L0 hash-pinned; não usa filesystem real.

use crystalline_lint::contracts::citation_freshness::{
    CitationFreshness, CitationFreshnessResolver, CitationStaleReason, CitationUnknownReason,
};
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::rule_traits::{
    Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
};
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
use std::{
    cell::RefCell,
    collections::HashSet,
    path::{Path, PathBuf},
};

struct MockFile<'a> {
    layer: Layer,
    language: Language,
    path: &'a Path,
    constants: Vec<SourceConstant<'a>>,
}
impl<'a> HasConstants<'a> for MockFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn constants(&self) -> &[SourceConstant<'a>] {
        &self.constants
    }
    fn path(&self) -> &'a Path {
        self.path
    }
    fn language(&self) -> &Language {
        &self.language
    }
}
struct MockFreshness {
    answer: CitationFreshness,
    calls: RefCell<Vec<(String, usize)>>,
}
impl MockFreshness {
    fn new(answer: CitationFreshness) -> Self {
        Self {
            answer,
            calls: RefCell::new(vec![]),
        }
    }
}
impl CitationFreshnessResolver for MockFreshness {
    fn resolve(&self, path: &str, line: usize) -> CitationFreshness {
        self.calls.borrow_mut().push((path.to_owned(), line));
        self.answer.clone()
    }
}
fn constant(snippet: &'static str, line: usize) -> SourceConstant<'static> {
    SourceConstant {
        kind: ConstantKind::FunctionNumberLiteral,
        snippet,
        line,
        column: line + 3,
        citation: None,
        is_test_origin: false,
        function_return_type: Some("Length"),
        is_in_binary_scaling: true,
        context_var: Some("style.size".to_owned()),
        geometric_sink: Some("layouter.regions.current.cursor_y".to_owned()),
        is_in_data_table: false,
    }
}
fn file(path: &'static str, constants: Vec<SourceConstant<'static>>) -> MockFile<'static> {
    MockFile {
        layer: Layer::L1,
        language: Language::Rust,
        path: Path::new(path),
        constants,
    }
}
fn config() -> V21RuleConfig {
    V21RuleConfig::default()
}
fn never_called() -> MockFreshness {
    MockFreshness::new(CitationFreshness::Unknown(CitationUnknownReason::Io))
}
fn citation(kind: CitationKind<'static>) -> Citation<'static> {
    Citation {
        kind,
        raw: "raw citation",
        line: 7,
    }
}
fn strict_config() -> V21RuleConfig {
    let mut c = config();
    c.strict_modules = vec!["strict".to_owned()];
    c
}

#[test]
fn language_matrix_and_empty_collection_are_silent() {
    for language in [Language::TypeScript, Language::Python, Language::Unknown] {
        let mut subject = file("01_core/layout.rs", vec![constant("0.6", 11)]);
        subject.language = language;
        let r = never_called();
        assert!(check(&subject, &config(), &r).is_empty());
        assert!(r.calls.borrow().is_empty());
    }
    let r = never_called();
    assert!(check(&file("01_core/layout.rs", vec![]), &config(), &r).is_empty());
    assert!(r.calls.borrow().is_empty());
}

#[test]
fn predicate_requires_product_of_three_axes() {
    let mut a = constant("0.61", 1);
    a.is_in_binary_scaling = false;
    let mut b = constant("0.62", 2);
    b.context_var = None;
    let mut c = constant("0.63", 3);
    c.geometric_sink = None;
    let r = never_called();
    assert!(check(&file("01_core/layout.rs", vec![a, b, c]), &config(), &r).is_empty());
    assert!(r.calls.borrow().is_empty());
    assert_eq!(
        check(
            &file("01_core/layout.rs", vec![constant("0.64", 4)]),
            &config(),
            &r
        )
        .len(),
        1
    );
}

#[test]
fn filters_are_exact_and_lookalikes_are_not_exempt() {
    let r = never_called();
    for p in ["01_core/export/pdf/mod.rs", "01_core/export/svg.rs"] {
        assert!(check(&file(p, vec![constant("0.6", 1)]), &config(), &r).is_empty());
    }
    for p in [
        "01_core/export/pdfish/mod.rs",
        "01_core/export/svg_extra.rs",
        "01_core/export/PDF/mod.rs",
    ] {
        assert_eq!(
            check(&file(p, vec![constant("0.6", 1)]), &config(), &r).len(),
            1,
            "{p}"
        );
    }
    let mut a = constant("0.6", 2);
    a.is_test_origin = true;
    let mut b = constant("0.7", 3);
    b.is_in_data_table = true;
    assert!(check(&file("01_core/layout.rs", vec![a, b]), &config(), &r).is_empty());
}

#[test]
fn configured_identifiers_match_segments_not_substrings() {
    let mut cfg = config();
    cfg.context_vars = vec!["size".into()];
    cfg.geometric_sinks = vec!["gap".into()];
    cfg.format_syntax_modules = vec![];
    cfg.scope_modules = vec![];
    cfg.scope_types = vec![];
    cfg.trivial_literals = HashSet::new();
    let mut exact = constant("0.61", 1);
    exact.context_var = Some("style.SIZE".into());
    exact.geometric_sink = Some("frame.gap".into());
    let mut bad_c = constant("0.62", 2);
    bad_c.context_var = Some("downsized".into());
    bad_c.geometric_sink = Some("gap".into());
    let mut bad_s = constant("0.63", 3);
    bad_s.context_var = Some("size".into());
    bad_s.geometric_sink = Some("gapless".into());
    let got = check(
        &file("01_core/layout.rs", vec![exact, bad_c, bad_s]),
        &cfg,
        &never_called(),
    );
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].location.line, 1);
}

#[test]
fn trivials_are_lexical_and_near_controls_visible() {
    let r = never_called();
    for x in [
        "0", "1", "-1", "2", "100", "0.0", "1.0", "\"\"", "\"a\"", "\"\\n\"",
    ] {
        assert!(
            check(
                &file("01_core/layout.rs", vec![constant(x, 1)]),
                &config(),
                &r
            )
            .is_empty(),
            "{x}"
        );
    }
    for x in ["0.00", "1.00", "-1.0", "2.0", "100.0", "\"ab\""] {
        assert_eq!(
            check(
                &file("01_core/layout.rs", vec![constant(x, 1)]),
                &config(),
                &r
            )
            .len(),
            1,
            "{x}"
        );
    }
}

#[test]
fn spec_and_rationale_require_nonempty_payload() {
    for k in [
        CitationKind::Spec("CSS §1"),
        CitationKind::Rationale("by design"),
    ] {
        let mut x = constant("0.6", 8);
        x.citation = Some(citation(k));
        let r = never_called();
        assert!(check(&file("01_core/layout.rs", vec![x]), &config(), &r).is_empty());
        assert!(r.calls.borrow().is_empty());
    }
    for k in [CitationKind::Spec("   "), CitationKind::Rationale("\t")] {
        let mut x = constant("0.6", 8);
        x.citation = Some(citation(k));
        assert_eq!(
            check(
                &file("01_core/layout.rs", vec![x]),
                &config(),
                &never_called()
            )
            .len(),
            1
        );
    }
}

#[test]
fn ref_states_and_reasons_are_observable_without_strict_promotion() {
    let mut x = constant("0.6", 13);
    x.citation = Some(citation(CitationKind::Ref {
        path: "oracle/layout.rs",
        line: 120,
    }));
    let valid = MockFreshness::new(CitationFreshness::Valid);
    assert!(check(
        &file("01_core/layout.rs", vec![x.clone()]),
        &config(),
        &valid
    )
    .is_empty());
    assert_eq!(
        &*valid.calls.borrow(),
        &[("oracle/layout.rs".to_owned(), 120)]
    );
    let stale = MockFreshness::new(CitationFreshness::Stale(CitationStaleReason::MissingFile));
    let got = check(
        &file("01_core/strict/layout.rs", vec![x.clone()]),
        &strict_config(),
        &stale,
    );
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].rule_id, "V21");
    assert_eq!(got[0].level, ViolationLevel::Warning);
    for n in ["StaleCitation", "oracle/layout.rs", "120", "MissingFile"] {
        assert!(got[0].message.contains(n), "{n}");
    }
    let unknown = MockFreshness::new(CitationFreshness::Unknown(
        CitationUnknownReason::OutsideRoot,
    ));
    let got = check(
        &file("01_core/strict/layout.rs", vec![x]),
        &strict_config(),
        &unknown,
    );
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].level, ViolationLevel::Warning);
    for n in [
        "CitationFreshnessUnknown",
        "oracle/layout.rs",
        "120",
        "OutsideRoot",
    ] {
        assert!(got[0].message.contains(n), "{n}");
    }
}

#[test]
fn evidence_location_severity_order_and_multiplicity_are_preserved() {
    let first = constant("0.625 * style.size", 41);
    let dup = first.clone();
    let mut altered = constant("0.62", 10);
    altered.kind = ConstantKind::NegativeLiteral;
    altered.function_return_type = Some("Irrelevant");
    let got = check(
        &file("01_core/layout.rs", vec![first, dup, altered]),
        &config(),
        &never_called(),
    );
    assert_eq!(got.len(), 3);
    assert_eq!(
        got.iter().map(|v| v.location.line).collect::<Vec<_>>(),
        vec![41, 41, 10]
    );
    assert!(got.iter().all(|v| v.rule_id == "V21"));
    assert_eq!(got[0].level, ViolationLevel::Warning);
    assert_eq!(got[0].location.path, PathBuf::from("01_core/layout.rs"));
    assert_eq!(got[0].location.column, 44);
    for n in [
        "0.625 * style.size",
        "style.size",
        "layouter.regions.current.cursor_y",
    ] {
        assert!(got[0].message.contains(n), "{n}");
    }
    let got = check(
        &file("01_core/strict/layout.rs", vec![constant("0.6", 1)]),
        &strict_config(),
        &never_called(),
    );
    assert_eq!(got[0].level, ViolationLevel::Error);
    let got = check(
        &file("01_core/strictly/layout.rs", vec![constant("0.6", 1)]),
        &strict_config(),
        &never_called(),
    );
    assert_eq!(got[0].level, ViolationLevel::Warning);
}

#[test]
fn resolver_is_only_called_for_eligible_refs() {
    let mut x = constant("0.6", 1);
    x.is_in_binary_scaling = false;
    x.citation = Some(citation(CitationKind::Ref {
        path: "never.rs",
        line: 9,
    }));
    let r = MockFreshness::new(CitationFreshness::Valid);
    assert!(check(&file("01_core/layout.rs", vec![x]), &config(), &r).is_empty());
    assert!(r.calls.borrow().is_empty());
}
