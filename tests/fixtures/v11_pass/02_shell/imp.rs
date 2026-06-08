//! @prompt 00_nucleo/prompts/core.md
//! @layer L2
//! @updated 2026-06-08

use crate::contracts::contract::Greeter;
pub struct G;
impl Greeter for G { fn hi(&self){} }
