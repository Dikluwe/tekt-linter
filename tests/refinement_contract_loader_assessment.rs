use crystalline_lint::infra::refinement_snapshot::{load_contract, load_contract_from_bytes};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const VALID: &str = r#"
id = "contract-v1"

[[relation]]
kind = "preserve"
source = "source.a"
target = "target.a"

[[relation]]
kind = "may-normalize"
source = "source.b"
target = "target.b"
accepted_targets = ["normalized-b", "canonical-b"]

[[relation]]
kind = "must-not-invent"
target = "target.c"
"#;

fn load_ok(input: &str) -> String {
    format!(
        "{:?}",
        load_contract_from_bytes(input.as_bytes(), "assessment-memory")
            .unwrap_or_else(|error| panic!("expected contract to load, got {error}"))
    )
}

fn assert_error(input: &[u8], prefix: &str) -> String {
    let error = load_contract_from_bytes(input, "assessment-memory")
        .expect_err("invalid contract must fail closed");
    assert!(
        error.starts_with(prefix),
        "expected {prefix:?} error, got {error:?}"
    );
    error
}

fn unique_temp_file(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "crystalline-p0095-b2-{}-{nonce}-{name}",
        std::process::id()
    ))
}

#[test]
fn accepts_the_three_closed_variants_and_preserves_text_and_order() {
    let input = r#"
id = "  contract identity  "

[[relation]]
kind = "preserve"
source = "  source first  "
target = "  target first  "

[[relation]]
kind = "may-normalize"
source = "source second"
target = "target second"
accepted_targets = ["  accepted one  ", "accepted two", "accepted three"]

[[relation]]
kind = "must-not-invent"
target = "target third"
"#;
    let debug = load_ok(input);

    for preserved in [
        "  contract identity  ",
        "  source first  ",
        "  target first  ",
        "  accepted one  ",
    ] {
        assert!(
            debug.contains(preserved),
            "trimmed or lost {preserved:?}: {debug}"
        );
    }

    let preserve = debug.find("Preserve").expect("preserve relation in value");
    let normalize = debug
        .find("MayNormalize")
        .expect("may-normalize relation in value");
    let prohibit = debug
        .find("MustNotInvent")
        .expect("must-not-invent relation in value");
    assert!(
        preserve < normalize && normalize < prohibit,
        "relation order changed: {debug}"
    );

    let one = debug.find("  accepted one  ").unwrap();
    let two = debug.find("accepted two").unwrap();
    let three = debug.find("accepted three").unwrap();
    assert!(
        one < two && two < three,
        "accepted target order changed: {debug}"
    );
}

#[test]
fn rejects_empty_document_id_and_relation_list() {
    for input in [
        "id = \"contract\"\n",
        "id = \"   \"\n[[relation]]\nkind = \"must-not-invent\"\ntarget = \"x\"\n",
        "[[relation]]\nkind = \"must-not-invent\"\ntarget = \"x\"\n",
    ] {
        assert_error(input.as_bytes(), "schema:");
    }
}

#[test]
fn enforces_exact_case_sensitive_kinds_and_conditional_fields() {
    let invalid = [
        // Unknown or differently cased kinds.
        "id='c'\n[[relation]]\nkind='Preserve'\nsource='s'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='may_normalize'\nsource='s'\ntarget='t'\naccepted_targets=['x']",
        "id='c'\n[[relation]]\nkind='must-exist'\ntarget='t'",
        // preserve requires source and target and permits no accepted list.
        "id='c'\n[[relation]]\nkind='preserve'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'",
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='t'\naccepted_targets=['x']",
        // may-normalize requires all three conditional fields.
        "id='c'\n[[relation]]\nkind='may-normalize'\ntarget='t'\naccepted_targets=['x']",
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\naccepted_targets=['x']",
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'",
        // must-not-invent permits only target.
        "id='c'\n[[relation]]\nkind='must-not-invent'",
        "id='c'\n[[relation]]\nkind='must-not-invent'\nsource='s'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\naccepted_targets=['x']",
    ];

    for input in invalid {
        assert_error(input.as_bytes(), "schema:");
    }
}

#[test]
fn requires_nonempty_preserved_relation_strings() {
    for input in [
        "id='c'\n[[relation]]\nkind='preserve'\nsource='   '\ntarget='t'",
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='\t'",
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['ok','  ']",
    ] {
        assert_error(input.as_bytes(), "schema:");
    }
}

