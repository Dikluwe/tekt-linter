use std::path::PathBuf;

use crystalline_lint::shell::update_snapshot::{
    format_plan, format_results, SnapshotEntry, SnapshotResult, SnapshotUnreadable,
};

const HOSTILE_SNAPSHOT: &str =
    "<snapshot>\nUpdated: definitely-not-a-status\nWould update: payload-only\n{\"interface\":\"SENTINEL::完整::<&>\"}\n</snapshot>";

#[test]
fn dry_run_plan_makes_the_complete_ready_entry_observable() {
    let source = PathBuf::from("source/SENTINEL source.rs");
    let prompt = "00_nucleo/prompts/SENTINEL prompt.md";
    let output = format_plan(&[SnapshotEntry::Ready {
        source_path: source.clone(),
        prompt_path: prompt.to_owned(),
        snapshot: HOSTILE_SNAPSHOT.to_owned(),
    }]);

    assert!(
        output.contains(source.to_str().unwrap()),
        "source path hidden: {output:?}"
    );
    assert!(output.contains(prompt), "prompt path hidden: {output:?}");
    assert!(
        output.contains(HOSTILE_SNAPSHOT),
        "snapshot was omitted, escaped, truncated, or reconstructed: {output:?}"
    );
}

#[test]
fn plan_presents_unreadable_causes_distinctly() {
    let missing_file = format_plan(&[SnapshotEntry::Unreadable {
        source_path: PathBuf::from("source/unreadable.rs"),
        reason: SnapshotUnreadable::MissingParsedFile,
    }]);
    let missing_header = format_plan(&[SnapshotEntry::Unreadable {
        source_path: PathBuf::from("source/unreadable.rs"),
        reason: SnapshotUnreadable::MissingPromptHeader,
    }]);

    assert_ne!(
        missing_file, missing_header,
        "the two normative unreadable causes are presentation-indistinguishable"
    );
}

#[test]
fn dry_run_result_is_not_presented_as_a_completed_update() {
    let source = PathBuf::from("source/SENTINEL result.rs");
    let prompt = "00_nucleo/prompts/SENTINEL result.md";
    let output = format_results(
        &[SnapshotResult::DryRun {
            source_path: source.clone(),
            prompt_path: prompt.to_owned(),
            snapshot: HOSTILE_SNAPSHOT.to_owned(),
        }],
        0,
    );

    assert!(
        output.contains(source.to_str().unwrap()),
        "source path hidden: {output:?}"
    );
    assert!(output.contains(prompt), "prompt path hidden: {output:?}");
    assert!(
        output.contains(HOSTILE_SNAPSHOT),
        "dry-run snapshot was omitted, escaped, truncated, or reconstructed: {output:?}"
    );

    let presentation = output.to_ascii_lowercase();
    assert!(
        presentation.contains("dry-run") || presentation.contains("would update"),
        "DryRun lacks an explicit dry-run/would-update status: {output:?}"
    );

    let status_text = output.replacen(HOSTILE_SNAPSHOT, "<snapshot-payload>", 1);
    assert!(
        !status_text.contains("Updated"),
        "DryRun uses the completed-update label: {output:?}"
    );
}
