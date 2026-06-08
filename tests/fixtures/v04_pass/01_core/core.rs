//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-08

pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t() { assert_eq!(add(1, 2), 3); }
}
