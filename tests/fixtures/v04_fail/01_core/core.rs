//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-08

pub fn read_it(p: &str) {
    let _ = std::fs::metadata(p);
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
