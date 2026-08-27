//! Oráculo P0118 — cada parser concreto transporta o papel recebido sem reclassificá-lo.

use crystalline_lint::contracts::file_provider::{SourceFile, SourceRole};
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::prompt_reader::{
    PromptHashResolution, PromptPresence, PromptReader,
};
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::infra::c_parser::CParser;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::cpp_parser::CppParser;
use crystalline_lint::infra::crate_registry::CrateRegistry;
use crystalline_lint::infra::elixir_parser::ElixirParser;
use crystalline_lint::infra::go_parser::GoParser;
use crystalline_lint::infra::java_parser::JavaParser;
use crystalline_lint::infra::py_parser::PyParser;
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::infra::ts_parser::TsParser;
use crystalline_lint::infra::zig_parser::ZigParser;
use std::path::PathBuf;

#[derive(Clone, Copy)]
struct NoPrompts;

impl PromptReader for NoPrompts {
    fn presence(&self, _prompt_path: &str) -> PromptPresence {
        PromptPresence::Missing
    }

    fn read_hash(&self, _prompt_path: &str) -> PromptHashResolution {
        PromptHashResolution::Missing
    }
}

impl PromptSnapshotReader for NoPrompts {
    fn read_snapshot(&self, _prompt_path: &str) -> Option<PublicInterface<'static>> {
        None
    }

    fn serialize_snapshot(&self, _interface: &PublicInterface<'_>) -> String {
        String::new()
    }
}

fn source(language: Language, content: &str) -> SourceFile {
    SourceFile {
        path: PathBuf::from("01_core/nome_produtivo.ext"),
        content: content.to_string(),
        language,
        layer: Layer::L1,
        source_role: SourceRole::Test,
        has_adjacent_test: false,
    }
}

macro_rules! assert_transport {
    ($parser:expr, $language:expr, $content:expr) => {{
        let source = source($language, $content);
        let parsed = $parser.parse(&source).expect("valid synthetic source");
        assert_eq!(parsed.source_role, SourceRole::Test);
    }};
}

#[test]
fn all_nine_concrete_parsers_transport_test_role_exactly() {
    let config = CrystallineConfig::default();
    let root = PathBuf::from(".");

    assert_transport!(
        RustParser::new(
            NoPrompts,
            NoPrompts,
            config.clone(),
            CrateRegistry::default()
        ),
        Language::Rust,
        "fn value() -> i32 { 1 }\n"
    );
    assert_transport!(
        TsParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::TypeScript,
        "export function value(): number { return 1; }\n"
    );
    assert_transport!(
        PyParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::Python,
        "def value():\n    return 1\n"
    );
    assert_transport!(
        CParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::C,
        "int value(void) { return 1; }\n"
    );
    assert_transport!(
        CppParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::Cpp,
        "int value() { return 1; }\n"
    );
    assert_transport!(
        ZigParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::Zig,
        "pub fn value() i32 { return 1; }\n"
    );
    assert_transport!(
        GoParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::Go,
        "package sample\nfunc value() int { return 1 }\n"
    );
    assert_transport!(
        JavaParser::new(NoPrompts, NoPrompts, config.clone(), root.clone()),
        Language::Java,
        "class Sample { int value() { return 1; } }\n"
    );
    assert_transport!(
        ElixirParser::new(NoPrompts, NoPrompts, config, root),
        Language::Elixir,
        "defmodule Sample do\n  def value, do: 1\nend\n"
    );
}
