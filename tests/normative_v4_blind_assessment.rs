use std::borrow::Cow;
use std::path::Path;

use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::{Token, TokenKind};
use crystalline_lint::entities::rule_traits::HasTokens;
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::impure_core;

const RUST_FORBIDDEN: &[&str] = &[
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

const TYPESCRIPT_FORBIDDEN: &[&str] = &[
    "fs",
    "node:fs",
    "fs/promises",
    "node:fs/promises",
    "child_process",
    "node:child_process",
    "net",
    "node:net",
    "http",
    "node:http",
    "https",
    "node:https",
    "dgram",
    "node:dgram",
    "dns",
    "node:dns",
    "readline",
    "node:readline",
    "process.env",
    "Date.now",
    "Math.random",
];

const PYTHON_FORBIDDEN: &[&str] = &[
    "os",
    "os.path",
    "pathlib",
    "shutil",
    "subprocess",
    "socket",
    "urllib",
    "http.client",
    "ftplib",
    "smtplib",
    "open",
    "random.random",
    "time.time",
    "datetime.now",
];

const C_FORBIDDEN: &[&str] = &[
    "stdio.h",
    "stdlib.h",
    "time.h",
    "unistd.h",
    "fcntl.h",
    "sys/socket.h",
    "pthread.h",
    "sys/stat.h",
    "windows.h",
];
const CPP_FORBIDDEN: &[&str] = &[
    "iostream",
    "fstream",
    "thread",
    "mutex",
    "chrono",
    "filesystem",
    "net",
    "stdio.h",
    "stdlib.h",
    "time.h",
    "unistd.h",
    "sys/socket.h",
    "windows.h",
];
const ZIG_FORBIDDEN: &[&str] = &[
    "std.fs",
    "std.io",
    "std.net",
    "std.os",
    "std.process",
    "std.time",
    "std.crypto",
];
const GO_FORBIDDEN: &[&str] = &[
    "os",
    "net",
    "net/http",
    "io/ioutil",
    "os/exec",
    "database/sql",
];
const JAVA_FORBIDDEN: &[&str] = &[
    "java.io",
    "java.net",
    "java.nio.file",
    "java.lang.ProcessBuilder",
    "java.sql",
    "javax.sql",
    "System.out",
    "System.err",
    "System.currentTimeMillis",
];
const ELIXIR_FORBIDDEN: &[&str] = &[
    "File",
    "File.Stream",
    "IO",
    "Path",
    "System.cmd",
    "HTTPoison",
    "Req",
    "Ecto",
];

fn normative_tables() -> [(Language, &'static [&'static str]); 9] {
    [
        (Language::Rust, RUST_FORBIDDEN),
        (Language::TypeScript, TYPESCRIPT_FORBIDDEN),
        (Language::Python, PYTHON_FORBIDDEN),
        (Language::C, C_FORBIDDEN),
        (Language::Cpp, CPP_FORBIDDEN),
        (Language::Zig, ZIG_FORBIDDEN),
        (Language::Go, GO_FORBIDDEN),
        (Language::Java, JAVA_FORBIDDEN),
        (Language::Elixir, ELIXIR_FORBIDDEN),
    ]
}

fn matches_table(symbol: &str, forbidden: &[&str]) -> bool {
    forbidden.iter().any(|entry| {
        symbol == *entry
            || symbol
                .strip_prefix(entry)
                .is_some_and(|suffix| suffix.starts_with("::") || suffix.starts_with('.'))
    })
}

struct MockFile<'a> {
    layer: Layer,
    language: Language,
    path: &'a Path,
    tokens: Vec<Token<'a>>,
}

impl<'a> HasTokens<'a> for MockFile<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }

    fn tokens(&self) -> &[Token<'a>] {
        &self.tokens
    }

    fn path(&self) -> &'a Path {
        self.path
    }

    fn language(&self) -> &Language {
        &self.language
    }
}

fn token(symbol: String, line: usize, column: usize) -> Token<'static> {
    Token {
        symbol: Cow::Owned(symbol),
        line,
        column,
        kind: TokenKind::CallExpression,
    }
}

fn file(language: Language, layer: Layer, symbols: Vec<String>) -> MockFile<'static> {
    MockFile {
        layer,
        language,
        path: Path::new("01_core/domain/pure.rs"),
        tokens: symbols
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| token(symbol, index + 10, index + 20))
            .collect(),
    }
}

