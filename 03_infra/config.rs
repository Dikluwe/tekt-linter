//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 9e806f55
//! @layer L3
//! @updated 2026-03-23

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Deserialize;

use crate::entities::layer::Layer;
use crate::entities::violation::ViolationLevel;

/// Entrada individual de `[rules]` — nível configurável por regra.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleEntry {
    pub level: Option<String>,
}

/// Configuração de exceções para V12 — lida de `[wiring_exceptions]`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WiringExceptionsConfig {
    /// `true` (padrão): structs de adapter são permitidas em L4.
    /// `false`: structs em L4 também disparam V12.
    pub allow_adapter_structs: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrystallineConfig {
    #[serde(default)]
    pub layers: HashMap<String, String>,
    /// Rust module name → layer string ("L1", "L2", ...)
    #[serde(default)]
    pub module_layers: HashMap<String, String>,
    /// Diretórios explicitamente excluídos — não disparam V8.
    /// Exemplo: { "build" = "target", "vcs" = ".git" }
    #[serde(default)]
    pub excluded: HashMap<String, String>,
    /// Subdiretórios de L1 acessíveis de L2/L3 — portas públicas para V9.
    /// Exemplo: { "entities" = "01_core/entities" }
    #[serde(default)]
    pub l1_ports: HashMap<String, String>,
    /// Prompts que existem legitimamente sem materialização Rust — isentos de V7.
    /// Exemplo: { "00_nucleo/prompts/template.md" = "template" }
    #[serde(default)]
    pub orphan_exceptions: HashMap<String, String>,
    /// Configuração de exceções V12 — lida de `[wiring_exceptions]`.
    #[serde(default)]
    pub wiring_exceptions: WiringExceptionsConfig,
    /// Aliases de path TypeScript — lida de `[ts_aliases]` (ADR-0009).
    /// Exemplo: { "@core" = "01_core", "@shell" = "02_shell" }
    #[serde(default)]
    pub ts_aliases: HashMap<String, String>,
    /// Aliases de package Python — lida de `[py_aliases]`.
    /// Exemplo: { "core" = "01_core", "shell" = "02_shell" }
    #[serde(default)]
    pub py_aliases: HashMap<String, String>,
    /// Ficheiros individuais excluídos por path relativo à raiz.
    /// Distinto de `excluded` que opera sobre nomes de directório.
    /// Exemplo: { "crate_root" = "lib.rs" }
    #[serde(default)]
    pub excluded_files: HashMap<String, String>,
    /// Pacotes externos permitidos em L1 por linguagem.
    /// Se ausente, L1 não pode importar nenhum externo.
    /// Chave: "rust", "typescript", "python"
    /// Valor: lista de nomes de pacote
    #[serde(default)]
    pub l1_allowed_external: HashMap<String, Vec<String>>,
    /// Níveis configuráveis por regra — lidos de `[rules]`.
    /// Exemplo: { "V11" => RuleEntry { level: Some("warning") } }
    #[serde(default)]
    pub rules: HashMap<String, RuleEntry>,

    /// Escape hatch para blanket impls do padrão 4 — ADR-0015.
    /// Traits satisfeitas por `impl<T: B> Trait for &T` / `Box<T>` / `Arc<T>`
    /// não são detectáveis estaticamente sem type checker completo.
    /// Chave: nome arbitrário (documentação). Valor: nome da trait.
    /// Exemplo: { "tracked_world_ref" = "TrackedWorld" }
    #[serde(default)]
    pub v11_blanket_exceptions: HashMap<String, String>,
}

