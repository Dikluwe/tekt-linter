//! Gate B1 do P0104: propriedade biunívoca de prompts, inteiramente in-memory.
//!
//! Oráculo: P0104 e contrato V15 fixado em
//! `81ba0f080eac8c2db78f27f04f206ff746eecdd358fdb55b146523192704f053`.
//! A API abaixo é deliberadamente a seam normativa mínima esperada em L1.

use std::path::PathBuf;

use crystalline_lint::{check_prompt_ownership, PromptOwnership, PromptOwnershipLayer};

fn ownership(path: &str, layer: PromptOwnershipLayer, prompts: &[&str]) -> PromptOwnership {
    PromptOwnership {
        code_path: PathBuf::from(path),
        layer,
        prompt_refs: prompts.iter().map(|prompt| (*prompt).to_owned()).collect(),
    }
}

fn productive(path: &str, prompt: &str) -> PromptOwnership {
    ownership(path, PromptOwnershipLayer::L1, &[prompt])
}

fn bytes(entries: &[PromptOwnership]) -> Vec<u8> {
    let violations = check_prompt_ownership(entries);
    violations
        .iter()
        .map(|violation| {
            format!(
                "{}|{:?}|{}|{}:{}:{}\n",
                violation.rule_id,
                violation.level,
                violation.message,
                violation.location.path.display(),
                violation.location.line,
                violation.location.column,
            )
        })
        .collect::<String>()
        .into_bytes()
}

#[test]
fn empty_and_one_to_one_pairs_are_bijective() {
    assert!(check_prompt_ownership(&[]).is_empty());

    let pairs = vec![
        productive("01_core/a.rs", "00_nucleo/prompts/a.md"),
        productive("02_shell/b.rs", "00_nucleo/prompts/b.md"),
    ];
    assert!(check_prompt_ownership(&pairs).is_empty());
}

#[test]
fn one_code_with_two_prompts_preserves_local_v15() {
    let entries = vec![ownership(
        "01_core/a.rs",
        PromptOwnershipLayer::L1,
        &["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"],
    )];

    let violations = check_prompt_ownership(&entries);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule_id, "V15");
    assert_eq!(format!("{:?}", violations[0].level), "Error");
    assert_eq!(violations[0].location.line, 1);
    assert_eq!(violations[0].location.column, 0);
    assert!(violations[0].message.contains("00_nucleo/prompts/a.md"));
    assert!(violations[0].message.contains("00_nucleo/prompts/b.md"));
}

#[test]
fn two_three_and_many_codes_yield_one_global_v15_per_prompt() {
    for cardinality in [2_usize, 3, 17] {
        let entries = (0..cardinality)
            .map(|index| {
                productive(
                    &format!("01_core/consumer-{index:02}.rs"),
                    "00_nucleo/prompts/shared.md",
                )
            })
            .collect::<Vec<_>>();

        let violations = check_prompt_ownership(&entries);
        assert_eq!(violations.len(), 1, "cardinality {cardinality}");
        let violation = &violations[0];
        assert_eq!(violation.rule_id, "V15");
        assert_eq!(format!("{:?}", violation.level), "Error");
        assert_eq!(
            violation.location.path,
            PathBuf::from("01_core/consumer-00.rs")
        );
        assert_eq!(violation.location.line, 1);
        assert_eq!(violation.location.column, 0);
        assert!(violation.message.contains("00_nucleo/prompts/shared.md"));
        assert!(violation.message.contains(&cardinality.to_string()));
        for index in 0..cardinality {
            assert!(violation
                .message
                .contains(&format!("01_core/consumer-{index:02}.rs")));
        }
    }
}

#[test]
fn duplicate_identical_pair_does_not_increase_cardinality() {
    let pair = productive("01_core/a.rs", "00_nucleo/prompts/a.md");
    let entries = vec![pair.clone(), pair.clone(), pair];
    assert!(check_prompt_ownership(&entries).is_empty());
}

#[test]
fn textually_near_prompt_identities_remain_distinct_and_case_sensitive() {
    let entries = vec![
        productive("01_core/a.rs", "00_nucleo/prompts/a.md"),
        productive("01_core/b.rs", "00_nucleo/prompts/A.md"),
        productive("01_core/c.rs", "00_nucleo/prompts/a-.md"),
        productive("01_core/d.rs", "00_nucleo/prompts/a.md "),
    ];
    assert!(check_prompt_ownership(&entries).is_empty());
}

#[test]
fn every_input_permutation_produces_identical_diagnostic_bytes() {
    let a = productive("01_core/z.rs", "00_nucleo/prompts/shared-z.md");
    let b = productive("01_core/a.rs", "00_nucleo/prompts/shared-z.md");
    let c = productive("02_shell/y.rs", "00_nucleo/prompts/shared-a.md");
    let d = productive("02_shell/b.rs", "00_nucleo/prompts/shared-a.md");
    let permutations = [
        vec![a.clone(), b.clone(), c.clone(), d.clone()],
        vec![d.clone(), c.clone(), b.clone(), a.clone()],
        vec![b.clone(), d.clone(), a.clone(), c.clone()],
        vec![c, a, d, b],
    ];
    let expected = bytes(&permutations[0]);
    for permutation in &permutations[1..] {
        assert_eq!(bytes(permutation), expected);
    }
    let rendered = String::from_utf8(expected).expect("diagnostics are UTF-8");
    assert!(rendered.find("shared-a.md").unwrap() < rendered.find("shared-z.md").unwrap());
    assert!(rendered.find("02_shell/b.rs").unwrap() < rendered.find("02_shell/y.rs").unwrap());
}

#[test]
fn hostile_paths_do_not_select_an_implicit_owner() {
    let prompt = "00_nucleo/prompts/Árvore.md";
    let entries = vec![
        productive("z/último.rs", prompt),
        productive("A/CASE.rs", prompt),
        productive("a/case.rs", prompt),
        productive(&format!("extreme/{}.rs", "x".repeat(4096)), prompt),
    ];
    let violations = check_prompt_ownership(&entries);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].location.path, PathBuf::from("A/CASE.rs"));
    assert!(violations[0].message.contains("4"));
    for entry in &entries {
        assert!(violations[0]
            .message
            .contains(&entry.code_path.to_string_lossy().into_owned()));
    }

    // Ausência/valor vazio já classificado a montante não vira colisão V15.
    let empty_upstream = ownership("01_core/empty.rs", PromptOwnershipLayer::L1, &[]);
    assert!(check_prompt_ownership(&[empty_upstream]).is_empty());
}

#[test]
fn non_productive_layers_do_not_enter_ownership() {
    for layer in [
        PromptOwnershipLayer::L0,
        PromptOwnershipLayer::Lab,
        PromptOwnershipLayer::Unknown,
    ] {
        let entries = vec![
            ownership("one", layer, &["00_nucleo/prompts/shared.md"]),
            ownership("two", layer, &["00_nucleo/prompts/shared.md"]),
        ];
        assert!(check_prompt_ownership(&entries).is_empty());
    }

    for layer in [
        PromptOwnershipLayer::L1,
        PromptOwnershipLayer::L2,
        PromptOwnershipLayer::L3,
        PromptOwnershipLayer::L4,
    ] {
        let entries = vec![
            ownership("one", layer, &["00_nucleo/prompts/shared.md"]),
            ownership("two", layer, &["00_nucleo/prompts/shared.md"]),
        ];
        assert_eq!(check_prompt_ownership(&entries).len(), 1);
    }
}