fn assert_complete_table(language: Language, forbidden: &[&str]) {
    let mut symbols = Vec::new();
    for entry in forbidden {
        symbols.push((*entry).to_string());
        symbols.push(format!("{entry}::member"));
        symbols.push(format!("{entry}.member"));
    }

    let mock = file(language, Layer::L1, symbols.clone());
    let violations = impure_core::check(&mock);
    assert_eq!(violations.len(), symbols.len());

    for (index, (violation, symbol)) in violations.iter().zip(symbols.iter()).enumerate() {
        assert_eq!(violation.rule_id, "V4");
        assert_eq!(violation.level, ViolationLevel::Error);
        assert_eq!(
            violation.message,
            format!("Núcleo Impuro: operação proibida '{symbol}' detectada em L1")
        );
        assert_eq!(violation.location.path.as_ref(), mock.path);
        assert_eq!(violation.location.line, index + 10);
        assert_eq!(violation.location.column, index + 20);
    }
}

#[test]
fn every_normative_entry_accepts_equality_and_both_delimited_prefix_forms() {
    assert_complete_table(Language::Rust, RUST_FORBIDDEN);
    assert_complete_table(Language::TypeScript, TYPESCRIPT_FORBIDDEN);
    assert_complete_table(Language::Python, PYTHON_FORBIDDEN);
    assert_complete_table(Language::C, C_FORBIDDEN);
    assert_complete_table(Language::Cpp, CPP_FORBIDDEN);
    assert_complete_table(Language::Zig, ZIG_FORBIDDEN);
    assert_complete_table(Language::Go, GO_FORBIDDEN);
    assert_complete_table(Language::Java, JAVA_FORBIDDEN);
    assert_complete_table(Language::Elixir, ELIXIR_FORBIDDEN);
}

#[test]
fn near_misses_for_every_normative_entry_are_allowed() {
    for (language, forbidden) in normative_tables() {
        let symbols = forbidden
            .iter()
            .flat_map(|entry| [format!("x{entry}"), format!("{entry}_near")])
            .filter(|candidate| !matches_table(candidate, forbidden))
            .collect();
        let mock = file(language, Layer::L1, symbols);
        assert!(impure_core::check(&mock).is_empty());
    }
}

#[test]
fn language_tables_are_isolated() {
    for (language, forbidden) in normative_tables() {
        for (other_language, other_forbidden) in normative_tables() {
            if language == other_language {
                continue;
            }
            let candidates = other_forbidden
                .iter()
                .copied()
                .filter(|symbol| !matches_table(symbol, forbidden))
                .map(str::to_string)
                .collect();
            assert!(impure_core::check(&file(language.clone(), Layer::L1, candidates)).is_empty());
        }
    }
}

#[test]
fn unknown_language_has_no_forbidden_symbols() {
    let mock = file(
        Language::Unknown,
        Layer::L1,
        vec![
            "std::fs::read".into(),
            "fs.readFileSync".into(),
            "os.path.join".into(),
            "anything".into(),
        ],
    );
    assert!(impure_core::check(&mock).is_empty());
}

#[test]
fn v4_is_scoped_exclusively_to_l1() {
    for layer in [
        Layer::L0,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        let mock = file(Language::Rust, layer, vec!["std::fs::read".into()]);
        assert!(impure_core::check(&mock).is_empty());
    }
}

#[test]
fn violations_preserve_token_order_multiplicity_and_exact_locations() {
    let mock = MockFile {
        layer: Layer::L1,
        language: Language::Rust,
        path: Path::new("01_core/entities/value.rs"),
        tokens: vec![
            token("reqwest::get".into(), 91, 7),
            token("pure::calculate".into(), 4, 2),
            token("reqwest::get".into(), 12, 33),
            token("rand::random".into(), 1, 0),
        ],
    };

    let violations = impure_core::check(&mock);
    assert_eq!(violations.len(), 3);
    assert_eq!(
        violations
            .iter()
            .map(|v| (v.location.line, v.location.column))
            .collect::<Vec<_>>(),
        vec![(91, 7), (12, 33), (1, 0)]
    );
    assert!(violations
        .iter()
        .all(|v| v.location.path.as_ref() == mock.path));
}

#[test]
fn borrowed_and_owned_symbols_are_treated_identically() {
    let mock = MockFile {
        layer: Layer::L1,
        language: Language::Rust,
        path: Path::new("01_core/entities/alias.rs"),
        tokens: vec![
            Token {
                symbol: Cow::Borrowed("std::fs::read"),
                line: 10,
                column: 3,
                kind: TokenKind::CallExpression,
            },
            token("std::fs::read".into(), 11, 4),
        ],
    };

    let violations = impure_core::check(&mock);
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].message, violations[1].message);
}
