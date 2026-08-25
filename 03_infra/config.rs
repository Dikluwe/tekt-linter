//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/config.md
//! @prompt-hash a7c5b358
//! @layer L3
//! @updated 2026-06-09

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use serde::Deserialize;

use crate::entities::layer::Layer;
use crate::entities::violation::ViolationLevel;

const LANGUAGES: &[&str] = &[
    "rust",
    "python",
    "typescript",
    "c",
    "cpp",
    "zig",
    "go",
    "java",
    "elixir",
];

fn is_language_key(key: &str) -> bool {
    LANGUAGES.contains(&key)
}

/// Entrada individual de `[rules]` — nível configurável por regra.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleEntry {
    pub level: Option<String>,
    pub languages: Option<Vec<String>>,
}

/// Entrada de `[l1_allowed_external]` — suporta formato legacy e type-level.
///
/// Legacy: `rust = ["thiserror"]` (lista de crates autorizadas crate-wide).
/// Type-level: `[l1_allowed_external.ecow] types = ["EcoString"]` (itens
/// explicitamente autorizados de uma crate).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AllowedExternalEntry {
    Legacy(Vec<String>),
    TypeLevel { types: Vec<String> },
}

/// Configuração de exceções para V12 — lida de `[wiring_exceptions]`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WiringExceptionsConfig {
    /// `true` (padrão): structs de adapter são permitidas em L4.
    /// `false`: structs em L4 também disparam V12.
    pub allow_adapter_structs: Option<bool>,
}

