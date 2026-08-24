use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::rule_traits::{
    BodyForm, DecisionArm, DecisionExpr, HasDecisionArms, ScrutineeForm,
};
use crystalline_lint::entities::violation::{Violation, ViolationLevel};
use crystalline_lint::rules::{
    compound_guard, deep_pattern_nesting, or_pattern_alternatives, range_pattern,
};
use std::path::Path;

struct F<'a> {
    layer: Layer,
    lang: Language,
    path: &'static Path,
    exprs: Vec<DecisionExpr<'a>>,
}
impl<'a> HasDecisionArms<'a> for F<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn decision_exprs(&self) -> &[DecisionExpr<'a>] {
        &self.exprs
    }
    fn path(&self) -> &'a Path {
        self.path
    }
    fn language(&self) -> &Language {
        &self.lang
    }
}
fn a(p: &'static str, l: usize, c: usize) -> DecisionArm<'static> {
    DecisionArm {
        pattern_snippet: p,
        is_catchall: false,
        bound_ident_used_in_body: false,
        qualified_prefixes: vec!["D"],
        has_guard: false,
        guard_is_compound: false,
        pattern_is_range: false,
        pattern_depth: 1,
        or_alternatives: 1,
        body_form: BodyForm::Other,
        body_snippet: "body",
        line: l,
        column: c,
    }
}
fn e(arms: Vec<DecisionArm<'static>>) -> DecisionExpr<'static> {
    DecisionExpr {
        snippet_scrutinee: "subject",
        scrutinee_form: ScrutineeForm::Path,
        arms,
        line: 1,
        column: 1,
    }
}
fn f(path: &str, exprs: Vec<DecisionExpr<'static>>) -> F<'static> {
    F {
        layer: Layer::Unknown,
        lang: Language::Rust,
        path: Box::leak(Path::new(path).to_path_buf().into_boxed_path()),
        exprs,
    }
}
fn one(v: &[Violation<'_>], id: &str, lvl: ViolationLevel, s: &str, p: &str, l: usize, c: usize) {
    assert_eq!(v.len(), 1, "{id}: {v:#?}");
    let v = &v[0];
    assert_eq!(v.rule_id, id);
    assert_eq!(v.level, lvl);
    assert!(v.message.contains(s));
    assert_eq!(v.location.path.as_ref(), Path::new(p));
    assert_eq!((v.location.line, v.location.column), (l, c));
}

#[test]
fn language_and_empty_matrix() {
    let mut x = a("A | B", 7, 9);
    x.has_guard = true;
    x.guard_is_compound = true;
    x.pattern_is_range = true;
    x.pattern_depth = u8::MAX;
    x.or_alternatives = u16::MAX;
    let mut z = f("src/x.rs", vec![e(vec![x])]);
    for lang in [
        Language::Python,
        Language::TypeScript,
        Language::C,
        Language::Cpp,
        Language::Zig,
        Language::Go,
        Language::Java,
        Language::Elixir,
        Language::Unknown,
    ] {
        z.lang = lang;
        assert!(compound_guard::check(&z).is_empty());
        assert!(range_pattern::check(&z).is_empty());
        assert!(or_pattern_alternatives::check(&z).is_empty());
        assert!(deep_pattern_nesting::check(&z).is_empty());
    }
    z.lang = Language::Rust;
    z.exprs.clear();
    assert!(compound_guard::check(&z).is_empty());
    assert!(range_pattern::check(&z).is_empty());
    assert!(or_pattern_alternatives::check(&z).is_empty());
    assert!(deep_pattern_nesting::check(&z).is_empty());
}
#[test]
fn v17_truth_table() {
    for (h, c, n) in [
        (false, false, 0),
        (false, true, 0),
        (true, false, 0),
        (true, true, 1),
    ] {
        let mut x = a("Some(x) if p(x) && q(x)", 17, 23);
        x.has_guard = h;
        x.guard_is_compound = c;
        let v = compound_guard::check(&f("src/g.rs", vec![e(vec![x])]));
        assert_eq!(v.len(), n);
        if n == 1 {
            one(
                &v,
                "V17",
                ViolationLevel::Warning,
                "Some(x) if p(x) && q(x)",
                "src/g.rs",
                17,
                23,
            )
        }
    }
}
#[test]
fn v18_component_case_separator_attacks() {
    for p in [
        "lexer.rs",
        "src/lexer.rs",
        "src/lexer/mod.rs",
        "src/numbering/x.rs",
        "src/syntax.rs",
        r"src\syntax\x.rs",
    ] {
        let mut x = a("'a'..='z'", 4, 6);
        x.pattern_is_range = true;
        assert!(
            range_pattern::check(&f(p, vec![e(vec![x])])).is_empty(),
            "{p}"
        );
    }
    for p in [
        "src/alexer.rs",
        "src/lexer_tools.rs",
        "src/numbering2.rs",
        "src/Syntax.rs",
        "src/LEXER/x.rs",
        "src/syntaxical/x.rs",
    ] {
        let mut x = a("0..=9", 8, 11);
        x.pattern_is_range = true;
        let v = range_pattern::check(&f(p, vec![e(vec![x])]));
        assert_eq!(v.len(), 1, "negative control unexpectedly exempt: {p}");
        one(&v, "V18", ViolationLevel::Warning, "0..=9", p, 8, 11);
    }
    assert!(range_pattern::check(&f("src/x.rs", vec![e(vec![a("0..=9", 1, 1)])])).is_empty());
}
#[test]
fn v19_threshold_max_count() {
    for n in [0, 1] {
        let mut x = a("A", 31, 5);
        x.or_alternatives = n;
        assert!(or_pattern_alternatives::check(&f("src/o.rs", vec![e(vec![x])])).is_empty())
    }
    for (n, s) in [(2, "A | B"), (u16::MAX, "Many | Alternatives")] {
        let mut x = a(s, 31, 5);
        x.or_alternatives = n;
        let v = or_pattern_alternatives::check(&f("src/o.rs", vec![e(vec![x])]));
        one(&v, "V19", ViolationLevel::Info, s, "src/o.rs", 31, 5);
        assert!(v[0].message.contains(&n.to_string()));
    }
}
fn deep(ps: &[&'static str]) -> Vec<DecisionArm<'static>> {
    ps.iter()
        .enumerate()
        .map(|(i, p)| {
            let mut x = a(p, i + 1, 1);
            x.pattern_depth = 4;
            x
        })
        .collect()
}
#[test]
fn v20_threshold_max_and_table_oracle() {
    for d in [0, 1, 2] {
        let mut x = a("Node(x)", 41, 7);
        x.pattern_depth = d;
        assert!(deep_pattern_nesting::check(&f("src/d.rs", vec![e(vec![x])])).is_empty())
    }
    for d in [3, u8::MAX] {
        let mut x = a("Node(Some(Ok(x)))", 41, 7);
        x.pattern_depth = d;
        let v = deep_pattern_nesting::check(&f("src/d.rs", vec![e(vec![x])]));
        one(
            &v,
            "V20",
            ViolationLevel::Info,
            "Node(Some(Ok(x)))",
            "src/d.rs",
            41,
            7,
        );
        assert!(v[0].message.contains(&d.to_string()))
    }
    let mut q = e(vec![{
        let mut x = a("heterogeneous", 1, 1);
        x.pattern_depth = 9;
        x
    }]);
    q.scrutinee_form = ScrutineeForm::Tuple;
    assert!(deep_pattern_nesting::check(&f("src/t.rs", vec![q])).is_empty());
    assert!(deep_pattern_nesting::check(&f(
        "src/t.rs",
        vec![e(deep(&["(A,X)", "(B,Y)", "(C,Z)"]))]
    ))
    .is_empty());
    assert_eq!(
        deep_pattern_nesting::check(&f("src/t.rs", vec![e(deep(&["(A,X)", "B", "(C,Z)"]))])).len(),
        3
    );
    assert_eq!(
        deep_pattern_nesting::check(&f("src/t.rs", vec![e(deep(&["(A,X)", "(B,Y)"]))])).len(),
        2
    );
    let mk = |middle: bool, two: bool| {
        let mut x = deep(&["(A,X)", "(B,Y)", "_"]);
        x[2].is_catchall = true;
        if middle {
            x.swap(1, 2)
        }
        if two {
            x[0].is_catchall = true
        }
        x
    };
    assert!(deep_pattern_nesting::check(&f("src/t.rs", vec![e(mk(false, false))])).is_empty());
    assert_eq!(
        deep_pattern_nesting::check(&f("src/t.rs", vec![e(mk(true, false))])).len(),
        3
    );
    assert_eq!(
        deep_pattern_nesting::check(&f("src/t.rs", vec![e(mk(false, true))])).len(),
        3
    );
    let mut x = deep(&["(A,X)", "(B,Y)", "(C,Z)"]);
    x[0].has_guard = true;
    x[0].guard_is_compound = true;
    assert!(deep_pattern_nesting::check(&f("src/t.rs", vec![e(x)])).is_empty());
}
#[test]
fn order_cardinality_and_isolation() {
    let mk = |s, l, c| {
        let mut x = a(s, l, c);
        x.has_guard = true;
        x.guard_is_compound = true;
        x
    };
    let v = compound_guard::check(&f(
        "src/order.rs",
        vec![
            e(vec![mk("FIRST", 10, 2), mk("SECOND", 20, 3)]),
            e(vec![mk("THIRD", 30, 4)]),
        ],
    ));
    assert_eq!(
        v.iter()
            .map(|x| (x.location.line, x.location.column))
            .collect::<Vec<_>>(),
        vec![(10, 2), (20, 3), (30, 4)]
    );
    let mut x = mk("stable guard", 55, 13);
    let b = compound_guard::check(&f("unicodé/路径.rs", vec![e(vec![x])])).remove(0);
    x = mk("stable guard", 55, 13);
    x.is_catchall = true;
    x.bound_ident_used_in_body = true;
    x.qualified_prefixes = vec!["N"];
    x.pattern_is_range = true;
    x.pattern_depth = u8::MAX;
    x.or_alternatives = u16::MAX;
    x.body_form = BodyForm::ErrorBarrier;
    x.body_snippet = "noise";
    let mut changed_expr = e(vec![x]);
    changed_expr.snippet_scrutinee = "changed scrutinee";
    changed_expr.scrutinee_form = ScrutineeForm::Tuple;
    changed_expr.line = usize::MAX;
    changed_expr.column = usize::MAX;
    let mut changed_file = f("unicodé/路径.rs", vec![changed_expr]);
    changed_file.layer = Layer::L1;
    let m = compound_guard::check(&changed_file).remove(0);
    assert_eq!(
        (b.rule_id, b.level, b.message, b.location),
        (m.rule_id, m.level, m.message, m.location)
    );

    let mut r = a("1..=9", 61, 2);
    r.pattern_is_range = true;
    r.has_guard = true;
    r.guard_is_compound = true;
    r.bound_ident_used_in_body = true;
    r.body_form = BodyForm::MessageProducer;
    r.pattern_depth = u8::MAX;
    r.or_alternatives = u16::MAX;
    one(
        &range_pattern::check(&f("src/ü.rs", vec![e(vec![r])])),
        "V18",
        ViolationLevel::Warning,
        "1..=9",
        "src/ü.rs",
        61,
        2,
    );
    let mut o = a("A | B", 62, 3);
    o.or_alternatives = 2;
    o.has_guard = true;
    o.pattern_is_range = true;
    o.body_form = BodyForm::ErrorBarrier;
    let v = or_pattern_alternatives::check(&f("src/ü.rs", vec![e(vec![o])]));
    one(&v, "V19", ViolationLevel::Info, "A | B", "src/ü.rs", 62, 3);
    let mut d = a("Outer(Middle(Inner))", 63, 4);
    d.pattern_depth = 3;
    d.has_guard = true;
    d.guard_is_compound = true;
    d.pattern_is_range = true;
    d.or_alternatives = u16::MAX;
    d.body_form = BodyForm::MessageProducer;
    let v = deep_pattern_nesting::check(&f("src/ü.rs", vec![e(vec![d])]));
    one(
        &v,
        "V20",
        ViolationLevel::Info,
        "Outer(Middle(Inner))",
        "src/ü.rs",
        63,
        4,
    );
}

fn same_diagnostic(a: &Violation<'_>, b: &Violation<'_>) {
    assert_eq!(a.rule_id, b.rule_id);
    assert_eq!(a.level, b.level);
    assert_eq!(a.message, b.message);
    assert_eq!(a.location, b.location);
}

#[test]
fn every_rule_preserves_multi_expression_multi_arm_order_and_cardinality() {
    let make = |s, l, c| {
        let mut x = a(s, l, c);
        x.has_guard = true;
        x.guard_is_compound = true;
        x.pattern_is_range = true;
        x.or_alternatives = 2;
        x.pattern_depth = 3;
        x
    };
    let input = || {
        f(
            "src/order-all.rs",
            vec![
                e(vec![make("FIRST", 10, 2), make("SECOND", 20, 3)]),
                e(vec![make("THIRD", 30, 4), make("FOURTH", 40, 5)]),
            ],
        )
    };
    let expected = vec![(10, 2), (20, 3), (30, 4), (40, 5)];
    let locations = |v: Vec<Violation<'_>>| {
        v.iter()
            .map(|x| (x.location.line, x.location.column))
            .collect::<Vec<_>>()
    };
    assert_eq!(locations(compound_guard::check(&input())), expected);
    assert_eq!(locations(range_pattern::check(&input())), expected);
    assert_eq!(
        locations(or_pattern_alternatives::check(&input())),
        expected
    );
    assert_eq!(locations(deep_pattern_nesting::check(&input())), expected);
}

#[test]
fn v18_systematic_irrelevant_field_mutation_preserves_entire_diagnostic() {
    let mut base = a("1..=9", 71, 8);
    base.pattern_is_range = true;
    let baseline = range_pattern::check(&f("src/domain.rs", vec![e(vec![base])])).remove(0);

    let mut changed = a("1..=9", 71, 8);
    changed.pattern_is_range = true;
    changed.is_catchall = true;
    changed.bound_ident_used_in_body = true;
    changed.qualified_prefixes = vec!["Other", "Noise"];
    changed.has_guard = true;
    changed.guard_is_compound = true;
    changed.pattern_depth = u8::MAX;
    changed.or_alternatives = u16::MAX;
    changed.body_form = BodyForm::ErrorBarrier;
    changed.body_snippet = "changed body";
    let mut changed_expr = e(vec![changed]);
    changed_expr.snippet_scrutinee = "changed scrutinee";
    changed_expr.scrutinee_form = ScrutineeForm::Tuple;
    changed_expr.line = usize::MAX;
    changed_expr.column = usize::MAX;
    let mut changed_file = f("src/domain.rs", vec![changed_expr]);
    changed_file.layer = Layer::L1;
    let mutated = range_pattern::check(&changed_file).remove(0);
    same_diagnostic(&baseline, &mutated);
}

#[test]
fn v19_systematic_irrelevant_field_mutation_preserves_entire_diagnostic() {
    let mut base = a("A | B", 72, 9);
    base.or_alternatives = 2;
    let baseline =
        or_pattern_alternatives::check(&f("src/domain.rs", vec![e(vec![base])])).remove(0);

    let mut changed = a("A | B", 72, 9);
    changed.or_alternatives = 2;
    changed.is_catchall = true;
    changed.bound_ident_used_in_body = true;
    changed.qualified_prefixes = vec!["Other", "Noise"];
    changed.has_guard = true;
    changed.guard_is_compound = true;
    changed.pattern_is_range = true;
    changed.pattern_depth = u8::MAX;
    changed.body_form = BodyForm::MessageProducer;
    changed.body_snippet = "changed body";
    let mut changed_expr = e(vec![changed]);
    changed_expr.snippet_scrutinee = "changed scrutinee";
    changed_expr.scrutinee_form = ScrutineeForm::Tuple;
    changed_expr.line = usize::MAX;
    changed_expr.column = usize::MAX;
    let mut changed_file = f("src/domain.rs", vec![changed_expr]);
    changed_file.layer = Layer::L1;
    let mutated = or_pattern_alternatives::check(&changed_file).remove(0);
    same_diagnostic(&baseline, &mutated);
}

#[test]
fn v20_systematic_irrelevant_field_mutation_preserves_entire_diagnostic() {
    let mut base = a("Outer(Middle(Inner))", 73, 10);
    base.pattern_depth = 3;
    let baseline = deep_pattern_nesting::check(&f("src/domain.rs", vec![e(vec![base])])).remove(0);

    let mut changed = a("Outer(Middle(Inner))", 73, 10);
    changed.pattern_depth = 3;
    changed.bound_ident_used_in_body = true;
    changed.qualified_prefixes = vec!["Other", "Noise"];
    changed.has_guard = true;
    changed.guard_is_compound = true;
    changed.pattern_is_range = true;
    changed.or_alternatives = u16::MAX;
    changed.body_form = BodyForm::MessageProducer;
    changed.body_snippet = "changed body";
    let mut changed_expr = e(vec![changed]);
    changed_expr.snippet_scrutinee = "changed scrutinee";
    changed_expr.line = usize::MAX;
    changed_expr.column = usize::MAX;
    let mut changed_file = f("src/domain.rs", vec![changed_expr]);
    changed_file.layer = Layer::L1;
    let mutated = deep_pattern_nesting::check(&changed_file).remove(0);
    same_diagnostic(&baseline, &mutated);
}
