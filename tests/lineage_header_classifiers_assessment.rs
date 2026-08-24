use crystalline_lint::entities::layer::Layer;
use crystalline_lint::entities::parsed_file::PromptHeader;
use crystalline_lint::entities::rule_traits::{HasPromptFilesystem, HasPromptRefs};
use crystalline_lint::entities::violation::ViolationLevel;
use crystalline_lint::rules::{multi_prompt_header, prompt_header};
use std::path::Path;

struct HeaderFixture<'a> {
    header: Option<PromptHeader<'a>>,
    exists: bool,
    path: &'a Path,
}

impl<'a> HasPromptFilesystem<'a> for HeaderFixture<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.header.as_ref()
    }
    fn prompt_file_exists(&self) -> bool {
        self.exists
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

fn header(layer: Layer) -> PromptHeader<'static> {
    PromptHeader {
        prompt_path: "00_nucleo/prompts/causal.md",
        prompt_hash: Some("aaaaaaaa"),
        current_hash: Some("aaaaaaaa".to_string()),
        layer,
        updated: None,
    }
}

#[test]
fn v1_observable_layer_scope_exempts_non_production_headers() {
    for layer in [Layer::L0, Layer::Lab, Layer::Unknown] {
        let fixture = HeaderFixture {
            header: Some(header(layer.clone())),
            exists: false,
            path: Path::new("outside/value.rs"),
        };
        assert!(
            prompt_header::check(&fixture, &[]).is_empty(),
            "V1 applied to exempt layer {layer:?}"
        );
    }

    // SPEC-GAP(A1): HasPromptFilesystem has no layer() method. Once header is None,
    // public inputs cannot distinguish an L1 file from L0/Lab/Unknown.
}

#[test]
fn v1_header_exists_truth_table_is_complete_and_emits_at_most_once() {
    for layer in [Layer::L1, Layer::L2, Layer::L3, Layer::L4] {
        for has_header in [false, true] {
            for exists in [false, true] {
                let fixture = HeaderFixture {
                    header: has_header.then(|| header(layer.clone())),
                    exists,
                    path: Path::new("module/value.rs"),
                };
                let violations = prompt_header::check(&fixture, &[]);
                assert_eq!(
                    violations.len(),
                    usize::from(!has_header || !exists),
                    "V1 table mismatch: {layer:?}, header={has_header}, exists={exists}"
                );
                if let Some(v) = violations.first() {
                    assert_eq!(v.rule_id, "V1");
                    assert_eq!(v.level, ViolationLevel::Error);
                }
            }
        }
    }
}

#[test]
fn v1_strict_directories_match_path_components_not_textual_prefixes() {
    let strict = vec!["01_core/contracts".to_string(), "área/核".to_string()];
    let cases = [
        ("01_core/contracts", ViolationLevel::Fatal),
        ("01_core/contracts/nested/file.rs", ViolationLevel::Fatal),
        ("área/核/file.rs", ViolationLevel::Fatal),
        ("01_core/contracts-ish/file.rs", ViolationLevel::Error),
        ("prefix/01_core/contracts/file.rs", ViolationLevel::Error),
        ("área/核心/file.rs", ViolationLevel::Error),
    ];
    for (path, expected_level) in cases {
        let fixture = HeaderFixture {
            header: None,
            exists: false,
            path: Path::new(path),
        };
        let violations = prompt_header::check(&fixture, &strict);
        assert_eq!(violations.len(), 1, "V1 missing at {path}");
        assert_eq!(
            violations[0].level, expected_level,
            "strict match at {path}"
        );
        assert_eq!(violations[0].location.path.as_ref(), Path::new(path));
        assert_eq!(
            (violations[0].location.line, violations[0].location.column),
            (1, 0)
        );
    }
}

struct RefsFixture<'a> {
    layer: Layer,
    refs: Vec<&'a str>,
    path: &'a Path,
}

impl<'a> HasPromptRefs<'a> for RefsFixture<'a> {
    fn layer(&self) -> &Layer {
        &self.layer
    }
    fn prompt_refs(&self) -> &[&'a str] {
        &self.refs
    }
    fn path(&self) -> &'a Path {
        self.path
    }
}

#[test]
fn v15_layer_and_cardinality_truth_table_is_exact() {
    for layer in [
        Layer::L0,
        Layer::L1,
        Layer::L2,
        Layer::L3,
        Layer::L4,
        Layer::Lab,
        Layer::Unknown,
    ] {
        for count in 0..=4 {
            let fixture = RefsFixture {
                layer: layer.clone(),
                refs: vec!["prompt.md"; count],
                path: Path::new("module.rs"),
            };
            let violations = multi_prompt_header::check(&fixture);
            let production = matches!(layer, Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4);
            assert_eq!(
                violations.len(),
                usize::from(production && count >= 2),
                "V15 mismatch for {layer:?}, count={count}"
            );
            if let Some(v) = violations.first() {
                assert_eq!(v.rule_id, "V15");
                assert_eq!(v.level, ViolationLevel::Error);
            }
        }
    }
}

#[test]
fn v15_preserves_count_order_duplicates_unicode_path_position_and_evidence() {
    let refs = vec![
        "prompts/á.md",
        "prompts/á.md",
        "prompts/a.md",
        "prompts/核.md",
    ];
    let path = Path::new("01_core/linhagem/Δ.rs");
    let violations = multi_prompt_header::check(&RefsFixture {
        layer: Layer::L1,
        refs: refs.clone(),
        path,
    });
    assert_eq!(violations.len(), 1);
    let violation = &violations[0];
    assert_eq!(violation.location.path.as_ref(), path);
    assert_eq!((violation.location.line, violation.location.column), (1, 0));
    assert!(violation.message.contains('4'), "message omitted ref count");
    let mut cursor = 0;
    for reference in refs {
        let offset = violation.message[cursor..]
            .find(reference)
            .unwrap_or_else(|| {
                panic!(
                    "message omitted ordered ref {reference:?}: {}",
                    violation.message
                )
            });
        cursor += offset + reference.len();
    }
}

#[test]
fn v1_and_v15_are_deterministic_and_preserve_distinct_unicode_representations() {
    let refs = vec!["prompts/Á.md", "prompts/A\u{301}.md"];
    let fixture = RefsFixture {
        layer: Layer::L4,
        refs,
        path: Path::new("04_wiring/Á.rs"),
    };
    let first = multi_prompt_header::check(&fixture);
    let second = multi_prompt_header::check(&fixture);
    assert_eq!(first, second);
    assert!(first[0].message.contains("prompts/Á.md"));
    assert!(first[0].message.contains("prompts/A\u{301}.md"));

    let missing = HeaderFixture {
        header: Some(header(Layer::L4)),
        exists: false,
        path: Path::new("04_wiring/Á.rs"),
    };
    let a = prompt_header::check(&missing, &[]);
    let b = prompt_header::check(&missing, &[]);
    assert_eq!(a, b);
    assert_eq!(a[0].location.path.as_ref(), missing.path);
    assert!(a[0].message.contains("00_nucleo/prompts/causal.md"));
}
