use std::path::PathBuf;

use crystalline_lint::shell::fix_hashes::{
    format_plan, format_results, FixEntry, FixResult, FixUnavailable,
};

fn assert_contains(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected output to contain {needle:?}, got:\n{haystack}"
        );
    }
}

#[test]
fn dry_run_exposes_both_paths_and_all_three_hash_values_without_claiming_fixed() {
    let output = format_results(
        &[FixResult::DryRun {
            source_path: PathBuf::from("hostile/source path [dry-run].rs"),
            prompt_path: "hostile/prompt path [dry-run].md".into(),
            old_hash: "OLD::<dry-run>".into(),
            new_hash: "HASH-A::<dry-run>".into(),
            source_hash: "HASH-B::<dry-run>".into(),
        }],
        0,
        0,
    );

    assert_contains(
        &output,
        &[
            "hostile/source path [dry-run].rs",
            "hostile/prompt path [dry-run].md",
            "OLD::<dry-run>",
            "HASH-A::<dry-run>",
            "HASH-B::<dry-run>",
        ],
    );
    assert!(
        !output.to_ascii_lowercase().contains("fixed"),
        "dry-run must not claim completed application:\n{output}"
    );
}

#[test]
fn applied_is_observably_distinct_from_dry_run() {
    let dry_run = format_results(
        &[FixResult::DryRun {
            source_path: PathBuf::from("same-source.rs"),
            prompt_path: "same-prompt.md".into(),
            old_hash: "same-old".into(),
            new_hash: "same-hash-a".into(),
            source_hash: "same-hash-b".into(),
        }],
        0,
        0,
    );
    let applied = format_results(
        &[FixResult::Applied {
            source_path: PathBuf::from("same-source.rs"),
            prompt_path: "same-prompt.md".into(),
            new_hash: "same-hash-a".into(),
            source_hash: "same-hash-b".into(),
        }],
        0,
        0,
    );

    assert_ne!(
        dry_run, applied,
        "Applied and DryRun must not share a presentation"
    );
    assert_contains(
        &applied,
        &[
            "same-source.rs",
            "same-prompt.md",
            "same-hash-a",
            "same-hash-b",
        ],
    );
}

#[test]
fn write_failures_expose_the_failed_phase_reason_and_both_hashes() {
    let code_failed = format_results(
        &[FixResult::CodeWriteFailed {
            source_path: PathBuf::from("code-phase-source.rs"),
            prompt_path: "code-phase-prompt.md".into(),
            new_hash: "CODE-HASH-A".into(),
            source_hash: "CODE-HASH-B".into(),
            reason: "CODE-REASON::<hostile>".into(),
        }],
        0,
        0,
    );
    let partial = format_results(
        &[FixResult::PartialWrite {
            source_path: PathBuf::from("prompt-phase-source.rs"),
            prompt_path: "prompt-phase-prompt.md".into(),
            applied_new_hash: "PROMPT-HASH-A".into(),
            rejected_source_hash: "PROMPT-HASH-B".into(),
            reason: "PROMPT-REASON::<hostile>".into(),
        }],
        0,
        0,
    );

    assert!(code_failed.to_ascii_lowercase().contains("code"));
    assert_contains(
        &code_failed,
        &["CODE-REASON::<hostile>", "CODE-HASH-A", "CODE-HASH-B"],
    );
    assert!(partial.to_ascii_lowercase().contains("prompt"));
    assert_contains(
        &partial,
        &["PROMPT-REASON::<hostile>", "PROMPT-HASH-A", "PROMPT-HASH-B"],
    );
    assert_ne!(code_failed, partial);
}

fn unavailable_cases() -> Vec<FixUnavailable> {
    vec![
        FixUnavailable::HeaderUnreadable,
        FixUnavailable::PromptHashUnavailable {
            prompt_path: "prompt-unavailable.md".into(),
            old_hash: "PROMPT-OLD".into(),
            source_hash: "PROMPT-SOURCE-HASH".into(),
        },
        FixUnavailable::SourceHashUnavailable {
            prompt_path: "source-unavailable.md".into(),
            old_hash: "SOURCE-OLD".into(),
            new_hash: "SOURCE-NEW-HASH".into(),
        },
        FixUnavailable::BothHashesUnavailable {
            prompt_path: "both-unavailable.md".into(),
            old_hash: "BOTH-OLD".into(),
        },
    ]
}

#[test]
fn every_unavailable_plan_state_is_distinguishable_and_exposes_its_payload() {
    let outputs: Vec<_> = unavailable_cases()
        .into_iter()
        .enumerate()
        .map(|(index, reason)| {
            format_plan(&[FixEntry::Unavailable {
                source_path: PathBuf::from(format!("unavailable-plan-{index}.rs")),
                reason,
            }])
        })
        .collect();

    assert!(outputs[0].to_ascii_lowercase().contains("header"));
    assert_contains(&outputs[0], &["unavailable-plan-0.rs"]);
    assert_contains(
        &outputs[1],
        &[
            "unavailable-plan-1.rs",
            "prompt-unavailable.md",
            "PROMPT-OLD",
            "PROMPT-SOURCE-HASH",
        ],
    );
    assert_contains(
        &outputs[2],
        &[
            "unavailable-plan-2.rs",
            "source-unavailable.md",
            "SOURCE-OLD",
            "SOURCE-NEW-HASH",
        ],
    );
    assert_contains(
        &outputs[3],
        &["unavailable-plan-3.rs", "both-unavailable.md", "BOTH-OLD"],
    );

    for left in 0..outputs.len() {
        for right in left + 1..outputs.len() {
            assert_ne!(outputs[left], outputs[right]);
        }
    }
}

#[test]
fn every_unavailable_result_state_is_distinguishable_and_exposes_its_payload() {
    let outputs: Vec<_> = unavailable_cases()
        .into_iter()
        .enumerate()
        .map(|(index, reason)| {
            format_results(
                &[FixResult::Unavailable {
                    source_path: PathBuf::from(format!("unavailable-result-{index}.rs")),
                    reason,
                }],
                0,
                0,
            )
        })
        .collect();

    assert!(outputs[0].to_ascii_lowercase().contains("header"));
    assert_contains(&outputs[0], &["unavailable-result-0.rs"]);
    assert_contains(
        &outputs[1],
        &[
            "unavailable-result-1.rs",
            "prompt-unavailable.md",
            "PROMPT-OLD",
            "PROMPT-SOURCE-HASH",
        ],
    );
    assert_contains(
        &outputs[2],
        &[
            "unavailable-result-2.rs",
            "source-unavailable.md",
            "SOURCE-OLD",
            "SOURCE-NEW-HASH",
        ],
    );
    assert_contains(
        &outputs[3],
        &["unavailable-result-3.rs", "both-unavailable.md", "BOTH-OLD"],
    );

    for left in 0..outputs.len() {
        for right in left + 1..outputs.len() {
            assert_ne!(outputs[left], outputs[right]);
        }
    }
}
