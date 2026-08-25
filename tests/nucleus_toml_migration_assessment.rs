use crystalline_lint::infra::nucleus::{
    audit_project, effective_nucleus_hash, parse_nucleus, parse_prompt_nucleus_refs, HashDependency,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;

const DOCUMENT: &[u8] = br#"tekt = 1
kind = "nucleus"
id = "path"
title = "Path identity"

[[claims]]
id = "identity"
level = "must"
statement = "Paths preserve identity."
"#;

fn prompt(path: &str) -> Vec<u8> {
    format!(
        "# Prompt\nHash do Código: 00000000\nNúcleos Tekt:\n- {path} sha256:{}\n\n## Body\nx\n",
        "0".repeat(64)
    )
    .into_bytes()
}

#[test]
fn b1_only_canonical_toml_paths_are_valid_nucleus_references() {
    let canonical = "00_nucleo/prompts/_nuclei/path.toml";
    assert_eq!(
        parse_prompt_nucleus_refs(&prompt(canonical)).unwrap()[0].path,
        canonical
    );
    for invalid in [
        "00_nucleo/prompts/_nuclei/path.tekt",
        "00_nucleo/prompts/_nuclei/path.tekt.toml",
        "00_nucleo/prompts/_nuclei/path.md",
        "00_nucleo/prompts/_nuclei/path",
        "00_nucleo/prompts/path.toml",
    ] {
        assert!(
            parse_prompt_nucleus_refs(&prompt(invalid)).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn b1_namespace_requires_schema_and_legacy_extension_is_observable() {
    let root = tempfile::tempdir().unwrap();
    let nuclei = root.path().join("00_nucleo/prompts/_nuclei");
    fs::create_dir_all(&nuclei).unwrap();
    fs::write(nuclei.join("generic.toml"), b"name = 'not-a-nucleus'\n").unwrap();
    fs::write(nuclei.join("legacy.tekt"), DOCUMENT).unwrap();

    let audit = audit_project(root.path());
    assert!(audit
        .issues
        .iter()
        .any(|(path, _)| path.ends_with("generic.toml")));
    assert!(audit
        .issues
        .iter()
        .any(|(path, message)| { path.ends_with("legacy.tekt") && message.contains("legacy") }));
    assert!(audit.entries.is_empty());
}

#[test]
fn b2_dependencies_use_toml_identity_and_path_changes_effective_digest_only() {
    let dependency = format!(
        "{}\n[[depends]]\npath='00_nucleo/prompts/_nuclei/base.toml'\nsha256='{}'\n",
        std::str::from_utf8(DOCUMENT).unwrap(),
        "a".repeat(64)
    );
    let parsed = parse_nucleus(dependency.as_bytes()).unwrap();
    assert_eq!(
        parsed.depends[0].path,
        "00_nucleo/prompts/_nuclei/base.toml"
    );

    let raw_before = Sha256::digest(DOCUMENT);
    let raw_after = Sha256::digest(DOCUMENT);
    assert_eq!(raw_before, raw_after);
    let old = HashDependency {
        path: "00_nucleo/prompts/_nuclei/path.tekt".into(),
        digest: raw_before.into(),
    };
    let new = HashDependency {
        path: "00_nucleo/prompts/_nuclei/path.toml".into(),
        digest: raw_after.into(),
    };
    assert_ne!(
        effective_nucleus_hash(b"consumer", &[old]),
        effective_nucleus_hash(b"consumer", &[new])
    );
}

#[test]
fn b3_production_code_cannot_own_a_toml_nucleus() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("00_nucleo/prompts/_nuclei")).unwrap();
    fs::create_dir_all(root.path().join("01_core")).unwrap();
    fs::write(
        root.path().join("crystalline.toml"),
        "[project]\nroot='.'\n[layers]\nL0='00_nucleo'\nL1='01_core'\n",
    )
    .unwrap();
    fs::write(
        root.path().join("00_nucleo/prompts/_nuclei/path.toml"),
        DOCUMENT,
    )
    .unwrap();
    fs::write(
        root.path().join("01_core/x.rs"),
        "//! Crystalline Lineage\n//! @prompt 00_nucleo/prompts/_nuclei/path.toml\n//! @prompt-hash 00000000\n//! @layer L1\nfn x() {}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(root.path())
        .args([".", "--checks", "v1,v26", "--format", "sarif"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "{stdout}");
    assert!(stdout.contains("V26"), "{stdout}");
    assert!(stdout.contains("cannot own"), "{stdout}");
}

#[test]
fn b4_prompt_without_nucleus_and_transaction_gates_remain_independent() {
    let prompt = b"# Prompt\nHash do C\xC3\xB3digo: deadbeef\nbody\n";
    let expected = hex::encode(Sha256::digest(b"# Prompt\nbody\n"))[..8].to_owned();
    assert_eq!(
        crystalline_lint::infra::nucleus::effective_prompt_hash(prompt, &[]).unwrap(),
        expected
    );
}