impl CrystallineConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid TOML: {e}"))
    }

    /// Resolve o nível efectivo para uma regra.
    /// Se `[rules]` declara um nível para `rule_id`, esse nível é retornado.
    /// Caso contrário, retorna `default`.
    pub fn level_for(&self, rule_id: &str, default: ViolationLevel) -> ViolationLevel {
        self.rules
            .get(rule_id)
            .and_then(|e| e.level.as_deref())
            .and_then(|s| match s {
                "fatal" | "Fatal" => Some(ViolationLevel::Fatal),
                "error" | "Error" => Some(ViolationLevel::Error),
                "warning" | "Warning" => Some(ViolationLevel::Warning),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Returns the set of allowed external packages for a given language.
    /// Returns an empty set if the language is not present in the config.
    pub fn l1_allowed_for_language(&self, language: &str) -> HashSet<String> {
        self.l1_allowed_external
            .get(language)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Resolve a Rust module name (e.g. "entities") to a Layer.
    /// Used by LayerResolver in RustParser.
    pub fn layer_for_module(&self, module_name: &str) -> Layer {
        match self.module_layers.get(module_name).map(String::as_str) {
            Some("L0") => Layer::L0,
            Some("L1") => Layer::L1,
            Some("L2") => Layer::L2,
            Some("L3") => Layer::L3,
            Some("L4") => Layer::L4,
            Some("lab") | Some("Lab") => Layer::Lab,
            _ => Layer::Unknown,
        }
    }
}

impl Default for CrystallineConfig {
    fn default() -> Self {
        let mut module_layers = HashMap::new();
        module_layers.insert("entities".to_string(), "L1".to_string());
        module_layers.insert("contracts".to_string(), "L1".to_string());
        module_layers.insert("rules".to_string(), "L1".to_string());
        module_layers.insert("shell".to_string(), "L2".to_string());
        module_layers.insert("infra".to_string(), "L3".to_string());

        let mut layers = HashMap::new();
        layers.insert("L0".to_string(), "00_nucleo".to_string());
        layers.insert("L1".to_string(), "01_core".to_string());
        layers.insert("L2".to_string(), "02_shell".to_string());
        layers.insert("L3".to_string(), "03_infra".to_string());
        layers.insert("L4".to_string(), "04_wiring".to_string());
        layers.insert("lab".to_string(), "lab".to_string());

        let mut excluded = HashMap::new();
        excluded.insert("build".to_string(), "target".to_string());
        excluded.insert("vcs".to_string(), ".git".to_string());
        excluded.insert("deps".to_string(), "node_modules".to_string());
        excluded.insert("cargo".to_string(), ".cargo".to_string());
        // Python environment and caches (ADR-0006 compliance)
        excluded.insert("venv1".to_string(), ".venv".to_string());
        excluded.insert("venv2".to_string(), "venv".to_string());
        excluded.insert("venv3".to_string(), "env".to_string());
        excluded.insert("pycache".to_string(), "__pycache__".to_string());
        excluded.insert("pytest".to_string(), ".pytest_cache".to_string());
        excluded.insert("temp".to_string(), "tmp".to_string());

        let mut l1_ports = HashMap::new();
        l1_ports.insert("entities".to_string(), "01_core/entities".to_string());
        l1_ports.insert("contracts".to_string(), "01_core/contracts".to_string());
        l1_ports.insert("rules".to_string(), "01_core/rules".to_string());

        Self {
            layers,
            module_layers,
            excluded,
            l1_ports,
            orphan_exceptions: HashMap::new(),
            wiring_exceptions: WiringExceptionsConfig::default(),
            ts_aliases: HashMap::new(),
            py_aliases: HashMap::new(),
            excluded_files: HashMap::new(),
            l1_allowed_external: HashMap::new(),
            rules: HashMap::new(),
            v11_blanket_exceptions: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_resolves_entities_to_l1() {
        let config = CrystallineConfig::default();
        assert_eq!(config.layer_for_module("entities"), Layer::L1);
    }

    #[test]
    fn default_config_resolves_shell_to_l2() {
        let config = CrystallineConfig::default();
        assert_eq!(config.layer_for_module("shell"), Layer::L2);
    }

    #[test]
    fn default_config_resolves_infra_to_l3() {
        let config = CrystallineConfig::default();
        assert_eq!(config.layer_for_module("infra"), Layer::L3);
    }

    #[test]
    fn unknown_module_resolves_to_unknown() {
        let config = CrystallineConfig::default();
        assert_eq!(config.layer_for_module("reqwest"), Layer::Unknown);
    }

    #[test]
    fn excluded_files_defaults_to_empty() {
        let config = CrystallineConfig::default();
        assert!(config.excluded_files.is_empty());
    }

    #[test]
    fn l1_allowed_external_defaults_to_empty() {
        let config = CrystallineConfig::default();
        assert!(config.l1_allowed_external.is_empty());
    }

    #[test]
    fn l1_allowed_for_language_returns_empty_for_missing_key() {
        let config = CrystallineConfig::default();
        assert!(config.l1_allowed_for_language("rust").is_empty());
    }

    #[test]
    fn level_for_returns_default_when_rules_empty() {
        let config = CrystallineConfig::default();
        assert_eq!(config.level_for("V11", ViolationLevel::Error), ViolationLevel::Error);
        assert_eq!(config.level_for("V7", ViolationLevel::Warning), ViolationLevel::Warning);
    }

    #[test]
    fn level_for_returns_configured_level_when_declared() {
        let mut config = CrystallineConfig::default();
        config.rules.insert("V11".to_string(), RuleEntry { level: Some("warning".to_string()) });
        assert_eq!(config.level_for("V11", ViolationLevel::Error), ViolationLevel::Warning);
    }

    #[test]
    fn level_for_unknown_rule_returns_default() {
        let config = CrystallineConfig::default();
        assert_eq!(config.level_for("V99", ViolationLevel::Warning), ViolationLevel::Warning);
    }
}
