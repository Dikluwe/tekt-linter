//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-08

pub struct S;
impl S {
    pub fn go(&self) -> u32 { 1 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
