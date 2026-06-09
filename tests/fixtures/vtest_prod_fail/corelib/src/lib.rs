//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-09

pub struct Item;
use infra::Thing;
pub fn build(_t: Thing) {}

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
