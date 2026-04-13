//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt 00_nucleo/prompts/architecture-standards.md
//! @prompt-hash 44f1f602
//! @layer L0
//! @updated 2026-03-20

// ── L1: Core ─────────────────────────────────────────────────────────────────
#[path = "01_core/entities/mod.rs"]
pub mod entities;

#[path = "01_core/contracts/mod.rs"]
pub mod contracts;

#[path = "01_core/rules/mod.rs"]
pub mod rules;

// ── L3: Infra ─────────────────────────────────────────────────────────────────
#[path = "03_infra/mod.rs"]
pub mod infra;

// ── L2: Shell ────────────────────────────────────────────────────────────────
#[path = "02_shell/mod.rs"]
pub mod shell;
