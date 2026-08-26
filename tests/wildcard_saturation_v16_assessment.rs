use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::rule_traits::{
    BodyForm, DecisionArm, DecisionExpr, HasDecisionArms, ScrutineeForm,
};
use crystalline_lint::entities::violation::{Violation, ViolationLevel};
use crystalline_lint::rules::wildcard_saturation;
use std::collections::HashMap;
use std::path::Path;

struct MockFile<'a> {
    language: Language,
    path: &'a Path,
    exprs: Vec<DecisionExpr<'a>>,
}

impl<'a> HasDecisionArms<'a> for MockFile<'a> {
    fn layer(&self) -> &Layer {
        panic!("V16 must not inspect architecture layer")
    }
    fn decision_exprs(&self) -> &[DecisionExpr<'a>] {
        &self.exprs
    }
    fn path(&self) -> &'a Path {
        self.path
    }
    fn language(&self) -> &Language {
        &self.language
    }
}

fn arm<'a>(
    pattern: &'a str,
    catchall: bool,
    prefixes: Vec<&'a str>,
    body: BodyForm,
    body_snippet: &'a str,
    line: usize,
) -> DecisionArm<'a> {
    DecisionArm {
        pattern_snippet: pattern,
        is_catchall: catchall,
        bound_ident_used_in_body: false,
        qualified_prefixes: prefixes,
        has_guard: false,
        guard_is_compound: false,
        pattern_is_range: false,
        pattern_depth: 1,
        or_alternatives: 1,
        body_form: body,
        body_snippet,
        mergeability: None,
        line,
        column: 7,
    }
}

fn expr<'a>(form: ScrutineeForm, line: usize, catchall: DecisionArm<'a>) -> DecisionExpr<'a> {
    DecisionExpr {
        snippet_scrutinee: "kind_λ",
        scrutinee_form: form,
        arms: vec![
            arm(
                "Unit::Pt",
                false,
                vec!["Unit", "Unit"],
                BodyForm::Other,
                "a",
                line,
            ),
            arm(
                "Unit::Em",
                false,
                vec!["Unit"],
                BodyForm::Other,
                "b",
                line + 1,
            ),
            catchall,
        ],
        line,
        column: 3,
    }
}

fn file<'a>(language: Language, path: &'a str, exprs: Vec<DecisionExpr<'a>>) -> MockFile<'a> {
    MockFile {
        language,
        path: Path::new(path),
        exprs,
    }
}

fn check<'a>(f: &'a MockFile<'a>, exceptions: &HashMap<String, String>) -> Vec<Violation<'a>> {
    wildcard_saturation::check(f, exceptions)
}

fn eligible<'a>(
    form: ScrutineeForm,
    body: BodyForm,
    body_snippet: &'a str,
    line: usize,
) -> DecisionExpr<'a> {
    expr(
        form,
        line,
        arm("_λ", true, vec![], body, body_snippet, line + 2),
    )
}

#[test]
fn language_empty_and_all_seven_scrutinees_are_total() {
    let none = file(Language::Rust, "src/a.rs", vec![]);
    assert!(check(&none, &HashMap::new()).is_empty());

    for language in [
        Language::Python,
        Language::TypeScript,
        Language::Go,
        Language::Zig,
    ] {
        let f = file(
            language,
            "src/a.any",
            vec![eligible(
                ScrutineeForm::Path,
                BodyForm::Other,
                "fallback_λ",
                10,
            )],
        );
        assert!(check(&f, &HashMap::new()).is_empty());
    }

    let forms = [
        (ScrutineeForm::Path, true),
        (ScrutineeForm::FieldAccess, true),
        (ScrutineeForm::MethodCall, false),
        (ScrutineeForm::Index, false),
        (ScrutineeForm::Literal, false),
        (ScrutineeForm::Tuple, true),
        (ScrutineeForm::Other, true),
    ];
    for (form, expected) in forms {
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![eligible(form.clone(), BodyForm::Other, "fallback", 10)],
        );
        assert_eq!(check(&f, &HashMap::new()).len(), usize::from(expected));
    }
}

