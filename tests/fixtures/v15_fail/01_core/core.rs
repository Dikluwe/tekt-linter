//! @prompt 00_nucleo/prompts/core.md
//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-07-23

pub fn add(a:u32,b:u32)->u32{a+b}

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
