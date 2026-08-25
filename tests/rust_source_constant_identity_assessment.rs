use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::entities::rule_traits::ConstantKind;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::crate_registry::CrateRegistry;
use crystalline_lint::infra::rs_parser::RustParser;

struct NoPrompts;

impl PromptReader for NoPrompts {
    fn exists(&self, _path: &str) -> bool {
        false
    }

    fn read_hash(&self, _path: &str) -> Option<String> {
        None
    }
}

struct NoSnapshots;

impl PromptSnapshotReader for NoSnapshots {
    fn read_snapshot(&self, _path: &str) -> Option<PublicInterface<'static>> {
        None
    }

    fn serialize_snapshot(&self, _interface: &PublicInterface<'_>) -> String {
        String::new()
    }
}

fn observed(source: &str) -> Vec<(ConstantKind, String, usize, usize)> {
    let file = SourceFile {
        path: "01_core/identity.rs".into(),
        content: source.to_owned(),
        language: Language::Rust,
        layer: Layer::L1,
        has_adjacent_test: true,
    };
    let parser = RustParser::new(
        NoPrompts,
        NoSnapshots,
        CrystallineConfig::default(),
        CrateRegistry::default(),
    );
    parser
        .parse(&file)
        .expect("a fixture Rust valida deve produzir IR")
        .constants
        .into_iter()
        .filter_map(|constant| {
            matches!(
                constant.kind,
                ConstantKind::FunctionNumberLiteral | ConstantKind::NegativeLiteral
            )
            .then(|| {
                (
                    constant.kind,
                    constant.snippet.to_owned(),
                    constant.line,
                    constant.column,
                )
            })
        })
        .collect()
}

#[test]
fn preserves_numeric_identity_suffix_sign_and_byte_columns() {
    let source =
        "fn identity() {\n    let prefix = \"ação\"; let a = 12u32;\n    let b = -3.5f64;\n}\n";

    assert_eq!(
        observed(source),
        vec![
            (ConstantKind::FunctionNumberLiteral, "12u32".into(), 2, 36),
            (ConstantKind::NegativeLiteral, "-3.5f64".into(), 3, 13),
        ]
    );
}

#[test]
fn preserves_preorder_nesting_repetition_and_multiplicity() {
    let source = "fn nested() { consume(7, wrap(8 + 7), -9); }\n";

    assert_eq!(
        observed(source),
        vec![
            (ConstantKind::FunctionNumberLiteral, "7".into(), 1, 23),
            (ConstantKind::FunctionNumberLiteral, "8".into(), 1, 31),
            (ConstantKind::FunctionNumberLiteral, "7".into(), 1, 35),
            (ConstantKind::NegativeLiteral, "-9".into(), 1, 39),
        ]
    );
}

#[test]
fn excludes_non_numeric_forms_even_when_nested_in_a_function() {
    let source = r#"fn exclusions() {
    let _ = "12";
    let _ = '3';
    let _ = b'4';
    let _ = VALUE;
    emit!(5);
}
"#;

    assert!(observed(source).is_empty());
}