#[test]
fn candidate_counts_prefix_once_per_distinct_arm() {
    let cases = vec![
        (vec![vec!["Unit"], vec!["Unit"]], 1),
        (vec![vec!["Unit", "Unit"], vec!["Other"]], 0),
        (vec![vec!["Unit"], vec!["Other"]], 0),
        (vec![vec![""], vec![""]], 0),
    ];
    for (prefixes, expected) in cases {
        let arms = vec![
            arm("A", false, prefixes[0].clone(), BodyForm::Other, "a", 1),
            arm("B", false, prefixes[1].clone(), BodyForm::Other, "b", 2),
            arm("_", true, vec![], BodyForm::Other, "fallback", 3),
        ];
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![DecisionExpr {
                snippet_scrutinee: "x",
                scrutinee_form: ScrutineeForm::Path,
                arms,
                line: 1,
                column: 1,
            }],
        );
        assert_eq!(
            check(&f, &HashMap::new()).len(),
            expected,
            "prefix matrix {prefixes:?}"
        );
    }

    let f = file(
        Language::Rust,
        "src/a.rs",
        vec![DecisionExpr {
            snippet_scrutinee: "x",
            scrutinee_form: ScrutineeForm::Path,
            arms: vec![
                arm("A", false, vec!["Unit"], BodyForm::Other, "a", 1),
                arm("_", true, vec!["Unit"], BodyForm::Other, "fallback", 2),
            ],
            line: 1,
            column: 1,
        }],
    );
    assert_eq!(
        check(&f, &HashMap::new()).len(),
        1,
        "catch-all prefix participates"
    );
}

#[test]
fn catchall_reincorporation_barriers_and_every_body_form() {
    let bodies = [
        (BodyForm::ErrorBarrier, 0, ViolationLevel::Warning),
        (BodyForm::MessageProducer, 0, ViolationLevel::Warning),
        (BodyForm::EnumPath, 1, ViolationLevel::Warning),
        (BodyForm::LiteralNeutral, 1, ViolationLevel::Warning),
        (BodyForm::LiteralOther, 1, ViolationLevel::Warning),
        (BodyForm::Call, 1, ViolationLevel::Info),
        (BodyForm::EmptyBlock, 1, ViolationLevel::Warning),
        (BodyForm::Continue, 1, ViolationLevel::Warning),
        (BodyForm::Other, 1, ViolationLevel::Warning),
    ];
    for (body, count, level) in bodies {
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![eligible(ScrutineeForm::Path, body, "BODY_λ", 20)],
        );
        let got = check(&f, &HashMap::new());
        assert_eq!(got.len(), count);
        if count == 1 {
            assert_eq!(got[0].level, level);
        }
    }

    let mut not_catchall = arm("Unit::Other", false, vec![], BodyForm::Other, "BODY", 22);
    let f = file(
        Language::Rust,
        "src/a.rs",
        vec![expr(ScrutineeForm::Path, 20, not_catchall)],
    );
    assert!(check(&f, &HashMap::new()).is_empty());
    not_catchall = arm("other", true, vec![], BodyForm::Other, "use(other)", 22);
    not_catchall.bound_ident_used_in_body = true;
    let f = file(
        Language::Rust,
        "src/a.rs",
        vec![expr(ScrutineeForm::Path, 20, not_catchall)],
    );
    assert!(check(&f, &HashMap::new()).is_empty());
}

#[test]
fn principal_preserves_evidence_location_order_and_occurrences() {
    let f = file(
        Language::Rust,
        "src/λ.rs",
        vec![
            eligible(ScrutineeForm::Path, BodyForm::Other, "primeiro_λ", 10),
            eligible(
                ScrutineeForm::Tuple,
                BodyForm::LiteralNeutral,
                "segundo_β",
                usize::MAX - 3,
            ),
        ],
    );
    let got = check(&f, &HashMap::new());
    assert_eq!(got.len(), 2);
    assert!(got.iter().all(|v| v.rule_id == "V16"));
    assert!(
        got[0].message.contains("wildcard `_ =>`")
            && got[0].message.contains("_λ")
            && got[0].message.contains("primeiro_λ")
    );
    assert!(got[1].message.contains("segundo_β"));
    assert_eq!(got[0].location.path.as_ref(), Path::new("src/λ.rs"));
    assert_eq!((got[0].location.line, got[0].location.column), (12, 7));
    assert_eq!(got[1].location.line, usize::MAX - 1);
}

#[test]
fn active_exception_never_silences_principal_and_validates_justification() {
    for (reason, warning_count) in [
        ("razão documentada", 0),
        ("", 1),
        ("   ", 1),
        ("ok", 1),
        ("OK", 1),
        ("ok.", 0),
        ("okay", 0),
        ("N16[α] razão", 0),
        ("N16[A] razão", 0),
        ("N16[b] razão", 0),
        ("N16[γ] razão", 0),
        ("N16[z] razão", 1),
    ] {
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![eligible(
                ScrutineeForm::Path,
                BodyForm::Other,
                "fallback",
                10,
            )],
        );
        let mut ex = HashMap::new();
        ex.insert("src/a.rs:12".into(), reason.into());
        let got = check(&f, &ex);
        assert_eq!(got.len(), 1 + warning_count, "reason={reason:?}");
        assert_eq!(got.last().unwrap().rule_id, "V16");
        assert!(got.last().unwrap().message.contains("fallback"));
    }
}

