//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/external-type-in-contract.md
//! @prompt-hash cc425ff2
//! @layer L1
//! @updated 2026-06-24

use std::collections::{HashMap, HashSet};

/// Whitelist de pacotes externos permitidos em L1.
/// Construída de config.l1_allowed_external em L4.
/// Injectada em V14 via parâmetro — L1 nunca lê o toml.
///
/// Suporta dois níveis de granularidade:
/// - crate-level (legacy): crate listado com conjunto vazio de itens autoriza
///   qualquer item desse crate;
/// - type-level: crate listado com conjunto não-vazio de itens autoriza apenas
///   os itens explicitamente nomeados (ou o último segmento de caminhos
///   qualificados, ex.: `citationberg::IndependentStyle` casa com
///   `IndependentStyle`).
pub struct L1AllowedExternal {
    /// Mapa crate → conjunto de itens autorizados.
    /// Conjunto vazio significa "todos os itens do crate estão autorizados".
    allowed: HashMap<String, HashSet<String>>,
    /// Prefixos sempre isentos (stdlib) — nunca verificados contra whitelist.
    /// Rust:   ["std", "core", "alloc", "super", "crate"]
    /// Python: módulos da stdlib são passados via `allowed` em crystalline.toml;
    ///         não há prefixo especial — todo import verificado contra whitelist.
    /// TS/C/Cpp: sem prefixos isentos (stdlib passada via allowed).
    exempt_prefixes: Vec<String>,
}

impl L1AllowedExternal {
    // ── Construtores por linguagem ─────────────────────────────────────────────

    pub fn for_rust(allowed: HashMap<String, HashSet<String>>) -> Self {
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
    pub fn for_python(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// TypeScript — nenhum prefixo isento automático.
    pub fn for_typescript(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// C — nenhum prefixo isento automático.
    pub fn for_c(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// C++ — nenhum prefixo isento automático.
    pub fn for_cpp(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self { allowed, exempt_prefixes: vec![] }
    }

    /// Zig — stdlib (std) é isenta por padrão.
    pub fn for_zig(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self { allowed, exempt_prefixes: vec!["std".to_string()] }
    }

    /// Go — stdlib isenta por padrão.
    pub fn for_go(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "fmt".to_string(),
                "strings".to_string(),
                "time".to_string(),
                "context".to_string(),
                "sync".to_string(),
                "os".to_string(),
                "path".to_string(),
                "filepath".to_string(),
                "bytes".to_string(),
                "io".to_string(),
                "errors".to_string(),
                "testing".to_string(),
                "log".to_string(),
                "net".to_string(),
                "math".to_string(),
                "sort".to_string(),
                "strconv".to_string(),
                "encoding".to_string(),
                "reflect".to_string(),
            ],
        }
    }

    /// Java — stdlib basica (java.util, java.lang, java.math) isenta por padrão.
    pub fn for_java(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "java.util".to_string(),
                "java.lang".to_string(),
                "java.math".to_string(),
            ],
        }
    }

    /// Elixir — módulos padrão do Kernel/Stdlib isentos por padrão.
    pub fn for_elixir(allowed: HashMap<String, HashSet<String>>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "Kernel".to_string(),
                "Enum".to_string(),
                "Map".to_string(),
                "List".to_string(),
                "String".to_string(),
                "Tuple".to_string(),
                "Module".to_string(),
                "ExUnit".to_string(),
            ],
        }
    }

    // ── Helpers de conveniência ────────────────────────────────────────────────

    pub fn empty_for_rust() -> Self {
        Self::for_rust(HashMap::new())
    }

    // ── Consulta ──────────────────────────────────────────────────────────────

    /// Verifica se um item externo é autorizado.
    ///
    /// `package_name` — primeiro segmento do path (nome do crate).
    /// `item_path` — caminho do item dentro do crate, sem o nome do crate
    /// (ex.: "EcoString", "citationberg::IndependentStyle"). `None` indica
    /// verificação crate-level legacy.
    pub fn is_allowed(&self, package_name: &str, item_path: Option<&str>) -> bool {
        if self.exempt_prefixes.iter().any(|p| package_name == p) {
            return true;
        }
        let Some(items) = self.allowed.get(package_name) else {
            return false;
        };
        let Some(item_path) = item_path else {
            // crate-level legacy: crate listado sem restrições de item.
            return true;
        };
        if items.is_empty() {
            // crate listado com lista de itens vazia: autoriza qualquer item.
            return true;
        }
        // Match directo (ex.: "EcoString" ou "citationberg::IndependentStyle").
        if items.contains(item_path) {
            return true;
        }
        // Match pelo último segmento (ex.: "citationberg::IndependentStyle" →
        // "IndependentStyle").
        if let Some(last) = item_path.split("::").last() {
            if items.contains(last) {
                return true;
            }
        }
        false
    }

    /// Verifica se um crate está listado, independentemente de itens.
    pub fn is_crate_allowed(&self, package_name: &str) -> bool {
        if self.exempt_prefixes.iter().any(|p| package_name == p || package_name.starts_with(&format!("{p}."))) {
            return true;
        }
        self.allowed.keys().any(|k| package_name == k || package_name.starts_with(&format!("{k}.")))
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
    pub go:         L1AllowedExternal,
    pub java:       L1AllowedExternal,
    pub elixir:     L1AllowedExternal,
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
            Language::Go         => &self.go,
            Language::Java       => &self.java,
            Language::Elixir     => &self.elixir,
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
        assert!(allowed.is_allowed("std", Some("collections::HashMap")));
        assert!(allowed.is_allowed("core", Some("fmt::Display")));
        assert!(allowed.is_allowed("alloc", Some("string::String")));
    }

    #[test]
    fn unlisted_package_is_not_allowed() {
        let allowed = L1AllowedExternal::empty_for_rust();
        assert!(!allowed.is_allowed("tokio", Some("Mutex")));
        assert!(!allowed.is_allowed("comemo", Some("Tracked")));
    }

    #[test]
    fn listed_package_legacy_is_allowed_for_any_item() {
        let mut map = HashMap::new();
        map.insert("thiserror".to_string(), HashSet::new());
        let allowed = L1AllowedExternal::for_rust(map);
        assert!(allowed.is_allowed("thiserror", Some("Error")));
        assert!(allowed.is_allowed("thiserror", None));
        assert!(!allowed.is_allowed("serde", Some("Serialize")));
    }

    #[test]
    fn type_level_whitelist_allows_only_listed_items() {
        let mut map = HashMap::new();
        let mut items = HashSet::new();
        items.insert("EcoString".to_string());
        items.insert("EcoVec".to_string());
        map.insert("ecow".to_string(), items);
        let allowed = L1AllowedExternal::for_rust(map);

        assert!(allowed.is_allowed("ecow", Some("EcoString")));
        assert!(allowed.is_allowed("ecow", Some("EcoVec")));
        assert!(!allowed.is_allowed("ecow", Some("EcoMap")));
        // Último segmento também casa.
        assert!(allowed.is_allowed("ecow", Some("vec::EcoVec")));
        assert!(!allowed.is_allowed("ecow", Some("map::EcoMap")));
    }
}