#[test]
fn accepted_targets_must_be_nonempty_and_duplicate_free() {
    for input in [
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=[]",
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['same','same']",
    ] {
        assert_error(input.as_bytes(), "schema:");
    }

    // Textual equality across roles is explicitly permitted.
    load_ok(
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='same'\ntarget='same'\naccepted_targets=['same']",
    );
}

#[test]
fn rejects_structural_duplicates_and_conflicts() {
    let invalid = [
        // Exact structural duplicate.
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='t'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='t'",
        // preserve and may-normalize conflict for the same pair.
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='t'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['x']",
        // Two may-normalize relations conflict for the same pair, even with different lists.
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['x']\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['y']",
        // must-not-invent conflicts with every other relation sharing its target.
        "id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\n[[relation]]\nkind='preserve'\nsource='s'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='may-normalize'\nsource='s'\ntarget='t'\naccepted_targets=['x']\n[[relation]]\nkind='must-not-invent'\ntarget='t'",
    ];

    for input in invalid {
        assert_error(input.as_bytes(), "schema:");
    }
}

#[test]
fn rejects_duplicate_unknown_keys_and_unknown_tables() {
    let invalid = [
        "id='c'\nid='again'\n[[relation]]\nkind='must-not-invent'\ntarget='t'",
        "id='c'\nextra=true\n[[relation]]\nkind='must-not-invent'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='preserve'\nkind='preserve'\nsource='s'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='preserve'\nsource='s'\nsource='again'\ntarget='t'",
        "id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\nunknown='closed'",
        "id='c'\n[[relation]]\nkind='must-not-invent'\ntarget='t'\n[mystery]\nvalue=1",
    ];

    for input in invalid {
        assert_error(input.as_bytes(), "schema:");
    }
}

#[test]
fn distinguishes_utf8_syntax_schema_and_limit_error_classes() {
    assert_error(&[0xff, 0xfe], "invalid-utf8:");
    assert_error(b"id='c'\n[[relation]\nkind='preserve'", "toml-syntax:");
    assert_error(
        b"id=7\n[[relation]]\nkind='must-not-invent'\ntarget='t'",
        "schema:",
    );

    let oversized = vec![b' '; 4 * 1024 * 1024 + 1];
    assert_error(&oversized, "limit:");
}

#[test]
fn from_bytes_treats_hostile_source_as_an_opaque_preserved_label() {
    let marker = unique_temp_file("must-not-exist");
    let source = format!(
        "$(touch {}) ; ../../not-a-source | `false`",
        marker.display()
    );
    let error = load_contract_from_bytes(&[0xff], &source).expect_err("invalid UTF-8");

    assert!(error.starts_with("invalid-utf8:"), "wrong class: {error}");
    assert!(
        error.contains(&source),
        "source label was not preserved: {error}"
    );
    assert!(
        !marker.exists(),
        "hostile source label was interpreted or executed"
    );
}

#[test]
fn regular_file_and_from_bytes_are_equivalent_for_value_and_error_class() {
    let valid_path = unique_temp_file("valid.toml");
    fs::write(&valid_path, VALID).expect("write regular fixture");

    let from_bytes =
        load_contract_from_bytes(VALID.as_bytes(), "bytes-valid").expect("valid bytes must load");
    let from_file = load_contract(&valid_path).expect("same regular file must load");
    assert_eq!(format!("{from_bytes:?}"), format!("{from_file:?}"));

    let invalid = "id=7\n[[relation]]\nkind='must-not-invent'\ntarget='t'";
    let invalid_path = unique_temp_file("invalid.toml");
    fs::write(&invalid_path, invalid).expect("write invalid regular fixture");
    let bytes_error = load_contract_from_bytes(invalid.as_bytes(), "bytes-invalid")
        .expect_err("invalid bytes must fail");
    let file_error = load_contract(&invalid_path).expect_err("same regular file must fail");
    assert_eq!(
        bytes_error.split_once(':').map(|(prefix, _)| prefix),
        file_error.split_once(':').map(|(prefix, _)| prefix)
    );

    fs::remove_file(valid_path).expect("remove valid fixture");
    fs::remove_file(invalid_path).expect("remove invalid fixture");
}
