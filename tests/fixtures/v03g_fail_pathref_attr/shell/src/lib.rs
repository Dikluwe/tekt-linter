//! @prompt 00_nucleo/prompts/core.md
//! @layer L2
//! @updated 2026-06-08

pub struct Cfg {
    #[arg(default_value_t = wiremod::N)]
    pub field: u32,
}
