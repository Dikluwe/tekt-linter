//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-07-23

pub fn add(a:u32,b:u32)->u32{a+b}

// @prompt 00_nucleo/prompts/core.md — menção em comentário normal,
// fora do bloco de doc-header: NÃO conta para V15.

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
