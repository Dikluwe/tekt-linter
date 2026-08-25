//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 10f02f10
//! @layer L1
//! @updated 2026-08-18

pub enum SyntheticKind {
    A,
    B,
    C,
}

pub fn handle_synthetic(k: SyntheticKind) -> i32 {
    match k {
        SyntheticKind::A => 1,
        _ => 0, // neutro: N16[α] — fechamento
    }
}

pub fn handle_synthetic_b(k: SyntheticKind) -> Option<i32> {
    match k {
        SyntheticKind::B => Some(2),
        _ => None, // neutro: N16[β] — uniforme
    }
}

pub fn handle_synthetic_c(k: SyntheticKind) -> i32 {
    match k {
        SyntheticKind::C => 3,
        _ => -1, // neutro: N16[γ] — fallback aberto sob evolução
    }
}
