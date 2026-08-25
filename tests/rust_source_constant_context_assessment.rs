use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::entities::rule_traits::ConstantKind;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::crate_registry::CrateRegistry;
use crystalline_lint::infra::rs_parser::RustParser;

struct NullPromptReader;

impl PromptReader for NullPromptReader {
    fn exists(&self, _path: &str) -> bool {
        false
    }

    fn read_hash(&self, _path: &str) -> Option<String> {
        None
    }
}

struct NullSnapshotReader;

impl PromptSnapshotReader for NullSnapshotReader {
    fn read_snapshot(&self, _path: &str) -> Option<PublicInterface<'static>> {
        None
    }

    fn serialize_snapshot(&self, _interface: &PublicInterface<'_>) -> String {
        String::new()
    }
}

fn source_file(source: &str) -> SourceFile {
    SourceFile {
        path: std::path::PathBuf::from("03_infra/context_assessment.rs"),
        content: source.to_owned(),
        language: Language::Rust,
        layer: Layer::L3,
        has_adjacent_test: false,
    }
}

fn parser() -> RustParser<NullPromptReader, NullSnapshotReader> {
    RustParser::new(
        NullPromptReader,
        NullSnapshotReader,
        CrystallineConfig::default(),
        CrateRegistry::default(),
    )
}

fn observe(source: &str) -> Result<Vec<(ConstantKind, String, usize, usize)>, ParseError> {
    let file = source_file(source);
    let parsed = parser().parse(&file)?;
    Ok(parsed
        .constants
        .iter()
        .map(|constant| {
            (
                constant.kind,
                constant.snippet.to_owned(),
                constant.line,
                constant.column,
            )
        })
        .collect())
}

fn parse_error(source: &str) -> ParseError {
    let file = source_file(source);
    parser()
        .parse(&file)
        .expect_err("invalid Rust must not expose partial IR")
}

#[test]
fn excludes_non_numeric_and_non_function_contexts() {
    let observed = observe(
        r#"
const TOP: i32 = 41;
static STATIC: i32 = 42;

fn candidate() {
    let named = TOP;
    let text = "43";
    let character = '4';
    let bytes = b"45";
    let byte = b'6';
    let expanded = value!(47);
    let range = 48..49;
    match named {
        50 => (),
        _ => (),
    }
}
"#,
    )
    .expect("valid Rust must parse");

    assert!(observed.is_empty());
}

#[test]
fn whitespace_and_comments_do_not_create_or_reorder_occurrences() {
    let observed = observe(
        r#"
fn spaced() {
    // 70 71 -72
    let first = 61;

    /* 73 */ let second =
        -62i16;
    let third = 61;
}
"#,
    )
    .expect("valid Rust must parse");

    assert_eq!(
        observed,
        vec![
            (ConstantKind::FunctionNumberLiteral, "61".to_owned(), 4, 17,),
            (ConstantKind::NegativeLiteral, "-62i16".to_owned(), 7, 9),
            (ConstantKind::FunctionNumberLiteral, "61".to_owned(), 8, 17,),
        ]
    );
}

#[test]
fn invalid_source_returns_syntax_error_without_partial_ir() {
    let error = parse_error("fn broken() { let visible = 81; let missing = ; }");

    assert!(matches!(error, ParseError::SyntaxError { .. }));
}
