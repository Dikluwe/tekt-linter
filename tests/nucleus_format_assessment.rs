use crystalline_lint::infra::nucleus::{parse_nucleus, NucleusLevel};

const MINIMAL: &[u8] = br#"tekt = 1
kind = "nucleus"
id = "path"
title = "Logical paths"

[[claims]]
id = "identity"
level = "must"
statement = "Paths preserve logical identity."
"#;

#[test]
fn strict_minimum_parses() {
    let doc = parse_nucleus(MINIMAL).unwrap();
    assert_eq!(doc.id, "path");
    assert_eq!(doc.claims[0].level, NucleusLevel::Must);
}

#[test]
fn missing_unknown_version_kind_and_unknown_fields_fail_closed() {
    for bytes in [
        b"kind='nucleus'\nid='x'\ntitle='x'\nclaims=[]".as_slice(),
        b"tekt=2\nkind='nucleus'\nid='x'\ntitle='x'\nclaims=[]".as_slice(),
        b"tekt=1\nkind='prompt'\nid='x'\ntitle='x'\nclaims=[]".as_slice(),
        b"tekt=1\nkind='nucleus'\nid='x'\ntitle='x'\nclaims=[]\nsurprise=true".as_slice(),
    ] {
        assert!(parse_nucleus(bytes).is_err());
    }
}

#[test]
fn identity_claim_and_text_limits_are_enforced() {
    let invalid = ["", "A", "a_", &format!("a{}", "x".repeat(64))];
    for id in invalid {
        let text = String::from_utf8(MINIMAL.to_vec())
            .unwrap()
            .replace("id = \"path\"", &format!("id = \"{id}\""));
        assert!(parse_nucleus(text.as_bytes()).is_err(), "id={id:?}");
    }
    let duplicate = String::from_utf8(MINIMAL.to_vec()).unwrap()
        + "\n[[claims]]\nid='identity'\nlevel='may'\nstatement='x'\n";
    assert!(parse_nucleus(duplicate.as_bytes()).is_err());
}

#[test]
fn malformed_utf8_nul_and_oversize_fail() {
    assert!(parse_nucleus(&[0xff]).is_err());
    let mut nul = MINIMAL.to_vec();
    nul.push(0);
    assert!(parse_nucleus(&nul).is_err());
    assert!(parse_nucleus(&vec![b'x'; 1024 * 1024 + 1]).is_err());
}
