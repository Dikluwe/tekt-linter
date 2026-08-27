//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/shell/command.md
//! @prompt-hash a41dc754
//! @layer L2
//! @updated 2026-08-26

use clap::Subcommand;

use crate::shell::config_init::InitArgs;
use crate::shell::refinement::{RefineArgs, RefineRevisionsArgs, SealRefinementArgs, SnapshotArgs};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a deterministic crystalline.toml for a recognized Tekt project
    Init(InitArgs),
    /// Compare explicit before/after semantic snapshots under a refinement contract
    Refine(RefineArgs),
    /// Extract a deterministic Rust-query refinement snapshot from a project
    Snapshot(SnapshotArgs),
    /// Compare two immutable Git revisions without checkout
    RefineRevisions(RefineRevisionsArgs),
    /// Validate frozen refinement oracles and publish a deterministic receipt
    SealRefinement(SealRefinementArgs),
}
