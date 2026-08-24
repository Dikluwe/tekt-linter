//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/infrastructure-error.md
//! @prompt-hash 4f9c2e89
//! @layer L1
//! @updated 2026-08-24

use std::borrow::Cow;

use crate::contracts::file_provider::SourceError;
use crate::contracts::parse_error::ParseError;
use crate::entities::violation::{Location, Violation, ViolationLevel};

pub fn source_error_to_violation(err: &SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location {
                path: Cow::Owned(path.clone()),
                line: 0,
                column: 0,
            },
        },
    }
}

pub fn parse_error_to_violation(err: ParseError) -> Violation<'static> {
    match err {
        ParseError::SyntaxError {
            path,
            line,
            column,
            message,
        } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Erro de sintaxe: {message}"),
            location: Location {
                path: Cow::Owned(path),
                line,
                column,
            },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Linguagem não suportada: {language:?}"),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Arquivo vazio ignorado".to_string(),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
    }
}
