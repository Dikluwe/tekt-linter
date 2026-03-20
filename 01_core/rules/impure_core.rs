//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/impure-core.md
//! @prompt-hash 3ff92bd1
//! @layer L1
//! @updated 2026-03-14

use std::borrow::Cow;

use crate::entities::rule_traits::HasTokens;
use crate::entities::layer::{Language, Layer};
use crate::entities::parsed_file::Token;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V4 — Impure core: forbidden I/O symbol detected in L1.
/// Operates semantically on ParsedFile.tokens (pre-extracted from AST by L3).
/// Never uses regex or string contains — only symbol comparison.
/// Token.symbol is Cow<'a, str> — V4 accesses via Deref as &str,
/// transparent to the Borrowed/Owned distinction.
fn forbidden_symbols_for(language: &Language) -> &'static [&'static str] {
    match language {
        Language::Rust => &[
            "std::fs", "std::io", "std::net", "std::process",
            "tokio::fs", "tokio::io", "tokio::process",
            "reqwest", "sqlx", "diesel",
            "std::time::SystemTime::now", "rand::random",
        ],
        Language::TypeScript => &[
            "fs", "node:fs", "fs/promises", "node:fs/promises",
            "child_process", "node:child_process",
            "net", "node:net", "http", "node:http",
            "https", "node:https", "dgram", "node:dgram",
            "dns", "node:dns", "readline", "node:readline",
            "process.env", "Date.now", "Math.random",
        ],
        Language::Python => &[
            "os", "os.path", "pathlib", "shutil", "subprocess",
            "socket", "urllib", "http.client", "ftplib", "smtplib",
            "open", "random.random", "time.time", "datetime.now",
        ],
        Language::Unknown => &[],
    }
}

pub fn check<'a, T: HasTokens<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    let forbidden = forbidden_symbols_for(file.language());

    file.tokens()
        .iter()
        .filter(|token| is_forbidden_symbol(&token.symbol, forbidden))
        .map(|token| make_violation(file, token))
        .collect()
}

fn is_forbidden_symbol(symbol: &str, forbidden: &[&str]) -> bool {
    forbidden.iter().any(|&f| {
        symbol == f
            || (symbol.starts_with(f) && symbol[f.len()..].starts_with("::"))
    })
}

fn make_violation<'a, T: HasTokens<'a>>(file: &T, token: &Token<'a>) -> Violation<'a> {
    Violation {
        rule_id: "V4".to_string(),
        level: ViolationLevel::Error,
        message: format!(
            "Núcleo Impuro: operação proibida '{}' detectada em L1",
            token.symbol
        ),
        location: Location { path: Cow::Borrowed(file.path()), line: token.line, column: token.column },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{Token, TokenKind};
    use std::borrow::Cow;
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        language: Language,
        tokens: Vec<Token<'static>>,
        path: &'static Path,
    }

    impl HasTokens<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &self.layer
        }
        fn tokens(&self) -> &[Token<'static>] {
            &self.tokens
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &self.language
        }
    }

    fn base_file(layer: Layer) -> MockFile {
        MockFile {
            layer,
            language: Language::Rust,
            tokens: vec![],
            path: Path::new("01_core/foo.rs"),
        }
    }

    fn call_token(symbol: &'static str, line: usize, column: usize) -> Token<'static> {
        Token {
            symbol: Cow::Borrowed(symbol),
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

    #[test]
    fn v4_flags_typescript_file_with_forbidden_symbol() {
        let path: &'static Path = Path::new("01_core/rules/mod.ts");
        let file = MockFile {
            layer: Layer::L1,
            language: Language::TypeScript,
            tokens: vec![Token {
                symbol: Cow::Borrowed("Date.now"),
                line: 5,
                column: 0,
                kind: TokenKind::CallExpression,
            }],
            path,
        };
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn v4_flags_python_file_with_forbidden_symbol() {
        let path: &'static Path = Path::new("01_core/rules/mod.py");
        let file = MockFile {
            layer: Layer::L1,
            language: Language::Python,
            tokens: vec![Token {
                symbol: Cow::Borrowed("os"),
                line: 3,
                column: 0,
                kind: TokenKind::CallExpression,
            }],
            path,
        };
        assert_eq!(check(&file).len(), 1);
    }

    #[test]
    fn v4_unknown_language_returns_no_violations() {
        let path: &'static Path = Path::new("01_core/rules/mod.txt");
        let file = MockFile {
            layer: Layer::L1,
            language: Language::Unknown,
            tokens: vec![Token {
                symbol: Cow::Borrowed("std::fs::read"),
                line: 1,
                column: 0,
                kind: TokenKind::CallExpression,
            }],
            path,
        };
        assert!(check(&file).is_empty());
    }
}