#[test]
fn exact_path_line_matching_and_stale_sort_are_deterministic() {
    let build = |reverse: bool| {
        let mut ex = HashMap::new();
        let entries = [
            ("src/a.rs:99", "stale z"),
            ("src/a.rs:4", "stale a"),
            ("src/a.rs:12", "valid"),
            ("src/aa.rs:12", "other"),
            ("/src/a.rs:12", "absolute"),
            ("src/a.rs:not-line", "bad"),
            ("src\\a.rs:12", "separator"),
        ];
        if reverse {
            for (k, v) in entries.iter().rev() {
                ex.insert((*k).into(), (*v).into());
            }
        } else {
            for (k, v) in entries {
                ex.insert(k.into(), v.into());
            }
        }
        ex
    };
    let f = file(
        Language::Rust,
        "src/a.rs",
        vec![eligible(
            ScrutineeForm::Path,
            BodyForm::Other,
            "fallback",
            10,
        )],
    );
    let a = check(&f, &build(false));
    let b = check(&f, &build(true));
    let projection = |xs: &[Violation<'_>]| {
        xs.iter()
            .map(|v| {
                (
                    v.rule_id.clone(),
                    v.level.clone(),
                    v.message.clone(),
                    v.location.line,
                    v.location.column,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(projection(&a), projection(&b));
    assert_eq!(
        a.len(),
        3,
        "principal plus two stale current-file exceptions"
    );
    assert!(
        a[0].message.contains("fallback"),
        "valid exception has no warning"
    );
    assert_eq!(
        [a[1].location.line, a[2].location.line],
        [4, 99],
        "stale keys sort lexically by complete key"
    );
}

#[test]
fn syntactic_catchall_keeps_exception_fresh_even_when_v16_exempt() {
    let mut ex = HashMap::new();
    ex.insert("src/a.rs:12".into(), "documented".into());
    for form in [
        ScrutineeForm::MethodCall,
        ScrutineeForm::Index,
        ScrutineeForm::Literal,
    ] {
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![eligible(form.clone(), BodyForm::Other, "fallback", 10)],
        );
        let got = check(&f, &ex);
        assert!(
            got.is_empty(),
            "open scrutinee {form:?}: {:?}",
            got.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }
    for body in [BodyForm::ErrorBarrier, BodyForm::MessageProducer] {
        let f = file(
            Language::Rust,
            "src/a.rs",
            vec![eligible(ScrutineeForm::Path, body.clone(), "fallback", 10)],
        );
        let got = check(&f, &ex);
        assert!(
            got.is_empty(),
            "barrier {body:?}: {:?}",
            got.iter().map(|v| &v.message).collect::<Vec<_>>()
        );
    }
    let mut a = arm("other", true, vec![], BodyForm::Other, "use(other)", 12);
    a.bound_ident_used_in_body = true;
    let f = file(
        Language::Rust,
        "src/a.rs",
        vec![expr(ScrutineeForm::Path, 10, a)],
    );
    assert!(check(&f, &ex).is_empty());

    let no_candidate = DecisionExpr {
        snippet_scrutinee: "x",
        scrutinee_form: ScrutineeForm::Path,
        arms: vec![
            arm("A", false, vec!["One"], BodyForm::Other, "a", 10),
            arm("B", false, vec!["Two"], BodyForm::Other, "b", 11),
            arm("_", true, vec![], BodyForm::Other, "fallback", 12),
        ],
        line: 10,
        column: 1,
    };
    let f = file(Language::Rust, "src/a.rs", vec![no_candidate]);
    assert!(
        check(&f, &ex).is_empty(),
        "syntactic catch-all is fresh without enum candidacy"
    );
}

#[test]
fn irrelevant_fields_unicode_and_extremes_do_not_change_v16() {
    let mut base = arm("_λ", true, vec![], BodyForm::Other, "β", usize::MAX);
    base.has_guard = true;
    base.guard_is_compound = true;
    base.pattern_is_range = true;
    base.pattern_depth = u8::MAX;
    base.or_alternatives = u16::MAX;
    base.column = usize::MAX;
    let f = file(
        Language::Rust,
        "/absoluto/λ.rs",
        vec![expr(ScrutineeForm::Other, 0, base)],
    );
    let got = check(&f, &HashMap::new());
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].rule_id, "V16");
    assert!(!got
        .iter()
        .any(|v| matches!(v.rule_id.as_str(), "V17" | "V18" | "V19" | "V20")));
    assert_eq!(
        (got[0].location.line, got[0].location.column),
        (usize::MAX, usize::MAX)
    );
}
