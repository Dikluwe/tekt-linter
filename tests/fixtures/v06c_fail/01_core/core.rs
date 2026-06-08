//! @prompt 00_nucleo/prompts/core.md
//! @layer L1
//! @updated 2026-06-08

pub fn transform(input: u32, label: &str) -> Option<u32> { let _ = label; Some(input) }
pub struct Point { x: i32, y: i32 }
pub enum Shape { Circle, Square }
pub trait Greeter { fn greet(&self) -> String; }
pub struct Wrapper<T> { inner: T }

#[cfg(test)]
mod tests {
    #[test]
    fn t() { assert!(true); }
}
