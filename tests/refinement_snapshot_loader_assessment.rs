use crystalline_lint::entities::refinement::ArtifactFacts;
use crystalline_lint::infra::refinement_snapshot::{load_snapshot, load_snapshot_from_bytes};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn valid(observables: &str) -> String {
    format!(
        r#"{{"format_version":1,"artifact_id":" artifact ","extractor_version":" extractor-v1 ","observables":{{{observables}}}}}"#
    )
}

fn load(text: &str) -> Result<ArtifactFacts, String> {
    load_snapshot_from_bytes(text.as_bytes(), "assessment://snapshot")
}

fn assert_prefix(result: Result<ArtifactFacts, String>, prefix: &str) {
    let error = result.expect_err("hostile snapshot must fail closed");
    assert!(
        error.starts_with(prefix),
        "expected error prefix {prefix:?}, got {error:?}"
    );
}

fn temporary_file() -> PathBuf {
    let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "crystalline-refinement-snapshot-assessment-{}-{serial}.json",
        std::process::id()
    ))
}

#[test]
fn accepts_all_closed_states_reasons_and_preserves_textual_identity() {
    let all = valid(
        r#"
        "empty":{"state":"known","value":""},
        "space":{"state":"known","value":"  "},
        "unicøde/雪":{"state":"known","value":"Olá\n雪\u0000"},
        "absent":{"state":"absent"},
        "u0":{"state":"unknown","reason":"missing-observable"},
        "u1":{"state":"unknown","reason":"ambiguous-identity"},
        "u2":{"state":"unknown","reason":"unsupported-parser"},
        "u3":{"state":"unknown","reason":"opaque-construction"},
        "u4":{"state":"unknown","reason":"partial-contract"},
        "u5":{"state":"unknown","reason":"budget-exhausted"}
        "#,
    );
    let facts = load(&all).expect("every normative state and reason must load");

    let trimmed_metadata = all
        .replace(" artifact ", "artifact")
        .replace(" extractor-v1 ", "extractor-v1");
    assert_ne!(
        facts,
        load(&trimmed_metadata).expect("trimmed metadata remains valid"),
        "metadata must be validated with trim but preserved verbatim"
    );

    let trimmed_key = all.replace("\"unicøde/雪\"", "\"unicøde/雪 \"");
    assert_ne!(
        facts,
        load(&trimmed_key).expect("nonempty whitespace-bearing key remains valid"),
        "observable keys must be preserved verbatim"
    );

    let changed_known = all.replace("\"value\":\"  \"", "\"value\":\" \"");
    assert_ne!(
        facts,
        load(&changed_known).expect("empty and whitespace Known values are valid"),
        "Known values must not be trimmed"
    );
}

