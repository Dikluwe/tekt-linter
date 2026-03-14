//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/impure-core.md
//! @prompt-hash 3f1c25cf
//! @layer L1
//! @updated 2026-03-14

use crate::entities::layer::Layer;
use crate::entities::parsed_file::Token;
use crate::entities::violation::{Location, Violation, ViolationLevel};
use std::path::Path;

pub trait HasTokens {
    fn layer(&self) -> &Layer;
    fn tokens(&self) -> &[Token];
    fn path(&self) -> &Path;
}

/// V4 — Impure core: forbidden I/O symbol detected in L1.
/// Operates semantically on ParsedFile.tokens (pre-extracted from AST by L3).
/// Never uses regex or string contains — only symbol comparison.
const FORBIDDEN_SYMBOLS: &[&str] = &[
    "std::fs",
    "std::io",
    "std::net",
    "std::process",
    "tokio::fs",
    "tokio::io",
    "tokio::process",
    "reqwest",
    "sqlx",
    "diesel",
    "std::time::SystemTime::now",
    "rand::random",
];

pub fn check<T: HasTokens>(file: &T) -> Vec<Violation> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.tokens()
        .iter()
        .filter(|token| is_forbidden_symbol(&token.symbol))
        .map(|token| make_violation(file, token))
        .collect()
}

fn is_forbidden_symbol(symbol: &str) -> bool {
    FORBIDDEN_SYMBOLS
        .iter()
        .any(|&forbidden| symbol == forbidden || symbol.starts_with(&format!("{}::", forbidden)))
}

fn make_violation<T: HasTokens>(file: &T, token: &Token) -> Violation {
    Violation {
        rule_id: "V4".to_string(),
        level: ViolationLevel::Error,
        message: format!(
            "Núcleo Impuro: operação proibida '{}' detectada em L1",
            token.symbol
        ),
        location: Location {
            path: file.path().to_path_buf(),
            line: token.line,
            column: token.column,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::{Token, TokenKind};
    use std::path::{Path, PathBuf};

    struct MockFile {
        layer: Layer,
        tokens: Vec<Token>,
        path: PathBuf,
    }

    impl HasTokens for MockFile {
        fn layer(&self) -> &Layer {
            &self.layer
        }
        fn tokens(&self) -> &[Token] {
            &self.tokens
        }
        fn path(&self) -> &Path {
            &self.path
        }
    }

    fn base_file(layer: Layer) -> MockFile {
        MockFile {
            layer,
            tokens: vec![],
            path: PathBuf::from("01_core/foo.rs"),
        }
    }

    fn call_token(symbol: &str, line: usize, column: usize) -> Token {
        Token {
            symbol: symbol.to_string(),
            line,
            column,
            kind: TokenKind::CallExpression,
        }
    }

    #[test]
    fn std_fs_read_in_l1_is_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("std::fs::read", 10, 4));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V4");
        assert_eq!(violations[0].location.line, 10);
        assert_eq!(violations[0].location.column, 4);
    }

    #[test]
    fn std_net_in_l1_is_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("std::net::TcpStream", 7, 0));
        let violations = check(&file);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V4");
    }

    #[test]
    fn system_time_now_in_l1_is_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("std::time::SystemTime::now", 3, 0));
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn reqwest_in_l1_is_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("reqwest::get", 5, 0));
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn pure_function_call_in_l1_is_not_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("my_module::compute", 2, 0));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn forbidden_symbol_in_l3_is_not_violation() {
        // V4 only fires for L1
        let mut file = base_file(Layer::L3);
        file.tokens.push(call_token("std::fs::read", 10, 0));
        assert!(check(&file).is_empty());
    }

    #[test]
    fn multiple_forbidden_tokens_each_produce_violation() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("std::fs::write", 3, 0));
        file.tokens.push(call_token("tokio::io::stdin", 8, 0));
        assert_eq!(check(&file).len(), 2);
    }

    #[test]
    fn violation_message_contains_symbol() {
        let mut file = base_file(Layer::L1);
        file.tokens.push(call_token("sqlx::query", 6, 0));
        let violations = check(&file);
        assert!(violations[0].message.contains("sqlx::query"));
    }
}
