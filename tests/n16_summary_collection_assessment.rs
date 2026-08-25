use crystalline_lint::contracts::file_provider::SourceFile;
use crystalline_lint::entities::layer::{Language, Layer};
use crystalline_lint::shell::n16_summary::{
    collect_n16_stats, extract_n16_module_name, extract_n16_tag, N16ModuleStats, N16Stats, N16Tag,
};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

fn source(path: &str, content: &str) -> SourceFile {
    SourceFile {
        path: PathBuf::from(path),
        content: content.to_owned(),
        language: Language::Rust,
        layer: Layer::L1,
        has_adjacent_test: false,
    }
}

fn module(alpha: usize, beta: usize, gamma: usize) -> N16ModuleStats {
    N16ModuleStats { alpha, beta, gamma }
}

fn stats(entries: &[(&str, N16ModuleStats)]) -> N16Stats {
    entries
        .iter()
        .map(|(name, counts)| ((*name).to_owned(), counts.clone()))
        .collect::<BTreeMap<_, _>>()
}

#[test]
fn token_grammar_requires_exactly_one_valid_non_overlapping_token() {
    for (text, expected) in [
        ("N16[α]", Some(N16Tag::Alpha)),
        ("prefixN16[β]suffix", Some(N16Tag::Beta)),
        ("N16[γ] trailing garbage", Some(N16Tag::Gamma)),
        ("N16[A]", Some(N16Tag::Alpha)),
        ("N16[a]", Some(N16Tag::Alpha)),
        ("N16[B]", Some(N16Tag::Beta)),
        ("N16[b]", Some(N16Tag::Beta)),
        ("N16[C]", Some(N16Tag::Gamma)),
        ("N16[c]", Some(N16Tag::Gamma)),
        ("", None),
        ("n16[A]", None),
        ("N16[]", None),
        ("N16[δ]", None),
        ("N16[AA]", None),
        ("N16[Α]", None),
        ("N１6[A]", None),
        ("N16[A", None),
        ("N16[A]]", Some(N16Tag::Alpha)),
        ("N16[A] N16[A]", None),
        ("N16[A] decoy N16[B]", None),
        ("N16[X] and N16[C]", Some(N16Tag::Gamma)),
    ] {
        assert_eq!(extract_n16_tag(text), expected, "input: {text:?}");
    }
}

#[test]
fn grouping_is_component_based_and_does_not_normalize_identity() {
    for (path, expected) in [
        ("01_core/src/entities/item.rs", "entities/"),
        ("root/01_core/src/math/layout/grid.rs", "math/layout/"),
        ("root\\01_core\\src\\math\\layout\\grid.rs", "math/layout/"),
        ("01_core//./src//parse/file.rs", "parse/"),
        ("01_core/src/../item.rs", "../"),
        ("01_core/src/file.rs", "01_core/"),
        ("01_core/src", "01_core/"),
        ("01_core/root.rs", "01_core/"),
        ("00_nucleo/notes.md", "00_nucleo/"),
        ("02_shell/src/main.rs", "02_shell/"),
        ("03_infra/src/io.rs", "03_infra/"),
        ("04_wiring/main.rs", "04_wiring/"),
        ("prefix/01_core_tools/src/fake.rs", "other/"),
        ("prefix/01_Core/src/fake.rs", "other/"),
        ("vendor/compiler/file.rs", "other/"),
    ] {
        assert_eq!(
            extract_n16_module_name(Path::new(path)),
            expected,
            "path: {path:?}"
        );
    }
}

#[test]
fn collection_parses_last_colon_preserves_nominal_locations_and_source_wins() {
    let sources = vec![
        source("C:/repo/01_core/src/entities/a.rs", "N16[A]\nN16[B]"),
        source("01_core/src/parse/p.rs", "plain\nN16[C] N16[C]"),
        source("01_core/src/math/layout/m.rs", "N16[γ]"),
    ];
    let exceptions = HashMap::from([
        // Exact duplicates do not count twice; the source classification wins conflicts.
        (
            "C:/repo/01_core/src/entities/a.rs:1".to_owned(),
            "N16[C]".to_owned(),
        ),
        (
            "C:/repo/01_core/src/entities/a.rs:2".to_owned(),
            "N16[B]".to_owned(),
        ),
        // A colon belongs to the path because only the final colon delimits the line.
        (
            "C:/repo/01_core/src/entities/a.rs:3".to_owned(),
            "N16[C]".to_owned(),
        ),
        // Nominally distinct spelling remains distinct even though grouping agrees.
        (
            "C:\\repo\\01_core\\src\\entities\\a.rs:1".to_owned(),
            "N16[A]".to_owned(),
        ),
        // Zero and empty path are valid nominal exception locations.
        ("01_core/src/parse/p.rs:0".to_owned(), "N16[B]".to_owned()),
        (":0".to_owned(), "N16[A]".to_owned()),
        // Malformed keys and non-classifying values are ignored.
        (
            "01_core/src/parse/no-line.rs".to_owned(),
            "N16[A]".to_owned(),
        ),
        (
            "01_core/src/parse/bad.rs:not-a-line".to_owned(),
            "N16[A]".to_owned(),
        ),
        (
            "01_core/src/parse/overflow.rs:999999999999999999999999999999999999999".to_owned(),
            "N16[A]".to_owned(),
        ),
        (
            "01_core/src/parse/multi.rs:7:tail".to_owned(),
            "N16[A]".to_owned(),
        ),
        (
            "01_core/src/parse/ambiguous.rs:8".to_owned(),
            "N16[A] N16[B]".to_owned(),
        ),
    ]);

    assert_eq!(
        collect_n16_stats(&sources, &exceptions),
        stats(&[
            ("entities/", module(2, 1, 1)),
            ("math/layout/", module(0, 0, 1)),
            ("other/", module(1, 0, 0)),
            ("parse/", module(0, 1, 0)),
        ])
    );
}

#[test]
fn every_classified_occurrence_counts_once_and_input_orders_are_unobservable() {
    let original_sources = || {
        vec![
            source("01_core/src/zeta/a.rs", "N16[A]\nN16[B]\nN16[C]"),
            source("03_infra/b.rs", "N16[C]\nnone\nN16[A]"),
            source("outside/c.rs", "N16[B]"),
        ]
    };
    let mut reversed_sources = original_sources();
    reversed_sources.reverse();

    let exception_pairs = [
        ("01_core/src/zeta/a.rs:1", "N16[C]"),
        ("01_core/src/zeta/extra.rs:9", "N16[A]"),
        ("03_infra/b.rs:2", "N16[B]"),
        ("outside/c.rs:1", "N16[B]"),
    ];
    let forward = exception_pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect::<HashMap<_, _>>();
    let reverse = exception_pairs
        .iter()
        .rev()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect::<HashMap<_, _>>();

    let expected = stats(&[
        ("03_infra/", module(1, 1, 1)),
        ("other/", module(0, 1, 0)),
        ("zeta/", module(2, 1, 1)),
    ]);

    assert_eq!(collect_n16_stats(&original_sources(), &forward), expected);
    assert_eq!(collect_n16_stats(&original_sources(), &reverse), expected);
    assert_eq!(collect_n16_stats(&reversed_sources, &forward), expected);
    assert_eq!(collect_n16_stats(&reversed_sources, &reverse), expected);
}
