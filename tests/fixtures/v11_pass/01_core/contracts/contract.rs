//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-08

pub trait Greeter { fn hi(&self); }

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
