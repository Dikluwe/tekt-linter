//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 53a76454
//! @layer L3
//! @updated 2026-03-13

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::entities::layer::Layer;

#[derive(Debug, Clone, Deserialize)]
pub struct CrystallineConfig {
    #[serde(default)]
    pub layers: HashMap<String, String>,
    /// Rust module name → layer string ("L1", "L2", ...)
    #[serde(default)]
    pub module_layers: HashMap<String, String>,
}

impl CrystallineConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        toml::from_str(&content).map_err(|e| format!("Invalid TOML: {e}"))
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

        Self { layers, module_layers }
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
}