/// Configuração para V21 — lida de `[v21]`, `[v21.context_vars]`, etc.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct V21ScopeConfig {
    #[serde(default)]
    pub modules: Option<Vec<String>>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct V21TableConfig {
    #[serde(default)]
    pub context_vars: Option<Vec<String>>,
    #[serde(default)]
    pub geometric_sinks: Option<Vec<String>>,
    #[serde(default)]
    pub format_syntax_modules: Option<Vec<String>>,
    #[serde(default)]
    pub scope_modules: Option<Vec<String>>,
    #[serde(default)]
    pub scope_types: Option<Vec<String>>,
    #[serde(default)]
    pub strict_modules: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct V21TrivialConfig {
    #[serde(default)]
    pub literals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum V21TrivialEntry {
    List(Vec<String>),
    Table(V21TrivialConfig),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct V21StrictConfig {
    #[serde(default)]
    pub modules: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum V21StrictEntry {
    List(Vec<String>),
    Table(V21StrictConfig),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LineageConfig {
    #[serde(default)]
    pub strict_directories: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnalysisConfig {
    #[serde(default)]
    pub lineage: LineageConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SemanticResolverConfig {
    pub symbol: String,
    pub context_arg: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ContextSemanticContract {
    pub id: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub resolvers: Vec<SemanticResolverConfig>,
    #[serde(default)]
    pub erasing_projections: Vec<String>,
    #[serde(default)]
    pub sinks: Vec<String>,
    #[serde(default)]
    pub absolute_sources: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectionSemanticContract {
    pub id: String,
    #[serde(default)]
    pub language: String,
    pub scope: String,
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub neutral_forms: Vec<String>,
    #[serde(default = "preserve_normalization")]
    pub normalization: String,
}

fn preserve_normalization() -> String {
    "preserve".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DecisionSemanticContract {
    pub id: String,
    #[serde(default)]
    pub language: String,
    pub owner: String,
    #[serde(default)]
    pub consumers: Vec<String>,
    #[serde(default)]
    pub explicit_sources: Vec<String>,
    #[serde(default)]
    pub proxies: Vec<String>,
    #[serde(default)]
    pub duplicate_owners: Vec<String>,
    #[serde(default)]
    pub canonicalizers: Vec<String>,
    #[serde(default)]
    pub resolved_after: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SemanticContractsConfig {
    #[serde(default)]
    pub context: Vec<ContextSemanticContract>,
    #[serde(default)]
    pub projection: Vec<ProjectionSemanticContract>,
    #[serde(default)]
    pub decision: Vec<DecisionSemanticContract>,
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
    /// Exceções declaradas para V16 — lidas de `[wildcard_exceptions]`.
    /// Exemplo: { "01_core/src/entities/gradient.rs:221" = "hub intencional" }
    #[serde(default)]
    pub wildcard_exceptions: HashMap<String, String>,
    /// Configuração completa V21 — lida de `[v21]`.
    #[serde(default)]
    pub v21: Option<V21TableConfig>,
    /// Configuração de escopo para V21 — lida de `[v21_scope]`.
    #[serde(default)]
    pub v21_scope: Option<V21ScopeConfig>,
    /// Allowlist de literais triviais para V21 — lida de `[v21_trivial]`.
    #[serde(default)]
    pub v21_trivial: Option<V21TrivialEntry>,
    /// Módulos com ratchet strict para V21 — lida de `[v21_strict]`.
    #[serde(default)]
    pub v21_strict: Option<V21StrictEntry>,
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
    ///
    /// Suporta dois formatos:
    /// - Legacy: `rust = ["thiserror"]` — chave é linguagem, valor é lista de
    ///   crates autorizadas crate-wide.
    /// - Type-level: `[l1_allowed_external.ecow] types = ["EcoString"]` — chave
    ///   é nome da crate, valor é lista de itens autorizados. Crate-keys sem
    ///   language explícito são atribuídos a Rust (adequado a projetos Rust-only
    ///   como typst-cristalino).
    #[serde(default)]
    pub l1_allowed_external: HashMap<String, AllowedExternalEntry>,
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

    /// Verificar imports nascidos em `#[cfg(test)]` na gravidade (V3/V9/V14) — 0061.
    /// Default **`false`** (`None` ⇒ excluir teste): a gravidade afirma o grafo de
    /// produção, e `#[cfg(test)]` é removido do build de release. A opção só **aperta**
    /// (liga o teste-como-canário), então o default tem de ser o comportamento seguro
    /// — a opção move para o menos seguro, nunca o contrário.
    #[serde(default)]
    pub check_test_imports: Option<bool>,

    #[serde(default)]
    pub analysis: AnalysisConfig,

    /// Configuração do relatório N16 — lida de .
    #[serde(default)]
    pub n16_summary: Option<N16SummaryConfig>,

    /// Contratos explícitos de preservação semântica (ADR-0018).
    #[serde(default)]
    pub semantic: SemanticContractsConfig,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub struct N16SummaryConfig {
    #[serde(default)]
    pub min_sample_size: Option<usize>,
}

impl CrystallineConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        let config: Self = toml::from_str(&content).map_err(|e| format!("Invalid TOML: {e}"))?;
        config.validate_layers()?;
        config.validate_semantic_contracts()?;
        Ok(config)
    }

    fn validate_layers(&self) -> Result<(), String> {
        const ALLOWED: &[&str] = &["L0", "L1", "L2", "L3", "L4", "lab", "Lab"];
        if self.layers.contains_key("lab") && self.layers.contains_key("Lab") {
            return Err("Invalid [layers]: `lab` and `Lab` aliases cannot coexist".to_string());
        }
        let mut directories = HashSet::new();
        for (layer, directory) in &self.layers {
            if !ALLOWED.contains(&layer.as_str()) {
                return Err(format!("Invalid [layers] key `{layer}`"));
            }
            let path = Path::new(directory);
            let valid_component = path.components().count() == 1
                && matches!(path.components().next(), Some(Component::Normal(_)));
            if directory.is_empty()
                || path.is_absolute()
                || directory.contains('/')
                || directory.contains('\\')
                || !valid_component
            {
                return Err(format!(
                    "Invalid [layers] directory `{directory}` for `{layer}`"
                ));
            }
            if !directories.insert(directory.as_str()) {
                return Err(format!(
                    "Invalid [layers]: directory `{directory}` is assigned more than once"
                ));
            }
        }
        Ok(())
    }

    fn validate_semantic_contracts(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for contract in &self.semantic.context {
            if contract.id.trim().is_empty()
                || contract.scopes.is_empty()
                || contract.sources.is_empty()
                || contract.resolvers.is_empty()
                || contract.sinks.is_empty()
            {
                return Err("Invalid [[semantic.context]]: id, scopes, sources, resolvers and sinks are required".to_string());
            }
            if !ids.insert(contract.id.as_str()) {
                return Err(format!("Duplicate semantic contract id `{}`", contract.id));
            }
        }
        for contract in &self.semantic.projection {
            if contract.id.trim().is_empty()
                || contract.scope.trim().is_empty()
                || contract.source.trim().is_empty()
                || contract.destination.trim().is_empty()
            {
                return Err("Invalid [[semantic.projection]]: id, scope, source and destination are required".to_string());
            }
            if !contract.destination.starts_with("return.") {
                return Err(format!(
                    "Unsupported semantic projection destination `{}`",
                    contract.destination
                ));
            }
            if !ids.insert(contract.id.as_str()) {
                return Err(format!("Duplicate semantic contract id `{}`", contract.id));
            }
        }
        for contract in &self.semantic.decision {
            if contract.id.trim().is_empty() || contract.owner.trim().is_empty() {
                return Err("Invalid [[semantic.decision]]: id and owner are required".to_string());
            }
            if !ids.insert(contract.id.as_str()) {
                return Err(format!("Duplicate semantic contract id `{}`", contract.id));
            }
        }
        Ok(())
    }

    /// Resolve o nível efectivo para uma regra.
    /// Se `[rules]` declara um nível para `rule_id`, esse nível é retornado.
    /// Caso contrário, retorna `default`.
    pub fn n16_min_sample_size(&self) -> usize {
        self.n16_summary
            .as_ref()
            .and_then(|s| s.min_sample_size)
            .unwrap_or(5)
    }

    pub fn level_for(&self, rule_id: &str, default: ViolationLevel) -> ViolationLevel {
        self.rules
            .get(rule_id)
            .and_then(|e| e.level.as_deref())
            .and_then(|s| match s {
                "fatal" | "Fatal" => Some(ViolationLevel::Fatal),
                "error" | "Error" => Some(ViolationLevel::Error),
                "warning" | "Warning" => Some(ViolationLevel::Warning),
                "info" | "Info" => Some(ViolationLevel::Info),
                _ => None,
            })
            .unwrap_or(default)
    }

    /// Returns the map of allowed external crates → allowed items for a given
    /// language. An empty item set means crate-wide allowance (legacy).
    /// Returns an empty map if the language is not present in the config.
    pub fn l1_allowed_for_language(&self, language: &str) -> HashMap<String, HashSet<String>> {
        let mut result = HashMap::new();
        for (key, entry) in &self.l1_allowed_external {
            match entry {
                AllowedExternalEntry::Legacy(crates) => {
                    if key == language {
                        for c in crates {
                            result.insert(c.clone(), HashSet::new());
                        }
                    }
                }
                AllowedExternalEntry::TypeLevel { types } => {
                    if is_language_key(key) {
                        // type-level por linguagem não é suportado; ignora.
                        continue;
                    }
                    // Crate-key sem language explícita: atribui a Rust.
                    if language == "rust" {
                        result.insert(key.clone(), types.iter().cloned().collect());
                    }
                }
            }
        }
        result
    }

    /// Resolve a Rust module name (e.g. "entities") to a Layer.
    /// Used by LayerResolver in RustParser.
    /// Converte a configuração de V21 para `V21RuleConfig` com os devidos defaults.
    pub fn v21_rule_config(&self) -> crate::rules::unsourced_constant::V21RuleConfig {
        let mut def = crate::rules::unsourced_constant::V21RuleConfig::default();
        if let Some(v21_table) = &self.v21 {
            if let Some(cvars) = &v21_table.context_vars {
                def.context_vars = cvars.clone();
            }
            if let Some(sinks) = &v21_table.geometric_sinks {
                def.geometric_sinks = sinks.clone();
            }
            if let Some(fmods) = &v21_table.format_syntax_modules {
                def.format_syntax_modules = fmods.clone();
            }
            if let Some(smods) = &v21_table.scope_modules {
                def.scope_modules = smods.clone();
            }
            if let Some(stys) = &v21_table.scope_types {
                def.scope_types = stys.clone();
            }
            if let Some(strict) = &v21_table.strict_modules {
                def.strict_modules = strict.clone();
            }
        }
        if let Some(scope) = &self.v21_scope {
            if let Some(mods) = &scope.modules {
                def.scope_modules = mods.clone();
            }
            if let Some(tys) = &scope.types {
                def.scope_types = tys.clone();
            }
        }
        if let Some(trivial_entry) = &self.v21_trivial {
            match trivial_entry {
                V21TrivialEntry::List(list) => {
                    for t in list {
                        def.trivial_literals.insert(t.clone());
                    }
                }
                V21TrivialEntry::Table(table) => {
                    if let Some(lits) = &table.literals {
                        for t in lits {
                            def.trivial_literals.insert(t.clone());
                        }
                    }
                }
            }
        }
        if let Some(strict_entry) = &self.v21_strict {
            match strict_entry {
                V21StrictEntry::List(list) => {
                    def.strict_modules = list.clone();
                }
                V21StrictEntry::Table(table) => {
                    if let Some(mods) = &table.modules {
                        def.strict_modules = mods.clone();
                    }
                }
            }
        }
        def
    }

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

        let mut strict_directories = Vec::new();
        strict_directories.push("01_core".to_string());
        strict_directories.push("02_shell".to_string());
        strict_directories.push("03_infra".to_string());
        strict_directories.push("04_wiring".to_string());

        Self {
            layers,
            module_layers,
            excluded,
            l1_ports,
            orphan_exceptions: HashMap::new(),
            wildcard_exceptions: HashMap::new(),
            v21: None,
            v21_scope: None,
            v21_trivial: None,
            v21_strict: None,
            wiring_exceptions: WiringExceptionsConfig::default(),
            ts_aliases: HashMap::new(),
            py_aliases: HashMap::new(),
            excluded_files: HashMap::new(),
            l1_allowed_external: HashMap::new(),
            rules: HashMap::new(),
            v11_blanket_exceptions: HashMap::new(),
            check_test_imports: None,
            n16_summary: None,
            semantic: SemanticContractsConfig::default(),
            analysis: AnalysisConfig {
                lineage: LineageConfig { strict_directories },
            },
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
    fn l1_allowed_for_language_parses_legacy_format() {
        let toml = r#"
[l1_allowed_external]
rust = ["thiserror", "serde"]
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        let allowed = config.l1_allowed_for_language("rust");
        assert!(allowed.contains_key("thiserror"));
        assert!(allowed.contains_key("serde"));
        assert!(allowed["thiserror"].is_empty());
        assert!(config.l1_allowed_for_language("python").is_empty());
    }

    #[test]
    fn l1_allowed_for_language_parses_type_level_format() {
        let toml = r#"
[l1_allowed_external.ecow]
types = ["EcoString", "EcoVec"]
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        let allowed = config.l1_allowed_for_language("rust");
        assert_eq!(allowed.get("ecow").map(|s| s.len()), Some(2));
        assert!(allowed["ecow"].contains("EcoString"));
        assert!(config.l1_allowed_for_language("python").is_empty());
    }

    #[test]
    fn level_for_returns_default_when_rules_empty() {
        let config = CrystallineConfig::default();
        assert_eq!(
            config.level_for("V11", ViolationLevel::Error),
            ViolationLevel::Error
        );
        assert_eq!(
            config.level_for("V7", ViolationLevel::Warning),
            ViolationLevel::Warning
        );
    }

    #[test]
    fn level_for_returns_configured_level_when_declared() {
        let mut config = CrystallineConfig::default();
        config.rules.insert(
            "V11".to_string(),
            RuleEntry {
                level: Some("warning".to_string()),
                languages: None,
            },
        );
        assert_eq!(
            config.level_for("V11", ViolationLevel::Error),
            ViolationLevel::Warning
        );
    }

    #[test]
    fn level_for_unknown_rule_returns_default() {
        let config = CrystallineConfig::default();
        assert_eq!(
            config.level_for("V99", ViolationLevel::Warning),
            ViolationLevel::Warning
        );
    }
    #[test]
    fn level_for_returns_info_when_configured() {
        let mut config = CrystallineConfig::default();
        config.rules.insert(
            "V19".to_string(),
            RuleEntry {
                level: Some("info".to_string()),
                languages: None,
            },
        );
        assert_eq!(
            config.level_for("V19", ViolationLevel::Warning),
            ViolationLevel::Info
        );
    }

    #[test]
    fn wildcard_exceptions_and_rule_languages_parse() {
        let toml = r#"
[rules.V16]
languages = ["rust"]
level = "warning"

[wildcard_exceptions]
"01_core/gradient.rs:221" = "hub intencional"
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        assert_eq!(
            config.rules.get("V16").unwrap().languages.as_ref().unwrap(),
            &vec!["rust"]
        );
        assert_eq!(
            config
                .wildcard_exceptions
                .get("01_core/gradient.rs:221")
                .unwrap(),
            "hub intencional"
        );
    }

    #[test]
    fn n16_summary_config_parses_min_sample_size() {
        let toml = r#"
[n16_summary]
min_sample_size = 10
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.n16_min_sample_size(), 10);
    }

    #[test]
    fn n16_summary_config_default_is_5() {
        let config = CrystallineConfig::default();
        assert_eq!(config.n16_min_sample_size(), 5);
    }

    #[test]
    fn v21_config_parses_custom_scope_and_trivial() {
        let toml = r#"
[v21_scope]
modules = ["geom/"]
types = ["Point", "Length"]

[v21_trivial]
literals = ["0", "1", "42"]

[v21_strict]
modules = ["geom/strict"]
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        let rule_cfg = config.v21_rule_config();
        assert_eq!(rule_cfg.scope_modules, vec!["geom/"]);
        assert_eq!(rule_cfg.scope_types, vec!["Point", "Length"]);
        assert!(rule_cfg.trivial_literals.contains("42"));
        assert_eq!(rule_cfg.strict_modules, vec!["geom/strict"]);
    }

    #[test]
    fn semantic_contracts_parse_and_validate() {
        let toml = r#"
[[semantic.context]]
id = "radius"
language = "rust"
scopes = ["render.rs::draw"]
sources = ["radius"]
resolvers = [{ symbol = "resolve", context_arg = 0 }]
sinks = ["draw"]
"#;
        let config: CrystallineConfig = toml::from_str(toml).unwrap();
        assert!(config.validate_semantic_contracts().is_ok());
        assert_eq!(config.semantic.context[0].resolvers[0].context_arg, 0);
    }

    #[test]
    fn duplicate_semantic_ids_are_rejected() {
        let mut config = CrystallineConfig::default();
        config.semantic.context.push(ContextSemanticContract {
            id: "same".into(),
            scopes: vec!["a::f".into()],
            sources: vec!["x".into()],
            resolvers: vec![SemanticResolverConfig {
                symbol: "resolve".into(),
                context_arg: 0,
            }],
            sinks: vec!["sink".into()],
            ..Default::default()
        });
        config.semantic.decision.push(DecisionSemanticContract {
            id: "same".into(),
            owner: "a::owner".into(),
            ..Default::default()
        });
        assert!(config
            .validate_semantic_contracts()
            .unwrap_err()
            .contains("Duplicate"));
    }

    #[test]
    fn layers_reject_unknown_alias_collision_invalid_paths_and_duplicates() {
        let invalid = [
            vec![("L5", "five")],
            vec![("lab", "lab"), ("Lab", "Lab")],
            vec![("L1", "")],
            vec![("L1", "/absolute")],
            vec![("L1", ".")],
            vec![("L1", "..")],
            vec![("L1", "nested/path")],
            vec![("L1", "nested\\path")],
            vec![("L1", "same"), ("L2", "same")],
        ];
        for entries in invalid {
            let mut config = CrystallineConfig::default();
            config.layers = entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
            assert!(config.validate_layers().is_err());
        }
        let mut alias = CrystallineConfig::default();
        alias.layers.remove("lab");
        alias.layers.insert("Lab".to_string(), "lab".to_string());
        assert!(alias.validate_layers().is_ok());
    }
}
