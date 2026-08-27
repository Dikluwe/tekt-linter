//! Oráculo independente P0118 — conteúdo Python não decide o papel do artefato.

use crystalline_lint::contracts::file_provider::{SourceFile, SourceRole};
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::prompt_reader::{
    PromptHashResolution, PromptPresence, PromptReader,
};
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::py_parser::PyParser;
use crystalline_lint::rules::test_file;
use std::path::PathBuf;

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

fn parse(
    path: &str,
    content: &str,
    adjacent: bool,
) -> crystalline_lint::entities::parsed_file::ParsedFile<'static> {
    let parser = Box::leak(Box::new(PyParser::new(
        NoPrompts,
        NoPrompts,
        CrystallineConfig::default(),
        PathBuf::from("."),
    )));
    let source = Box::leak(Box::new(SourceFile {
        path: PathBuf::from(path),
        content: content.to_string(),
        language: Language::Python,
        layer: Layer::L1,
        has_adjacent_test: adjacent,
        source_role: crystalline_lint::infra::walker::classify_source_role(
            &Language::Python,
            std::path::Path::new(path),
        ),
    }));
    parser.parse(source).expect("valid synthetic Python")
}

#[test]
fn both_python_test_name_forms_ignore_body_style_for_v2() {
    let bodies = [
        "def test_value():\n    assert 2 + 2 == 4\n",
        "import pytest\ndef test_value():\n    with pytest.raises(ValueError):\n        raise ValueError()\n",
        "def helper(value):\n    return value + 1\n",
        "import pytest\n@pytest.fixture\ndef value():\n    return 4\n",
    ];

    for path in ["01_core/test_catalogo.py", "01_core/catalogo_test.py"] {
        for body in bodies {
            let parsed = parse(path, body, false);
            assert_eq!(parsed.source_role, SourceRole::Test, "path={path}");
            assert!(test_file::check(&parsed).is_empty(), "path={path}");
        }
    }
}

#[test]
fn pytest_call_in_production_is_not_coverage() {
    let parsed = parse(
        "01_core/catalogo.py",
        "import pytest\ndef validate():\n    with pytest.raises(ValueError):\n        raise ValueError()\n",
        false,
    );
    assert_eq!(parsed.source_role, SourceRole::Production);
    let findings = test_file::check(&parsed);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "V2");
}

#[test]
fn adjacent_test_covers_production_without_changing_its_role() {
    let parsed = parse(
        "01_core/catalogo.py",
        "def catalogo(value):\n    return value\n",
        true,
    );
    assert_eq!(parsed.source_role, SourceRole::Production);
    assert!(test_file::check(&parsed).is_empty());
}
