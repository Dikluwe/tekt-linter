//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash ad60b69b
//! @layer L1
//! @updated 2026-03-22

pub mod alien_file;
pub mod dangling_contract;
pub mod external_type_in_contract;
pub mod forbidden_import;
pub mod impure_core;
pub mod mutable_state_core;
pub mod orphan_prompt;
pub mod prompt_drift;
pub mod prompt_header;
pub mod prompt_stale;
pub mod pub_leak;
pub mod quarantine_leak;
pub mod test_file;
pub mod wiring_logic_leak;

