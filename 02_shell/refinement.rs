//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/refinement-validator.md
//! @prompt-hash cc8920e0
//! @layer L2
//! @updated 2026-08-23

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use serde_json::json;

use crate::entities::refinement::{
    Inconclusive, ObservableValue, RefinementVerdict, UnknownReason, Witness,
};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compare explicit before/after semantic snapshots under a refinement contract
    Refine(RefineArgs),
    /// Extract a deterministic Rust-query refinement snapshot from a project
    Snapshot(SnapshotArgs),
    /// Compare two immutable Git revisions without checkout
    RefineRevisions(RefineRevisionsArgs),
    /// Validate frozen refinement oracles and publish a deterministic receipt
    SealRefinement(SealRefinementArgs),
}

#[derive(Debug, Args)]
pub struct SealRefinementArgs {
    /// Git repository and root of all frozen inputs
    pub repository: PathBuf,
    /// Repository-relative segregated refinement manifest
    #[arg(long)]
    pub manifest: PathBuf,
    /// Destination JSON receipt
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct RefineArgs {
    /// Source artifact facts in refinement snapshot JSON format
    #[arg(long)]
    pub before: PathBuf,
    /// Target artifact facts in refinement snapshot JSON format
    #[arg(long)]
    pub after: PathBuf,
    /// Refinement contract in TOML format
    #[arg(long)]
    pub contract: PathBuf,
    /// Output format for the refinement verdict
    #[arg(long, default_value = "text")]
    pub format: RefinementOutputFormat,
}

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    /// Project root whose files are queried
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Refinement contract containing [[observable]] declarations
    #[arg(long)]
    pub contract: PathBuf,
    /// Stable artifact identifier recorded in the snapshot
    #[arg(long)]
    pub artifact_id: String,
    /// Destination refinement snapshot JSON file
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct RefineRevisionsArgs {
    /// Git repository whose object database is read without checkout
    #[arg(default_value = ".")]
    pub repository: PathBuf,
    /// Source commit SHA or ref, resolved once to an immutable OID
    #[arg(long)]
    pub before_ref: String,
    /// Target commit SHA or ref, resolved once to an immutable OID
    #[arg(long)]
    pub after_ref: String,
    /// Refinement contract containing observables and relations
    #[arg(long)]
    pub contract: PathBuf,
    /// Output format for the refinement verdict
    #[arg(long, default_value = "text")]
    pub format: RefinementOutputFormat,
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum RefinementOutputFormat {
    Text,
    Sarif,
}

pub fn exit_code(verdict: &RefinementVerdict) -> i32 {
    match verdict {
        RefinementVerdict::Preserved => 0,
        RefinementVerdict::Violated { .. } => 1,
        RefinementVerdict::Unknown { .. } => 2,
    }
}

pub fn format_snapshot_success(path: &std::path::Path, observable_count: usize) -> String {
    format!(
        "SNAPSHOT {} ({} observables)\n",
        path.display(),
        observable_count
    )
}

fn value_text(value: &ObservableValue) -> String {
    match value {
        ObservableValue::Known(value) => format!("known({value})"),
        ObservableValue::Absent => "absent".to_string(),
        ObservableValue::Unknown(reason) => format!("unknown({})", reason_text(reason)),
    }
}

fn reason_text(reason: &UnknownReason) -> &'static str {
    match reason {
        UnknownReason::MissingObservable => "missing-observable",
        UnknownReason::AmbiguousIdentity => "ambiguous-identity",
        UnknownReason::UnsupportedParser => "unsupported-parser",
        UnknownReason::OpaqueConstruction => "opaque-construction",
        UnknownReason::PartialContract => "partial-contract",
        UnknownReason::BudgetExhausted => "budget-exhausted",
    }
}

fn witness_text(witness: &Witness) -> String {
    format!(
        "VIOLATED [{}:{}] {} -> {}: {} = {}; {} = {}",
        witness.contract_id,
        witness.relation,
        witness.source_artifact,
        witness.target_artifact,
        witness.source_observable.as_deref().unwrap_or("<none>"),
        witness
            .source_value
            .as_ref()
            .map(value_text)
            .unwrap_or_else(|| "<none>".to_string()),
        witness.target_observable,
        value_text(&witness.target_value),
    )
}

fn inconclusive_text(reason: &Inconclusive) -> String {
    format!(
        "UNKNOWN [{}:{}] {}: {}",
        reason.contract_id,
        reason.relation,
        reason.observable,
        reason_text(&reason.reason)
    )
}

pub fn format_text(verdict: &RefinementVerdict) -> String {
    match verdict {
        RefinementVerdict::Preserved => "PRESERVED\n".to_string(),
        RefinementVerdict::Violated {
            witnesses,
            inconclusive,
        } => {
            let mut lines: Vec<String> = witnesses.iter().map(witness_text).collect();
            lines.extend(inconclusive.iter().map(inconclusive_text));
            format!("{}\n", lines.join("\n"))
        }
        RefinementVerdict::Unknown { reasons } => {
            let lines: Vec<String> = reasons.iter().map(inconclusive_text).collect();
            format!("{}\n", lines.join("\n"))
        }
    }
}

pub fn format_sarif(verdict: &RefinementVerdict) -> String {
    let mut results = Vec::new();
    match verdict {
        RefinementVerdict::Preserved => {}
        RefinementVerdict::Violated {
            witnesses,
            inconclusive,
        } => {
            results.extend(witnesses.iter().map(|witness| {
                json!({
                    "ruleId": "REFINEMENT",
                    "level": "warning",
                    "message": { "text": witness_text(witness) },
                    "properties": {
                        "contractId": witness.contract_id,
                        "relation": witness.relation,
                        "sourceArtifact": witness.source_artifact,
                        "targetArtifact": witness.target_artifact,
                        "sourceExtractorVersion": witness.source_extractor_version,
                        "targetExtractorVersion": witness.target_extractor_version,
                    }
                })
            }));
            results.extend(inconclusive.iter().map(|reason| {
                json!({
                    "ruleId": "REFINEMENT_UNKNOWN",
                    "level": "note",
                    "message": { "text": inconclusive_text(reason) }
                })
            }));
        }
        RefinementVerdict::Unknown { reasons } => {
            results.extend(reasons.iter().map(|reason| {
                json!({
                    "ruleId": "REFINEMENT_UNKNOWN",
                    "level": "note",
                    "message": { "text": inconclusive_text(reason) }
                })
            }));
        }
    }
    serde_json::to_string_pretty(&json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": {
                "name": "crystalline-lint",
                "rules": [
                    { "id": "REFINEMENT", "name": "ContractRefinement" },
                    { "id": "REFINEMENT_UNKNOWN", "name": "RefinementUnknown" }
                ]
            }},
            "results": results
        }]
    }))
    .expect("refinement SARIF is serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserved_is_exit_zero() {
        assert_eq!(exit_code(&RefinementVerdict::Preserved), 0);
        assert_eq!(format_text(&RefinementVerdict::Preserved), "PRESERVED\n");
    }

    #[test]
    fn unknown_is_exit_two_and_sarif_note() {
        let verdict = RefinementVerdict::Unknown {
            reasons: vec![Inconclusive {
                contract_id: "c".to_string(),
                relation: "preserve".to_string(),
                observable: "x".to_string(),
                reason: UnknownReason::MissingObservable,
            }],
        };
        assert_eq!(exit_code(&verdict), 2);
        assert!(format_sarif(&verdict).contains("REFINEMENT_UNKNOWN"));
    }

    #[test]
    fn snapshot_success_is_stable() {
        assert_eq!(
            format_snapshot_success(std::path::Path::new("facts.json"), 3),
            "SNAPSHOT facts.json (3 observables)\n"
        );
    }
}
