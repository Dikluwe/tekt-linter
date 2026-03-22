//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/external-type-in-contract.md
//! @prompt-hash 8491d7c1
//! @layer L1
//! @updated 2026-03-22

use std::collections::HashSet;

/// Whitelist de pacotes externos permitidos em L1.
/// Construída de config.l1_allowed_external em L4.
/// Injectada em V14 via parâmetro — L1 nunca lê o toml.
pub struct L1AllowedExternal {
    /// Nomes de pacotes autorizados para a linguagem em análise.
    allowed: HashSet<String>,
    /// Prefixos sempre isentos (stdlib) — nunca verificados contra whitelist.
    /// Rust: ["std", "core", "alloc"]
    exempt_prefixes: Vec<String>,
}

impl L1AllowedExternal {
    pub fn for_rust(allowed: HashSet<String>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "std".to_string(),
                "core".to_string(),
                "alloc".to_string(),
                // Intra-crate path qualifiers — never external packages
                "super".to_string(),
                "crate".to_string(),
            ],
        }
    }

    pub fn empty_for_rust() -> Self {
        Self::for_rust(HashSet::new())
    }

    pub fn is_allowed(&self, package_name: &str) -> bool {
        if self.exempt_prefixes.iter().any(|p| package_name == p) {
            return true;
        }
        self.allowed.contains(package_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_is_always_exempt() {
        let allowed = L1AllowedExternal::empty_for_rust();
        assert!(allowed.is_allowed("std"));
        assert!(allowed.is_allowed("core"));
        assert!(allowed.is_allowed("alloc"));
    }

    #[test]
    fn unlisted_package_is_not_allowed() {
        let allowed = L1AllowedExternal::empty_for_rust();
        assert!(!allowed.is_allowed("tokio"));
        assert!(!allowed.is_allowed("comemo"));
    }

    #[test]
    fn listed_package_is_allowed() {
        let mut set = HashSet::new();
        set.insert("thiserror".to_string());
        let allowed = L1AllowedExternal::for_rust(set);
        assert!(allowed.is_allowed("thiserror"));
        assert!(!allowed.is_allowed("serde"));
    }
}
