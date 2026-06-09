//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-09

pub struct Item;

#[cfg(test)]
mod tests {
    use infra::Thing;
    #[test]
    fn t() { let _x: Option<Thing> = None; }
}
