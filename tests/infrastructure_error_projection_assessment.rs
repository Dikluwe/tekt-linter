use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crystalline_lint::contracts::file_provider::SourceError;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::entities::layer::Language;
use crystalline_lint::entities::violation::{Violation, ViolationLevel};
use crystalline_lint::rules::infrastructure_error::{
    parse_error_to_violation, source_error_to_violation,
};

fn assert_owned_path(violation: &Violation<'_>, expected: &Path) {
    match &violation.location.path {
        Cow::Owned(path) => assert_eq!(path, expected),
        Cow::Borrowed(path) => panic!("expected Cow::Owned, got borrowed path {path:?}"),
    }
}

#[test]
fn unreadable_is_exactly_one_owned_fatal_v0_and_preserves_hostile_evidence() {
    let path = PathBuf::from("projeto/\u{1f9ea}/linha\n\t\0.rs");
    let reason = "permissão negada: \"x\"\n\t\0🧨".to_owned();
    let error = SourceError::Unreadable {
        path: path.clone(),
        reason: reason.clone(),
    };
    let unchanged = error.clone();

    let projected = vec![source_error_to_violation(&error)];

    assert_eq!(projected.len(), 1);
    let violation = &projected[0];
    assert_eq!(violation.rule_id, "V0");
    assert_eq!(violation.level, ViolationLevel::Fatal);
    assert_eq!(violation.message, format!("Arquivo ilegível: {reason}"));
    assert_eq!((violation.location.line, violation.location.column), (0, 0));
    assert_owned_path(violation, &path);
    assert_eq!(error, unchanged, "the borrowed input must not be modified");
    assert_eq!(source_error_to_violation(&error), violation.clone());
    assert_eq!(source_error_to_violation(&error.clone()), violation.clone());
}

#[test]
fn unreadable_preserves_an_empty_reason_exactly() {
    let path = PathBuf::from("vazio.rs");
    let error = SourceError::Unreadable {
        path: path.clone(),
        reason: String::new(),
    };

    let violation = source_error_to_violation(&error);

    assert_eq!(violation.message, "Arquivo ilegível: ");
    assert_owned_path(&violation, &path);
}

#[test]
fn syntax_error_is_exactly_one_owned_parse_error_with_preserved_position_and_message() {
    let path = PathBuf::from("fonte/λ/erro\n.rs");
    let message = "esperado `}`; recebido \"\0\" 🦀".to_owned();
    let error = ParseError::SyntaxError {
        path: path.clone(),
        line: 37,
        column: 19,
        message: message.clone(),
    };

    let first = vec![parse_error_to_violation(error.clone())];
    let second = vec![parse_error_to_violation(error.clone())];

    assert_eq!(first.len(), 1);
    assert_eq!(
        first, second,
        "clones and repetitions must be deterministic"
    );
    let violation = &first[0];
    assert_eq!(violation.rule_id, "PARSE");
    assert_eq!(violation.level, ViolationLevel::Error);
    assert_eq!(violation.message, format!("Erro de sintaxe: {message}"));
    assert_eq!(
        (violation.location.line, violation.location.column),
        (37, 19)
    );
    assert_owned_path(violation, &path);
}

#[test]
fn syntax_error_preserves_an_empty_causal_message_exactly() {
    let violation = parse_error_to_violation(ParseError::SyntaxError {
        path: PathBuf::from("empty-message.rs"),
        line: 1,
        column: 2,
        message: String::new(),
    });

    assert_eq!(violation.message, "Erro de sintaxe: ");
    assert_eq!((violation.location.line, violation.location.column), (1, 2));
}

#[test]
fn unsupported_language_is_exactly_one_owned_parse_warning_at_unavailable_position() {
    let path = PathBuf::from("未知/arquivo.💎");
    let error = ParseError::UnsupportedLanguage {
        path: path.clone(),
        language: Language::Unknown,
    };

    let projected = vec![parse_error_to_violation(error.clone())];

    assert_eq!(projected.len(), 1);
    let violation = &projected[0];
    assert_eq!(violation.rule_id, "PARSE");
    assert_eq!(violation.level, ViolationLevel::Warning);
    assert_eq!(violation.message, "Linguagem não suportada: Unknown");
    assert_eq!((violation.location.line, violation.location.column), (0, 0));
    assert_owned_path(violation, &path);
    assert_eq!(
        parse_error_to_violation(error.clone()),
        parse_error_to_violation(error)
    );
}

#[test]
fn empty_source_is_exactly_one_owned_parse_warning_at_unavailable_position() {
    let path = PathBuf::from("🫙/empty\n\t\0.rs");
    let error = ParseError::EmptySource { path: path.clone() };

    let projected = vec![parse_error_to_violation(error.clone())];

    assert_eq!(projected.len(), 1);
    let violation = &projected[0];
    assert_eq!(violation.rule_id, "PARSE");
    assert_eq!(violation.level, ViolationLevel::Warning);
    assert_eq!(violation.message, "Arquivo vazio ignorado");
    assert_eq!((violation.location.line, violation.location.column), (0, 0));
    assert_owned_path(violation, &path);
    assert_eq!(
        parse_error_to_violation(error.clone()),
        parse_error_to_violation(error)
    );
}

#[test]
fn four_modalities_do_not_silence_or_exchange_ids_levels_or_cardinality() {
    let violations = vec![
        source_error_to_violation(&SourceError::Unreadable {
            path: PathBuf::from("a"),
            reason: "r".to_owned(),
        }),
        parse_error_to_violation(ParseError::SyntaxError {
            path: PathBuf::from("b"),
            line: 8,
            column: 13,
            message: "m".to_owned(),
        }),
        parse_error_to_violation(ParseError::UnsupportedLanguage {
            path: PathBuf::from("c"),
            language: Language::Unknown,
        }),
        parse_error_to_violation(ParseError::EmptySource {
            path: PathBuf::from("d"),
        }),
    ];

    assert_eq!(violations.len(), 4);
    assert_eq!(violations.iter().filter(|v| v.rule_id == "V0").count(), 1);
    assert_eq!(
        violations.iter().filter(|v| v.rule_id == "PARSE").count(),
        3
    );
    assert_eq!(
        violations.iter().map(|v| &v.level).collect::<Vec<_>>(),
        vec![
            &ViolationLevel::Fatal,
            &ViolationLevel::Error,
            &ViolationLevel::Warning,
            &ViolationLevel::Warning,
        ]
    );
}
