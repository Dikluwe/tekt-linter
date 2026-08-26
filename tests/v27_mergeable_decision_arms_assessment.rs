use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::entities::parsed_file::PublicInterface;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::crate_registry::CrateRegistry;
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::rules::mergeable_decision_arms;

#[derive(Clone)]
struct NoPrompts;
impl PromptReader for NoPrompts {
    fn exists(&self, _path: &str) -> bool {
        false
    }
    fn read_hash(&self, _path: &str) -> Option<String> {
        None
    }
}

#[derive(Clone)]
struct NoSnapshots;
impl PromptSnapshotReader for NoSnapshots {
    fn read_snapshot(&self, _path: &str) -> Option<PublicInterface<'static>> {
        None
    }
    fn serialize_snapshot(&self, _interface: &PublicInterface<'_>) -> String {
        String::new()
    }
}

fn lint(source: &str) -> Vec<(usize, String)> {
    let file = SourceFile {
        path: "01_core/v27.rs".into(),
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
    let parsed = parser.parse(&file).expect("fixture Rust deve ser válida");
    mergeable_decision_arms::check(&parsed)
        .into_iter()
        .map(|violation| (violation.location.line, violation.message))
        .collect()
}

#[test]
fn parser_and_rule_accept_plain_guard_and_existing_or_pattern() {
    let plain = lint("fn f(k: K) { match k { K::A => hit(), K::B => hit(), _ => miss() } }");
    assert_eq!(plain.len(), 1, "{plain:#?}");
    assert!(plain[0].1.contains("K::A | K::B"));

    let guarded = lint(
        "fn f(k: K) { match k { K::A if ok() => hit(), K::B if ok() => hit(), _ => miss() } }",
    );
    assert_eq!(guarded.len(), 1, "{guarded:#?}");

    let existing =
        lint("fn f(k: K) { match k { K::A | K::B => hit(), K::C => hit(), _ => miss() } }");
    assert_eq!(existing.len(), 1, "{existing:#?}");
    assert!(existing[0].1.contains("K::A | K::B | K::C"));
}

#[test]
fn parser_preserves_operators_arguments_control_tokens_and_binding_modes() {
    for source in [
        "fn f(k: K) { match k { K::A => x == y, K::B => x != y, _ => false } }",
        "fn f(k: K) { match k { K::A => call(x, y), K::B => call(y, x), _ => z } }",
        "fn f(k: K) { match k { K::A(x) => use_it(x), K::B(ref x) => use_it(x), _ => z() } }",
        "fn f(k: K) { match k { K::A(x) => use_it(x), K::B(x) => use_it(x), _ => z() } }",
        "fn f(k: K) { match k { K::A { pos, .. } => use_it(pos), K::B { pos, .. } => use_it(pos), _ => z() } }",
        "fn f(k: K) { match k { K::A(x) if left(x) => use_it(x), K::B(x) if right(x) => use_it(x), _ => z() } }",
        "fn f(k: K) { match k { K::A => return left(), K::B => left(), _ => z() } }",
    ] {
        assert!(lint(source).is_empty(), "falso positivo: {source}");
    }
}

#[test]
fn does_not_bridge_arms_or_accept_macros_placeholders_empty_ranges_and_catchalls() {
    for source in [
        "fn f(k: K) { match k { K::A => hit(), K::Middle => other(), K::B => hit() } }",
        "fn f(k: K) { match k { K::A => emit!(), K::B => emit!(), _ => miss() } }",
        "fn f(k: K) { match k { K::A => todo!(), K::B => todo!(), _ => miss() } }",
        "fn f(k: K) { match k { K::A => {}, K::B => {}, _ => miss() } }",
        "fn f(k: K) { match k { #[cfg(feature = \"a\")] K::A => hit(), K::B => hit(), _ => miss() } }",
        "fn f(k: u8) { match k { 0..=2 => hit(), 3..=5 => hit(), _ => miss() } }",
        "fn f(k: K) { match k { K::A => hit(), _ => hit() } }",
    ] {
        assert!(lint(source).is_empty(), "falso positivo: {source}");
    }
}

#[test]
fn comments_and_formatting_are_inert_but_grouping_is_maximal_and_deterministic() {
    let source = r#"
fn f(k: K) {
    match k {
        K::A => { /* first */ hit() },
        K::B => {
            // second
            hit()
        },
        K::C => { hit() },
        _ => miss(),
    }
}
"#;
    let first = lint(source);
    let second = lint(source);
    assert_eq!(first, second);
    assert_eq!(first.len(), 1, "{first:#?}");
    assert!(first[0].1.contains("K::A | K::B | K::C"));
}