#[test]
fn root_is_closed_versioned_and_rejects_duplicate_top_level_fields() {
    for json in [
        r#"{"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{},"extra":0}"#,
        r#"{"format_version":1,"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":1,"artifact_id":"a","artifact_id":"b","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{},"observables":{}}"#,
    ] {
        assert_prefix(load(json), "schema:");
    }

    for json in [
        r#"{"artifact_id":"a","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":-1,"artifact_id":"a","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":1.0,"artifact_id":"a","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":"1","artifact_id":"a","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":184467440737095516160,"artifact_id":"a","extractor_version":"e","observables":{}}"#,
    ] {
        let error = load(json).expect_err("invalid version representation must fail");
        assert!(
            error.starts_with("schema:") || error.starts_with("unsupported-version:"),
            "version shape is schema and supported integer value is version: {error:?}"
        );
    }
    assert_prefix(
        load(r#"{"format_version":2,"artifact_id":"a","extractor_version":"e","observables":{}}"#),
        "unsupported-version:",
    );
}

#[test]
fn metadata_and_observable_keys_require_nonempty_trim_without_coercion() {
    for json in [
        r#"{"format_version":1,"artifact_id":"","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":1,"artifact_id":" \n\t","extractor_version":"e","observables":{}}"#,
        r#"{"format_version":1,"artifact_id":"a","extractor_version":" ","observables":{}}"#,
        r#"{"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{" ":{"state":"absent"}}}"#,
    ] {
        assert_prefix(load(json), "schema:");
    }
}

#[test]
fn observable_objects_are_closed_and_duplicate_safe() {
    for body in [
        r#""x":{"state":"known","value":"v","extra":0}"#,
        r#""x":{"state":"known","state":"known","value":"v"}"#,
        r#""x":{"state":"known","value":"v","value":"w"}"#,
        r#""x":{"state":"unknown","reason":"partial-contract","reason":"budget-exhausted"}"#,
        r#""x":{"state":"known"}"#,
        r#""x":{"state":"known","value":1}"#,
        r#""x":{"state":"known","value":"v","reason":"partial-contract"}"#,
        r#""x":{"state":"absent","value":"v"}"#,
        r#""x":{"state":"unknown"}"#,
        r#""x":{"state":"unknown","reason":"other"}"#,
        r#""x":{"state":"mystery"}"#,
        r#""x":{"state":"absent"},"x":{"state":"absent"}"#,
    ] {
        assert_prefix(load(&valid(body)), "schema:");
    }
}

#[test]
fn property_order_does_not_change_the_loaded_value() {
    let a = r#"{"format_version":1,"artifact_id":"a","extractor_version":"e","observables":{"z":{"state":"unknown","reason":"partial-contract"},"a":{"state":"known","value":"雪"}}}"#;
    let b = r#"{"observables":{"a":{"value":"雪","state":"known"},"z":{"reason":"partial-contract","state":"unknown"}},"extractor_version":"e","artifact_id":"a","format_version":1}"#;
    assert_eq!(load(a).unwrap(), load(b).unwrap());
}

#[test]
fn errors_have_stable_classes_and_from_bytes_preserves_source_label() {
    assert_prefix(
        load_snapshot_from_bytes(&[0xff], "hostile;$(never-run)\nsource"),
        "invalid-utf8:",
    );
    let invalid_utf8 = load_snapshot_from_bytes(&[0xff], "hostile;$(never-run)\nsource")
        .expect_err("invalid UTF-8 must fail");
    assert!(invalid_utf8.contains("hostile;$(never-run)\nsource"));
    assert_prefix(load("{"), "json-syntax:");
    assert_prefix(load("[]"), "schema:");
}

#[test]
fn practical_byte_string_and_cardinality_limits_fail_before_exhaustion() {
    let oversized_bytes = vec![b' '; 4 * 1024 * 1024 + 1];
    assert_prefix(
        load_snapshot_from_bytes(&oversized_bytes, "assessment://oversized"),
        "limit:",
    );

    let long_value = "x".repeat(64 * 1024 + 1);
    assert_prefix(
        load(&valid(&format!(
            r#""x":{{"state":"known","value":"{long_value}"}}"#
        ))),
        "limit:",
    );

    let observables = (0..4097)
        .map(|i| format!(r#""k{i}":{{"state":"absent"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    assert_prefix(load(&valid(&observables)), "limit:");
}

#[test]
fn small_regular_file_matches_from_bytes_and_missing_path_is_io() {
    let bytes = valid(r#""k":{"state":"known","value":"v"}"#);
    let path = temporary_file();
    fs::write(&path, bytes.as_bytes()).expect("create confined regular fixture");
    let from_path = load_snapshot(&path).expect("small regular file must load");
    let from_bytes = load_snapshot_from_bytes(bytes.as_bytes(), path.to_str().unwrap())
        .expect("same bytes must load");
    fs::remove_file(&path).expect("remove confined fixture");
    assert_eq!(from_path, from_bytes);

    assert_prefix(load_snapshot(&path), "io:");
}

// Symlink traversal and concurrent replacement are intentionally absent: the normative
// outcome depends on filesystem timing/platform behavior, so this blind gate keeps only
// deterministic from-bytes and stable regular-file coverage.
