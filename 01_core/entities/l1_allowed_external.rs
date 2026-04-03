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
    /// Rust:   ["std", "core", "alloc", "super", "crate"]
    /// Python: módulos da stdlib são passados via `allowed` em crystalline.toml;
    ///         não há prefixo especial — todo import verificado contra whitelist.
    /// TS/C/Cpp: sem prefixos isentos (stdlib passada via allowed).
    exempt_prefixes: Vec<String>,
}

impl L1AllowedExternal {
    // ── Construtores por linguagem ─────────────────────────────────────────────

    pub fn for_rust(allowed: HashSet<String>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "std".to_string(),
                "core".to_string(),
                "alloc".to_string(),
                // Qualificadores intra-crate — nunca são pacotes externos
                "super".to_string(),
                "crate".to_string(),
            ],
        }
    }

    /// Python — nenhum prefixo isento automático.
    /// Módulos stdlib (typing, math, …) devem ser declarados em
    /// `[l1_allowed_external] python = ["typing", "math", …]`.
    pub fn for_python(allowed: HashSet<String>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// TypeScript — nenhum prefixo isento automático.
    pub fn for_typescript(allowed: HashSet<String>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// C — nenhum prefixo isento automático.
    pub fn for_c(allowed: HashSet<String>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// C++ — nenhum prefixo isento automático.
    pub fn for_cpp(allowed: HashSet<String>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// Zig — nenhum prefixo isento automático.
    pub fn for_zig(allowed: HashSet<String>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    // ── Helpers de conveniência ────────────────────────────────────────────────

    pub fn empty_for_rust() -> Self {
        Self::for_rust(HashSet::new())
    }

    // ── Consulta ──────────────────────────────────────────────────────────────

    pub fn is_allowed(&self, package_name: &str) -> bool {
        if self.exempt_prefixes.iter().any(|p| package_name == p) {
            return true;
        }
        self.allowed.contains(package_name)
    }
}

/// Agrega as whitelists de L1 para todas as linguagens suportadas.
/// Construído em L4 a partir do crystalline.toml — L1 nunca lê a config.
/// V14 recebe `&L1AllowedExternalSet` e chama `for_language(&file.language)`
/// para obter a instância correta para cada arquivo analisado.
pub struct L1AllowedExternalSet {
    pub rust:       L1AllowedExternal,
    pub python:     L1AllowedExternal,
    pub typescript: L1AllowedExternal,
    pub c:          L1AllowedExternal,
    pub cpp:        L1AllowedExternal,
    pub zig:        L1AllowedExternal,
}

impl L1AllowedExternalSet {
    pub fn for_language<'a>(&'a self, language: &crate::entities::layer::Language) -> &'a L1AllowedExternal {
        use crate::entities::layer::Language;
        match language {
            Language::Rust       => &self.rust,
            Language::Python     => &self.python,
            Language::TypeScript => &self.typescript,
            Language::C          => &self.c,
            Language::Cpp        => &self.cpp,
            Language::Zig        => &self.zig,
            Language::Unknown    => &self.rust, // fallback conservador
        }
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
