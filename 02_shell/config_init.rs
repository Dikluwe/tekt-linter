//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/shell/config-init.md
//! @prompt-hash f6ad4522
//! @layer L2
//! @updated 2026-08-26

use std::path::{Path, PathBuf};

use clap::Args;

use crate::entities::config_init::{DetectedLayer, TopologyAmbiguity};
use crate::shell::path_encoding::human_path;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Project root where crystalline.toml will be created
    #[arg(default_value = ".")]
    pub path: PathBuf,
}

pub fn created(path: &Path) -> String {
    format!("CREATED {}\n", human_path(path))
}

pub fn present(path: &Path) -> String {
    format!(
        "crystalline-lint: config already exists: {}\n",
        human_path(path)
    )
}

pub fn unsafe_message(reason: &str) -> String {
    format!("crystalline-lint: unsafe config initialization: {reason}\n")
}

pub fn unknown_message(reason: &TopologyAmbiguity) -> String {
    let reason = match reason {
        TopologyAmbiguity::CaseAmbiguous(names) => format!(
            "case-ambiguous layer directories: {}",
            names
                .iter()
                .map(|name| String::from_utf8_lossy(name).into_owned())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TopologyAmbiguity::RequiredMissing(layers) => format!(
            "required directories missing: {}",
            layers
                .iter()
                .map(|layer| match layer {
                    DetectedLayer::L0 => "00_nucleo",
                    DetectedLayer::L1 => "01_core",
                    _ => unreachable!("only required layers can be missing"),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    format!("crystalline-lint: cannot infer Tekt topology: {reason}\n")
}
