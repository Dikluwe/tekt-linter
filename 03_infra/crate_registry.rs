//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/crate-registry.md
//! @prompt-hash b4ca6455
//! @layer L3
//! @updated 2026-06-06

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::entities::layer::Layer;
use crate::infra::config::CrystallineConfig;
use crate::infra::walker::resolve_file_layer;

// ── MemberCrate ────────────────────────────────────────────────────────────────

/// Um crate-membro do workspace do projeto-alvo.
/// `name` e `deps` são normalizados (`-` → `_`) para casar com paths de `use`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberCrate {
    /// Nome do pacote, normalizado `-`→`_` (ex.: "lente_core").
    pub name: String,
    /// Diretório do membro (root.join(membro relativo)).
    pub dir: PathBuf,
    /// Camada do membro, via `resolve_file_layer` sobre `dir` (mesma lógica `[layers]`).
    pub layer: Layer,
    /// Dependências declaradas (`[dependencies]` + `[dev-dependencies]`),
    /// normalizadas `-`→`_`. Distinguem externo real de item local.
    pub deps: HashSet<String>,
    /// Renomeações deste membro: chave de import → pacote real (cego #3, 0059).
    /// `classify_import` resolve a chave através deste mapa antes de cair em `Unknown`.
    pub renames: HashMap<String, String>,
}

// ── CrateRegistry ──────────────────────────────────────────────────────────────

/// Registro imutável dos membros do workspace. Construído uma vez (I/O em
/// `Cargo.toml`) e partilhado por `&self` no pipeline — sem estado mutável.
#[derive(Debug, Clone, Default)]
pub struct CrateRegistry {
    members: Vec<MemberCrate>,
}

impl CrateRegistry {
    /// Registro vazio — classificação idêntica ao legado.
    pub fn empty() -> Self {
        Self { members: Vec::new() }
    }

    /// Constrói a partir de uma lista de membros (puro — usado por `from_root` e testes).
    pub fn from_members(members: Vec<MemberCrate>) -> Self {
        Self { members }
    }

    /// Camada de um membro first-party pelo nome do pacote (normalizado).
    pub fn member_layer(&self, name: &str) -> Option<Layer> {
        self.members
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.layer.clone())
    }

    /// O membro dono de um ficheiro: aquele cujo `dir` é ancestral do path.
    /// Prefixo mais longo vence (membro aninhado ganha da raiz).
    pub fn owner_of(&self, file: &Path) -> Option<&MemberCrate> {
        self.members
            .iter()
            .filter(|m| file.starts_with(&m.dir))
            .max_by_key(|m| m.dir.components().count())
    }

    /// Constrói o registro lendo o workspace do projeto-alvo (I/O).
    /// `Cargo.toml` ausente/ilegível ⇒ registro vazio (projeto não-cargo) ⇒ legado.
    pub fn from_root(root: &Path, config: &CrystallineConfig) -> Self {
        let root_manifest = root.join("Cargo.toml");
        let content = match std::fs::read_to_string(&root_manifest) {
            Ok(c) => c,
            Err(_) => return Self::empty(),
        };
        let value: toml::Value = match toml::from_str(&content) {
            Ok(v) => v,
            Err(_) => return Self::empty(),
        };

        // Diretórios relativos dos membros (workspace) ou a própria raiz (package único).
        let member_dirs = match workspace_member_patterns(&value) {
            Some(patterns) => patterns
                .iter()
                .flat_map(|p| expand_member_pattern(root, p))
                .collect::<Vec<_>>(),
            None => vec![root.to_path_buf()],
        };

        let mut members = Vec::new();
        for dir in member_dirs {
            let manifest = dir.join("Cargo.toml");
            let Ok(text) = std::fs::read_to_string(&manifest) else { continue };
            let Ok(info) = parse_manifest(&text) else { continue };
            let Some(name) = info.name else { continue };
            members.push(MemberCrate {
                name,
                layer: resolve_file_layer(&dir, root, config),
                dir,
                deps: info.deps,
                renames: info.renames,
            });
        }

        Self { members }
    }
}

// ── Manifest parsing (puro, testável) ──────────────────────────────────────────

/// Informação extraída de um `Cargo.toml` de membro.
#[derive(Debug, PartialEq, Eq)]
pub struct ManifestInfo {
    /// `[package].name` normalizado `-`→`_`. `None` se ausente (ex.: manifesto de workspace puro).
    pub name: Option<String>,
    /// União de `[dependencies]` e `[dev-dependencies]`, normalizadas `-`→`_`.
    pub deps: HashSet<String>,
    /// Renomeações por-membro: chave de dependência → pacote real, ambos `-`→`_`.
    /// Lido de `chave = { package = "real" }` (cego #3, 0059). Só entradas renomeadas.
    pub renames: HashMap<String, String>,
}

/// Normaliza um nome de crate para a forma usada em paths `use` (`-` → `_`).
fn normalize(name: &str) -> String {
    name.replace('-', "_")
}

/// Extrai nome do pacote e deps declaradas de um `Cargo.toml`.
pub fn parse_manifest(content: &str) -> Result<ManifestInfo, toml::de::Error> {
    let value: toml::Value = toml::from_str(content)?;

    let name = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(normalize);

    let mut deps = HashSet::new();
    let mut renames = HashMap::new();
    for table in ["dependencies", "dev-dependencies"] {
        if let Some(t) = value.get(table).and_then(|d| d.as_table()) {
            for (key, val) in t.iter() {
                let key_norm = normalize(key);
                deps.insert(key_norm.clone());
                // `chave = { package = "real" }` → renomeação por-membro.
                if let Some(pkg) = val.get("package").and_then(|p| p.as_str()) {
                    renames.insert(key_norm, normalize(pkg));
                }
            }
        }
    }

    Ok(ManifestInfo { name, deps, renames })
}

/// Padrões de membros de um workspace (`[workspace].members`), ou `None` se não houver workspace.
fn workspace_member_patterns(value: &toml::Value) -> Option<Vec<String>> {
    let members = value.get("workspace")?.get("members")?.as_array()?;
    Some(
        members
            .iter()
            .filter_map(|m| m.as_str().map(String::from))
            .collect(),
    )
}

/// Expande um padrão de membro para diretórios concretos.
/// Suporta path exato e um `*` final (ex.: "crates/*").
fn expand_member_pattern(root: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Some(prefix) = pattern.strip_suffix("/*").or_else(|| pattern.strip_suffix('*').map(|p| p.trim_end_matches('/'))) {
        let base = root.join(prefix);
        let Ok(entries) = std::fs::read_dir(&base) else { return Vec::new() };
        entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.join("Cargo.toml").is_file())
            .collect()
    } else {
        vec![root.join(pattern)]
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, dir: &str, layer: Layer, deps: &[&str]) -> MemberCrate {
        MemberCrate {
            name: name.to_string(),
            dir: PathBuf::from(dir),
            layer,
            deps: deps.iter().map(|s| s.to_string()).collect(),
            renames: HashMap::new(),
        }
    }

    // ── parse_manifest ──────────────────────────────────────────────────────

    #[test]
    fn parse_manifest_extracts_name_normalized() {
        let info = parse_manifest("[package]\nname = \"lente-core\"\n").unwrap();
        assert_eq!(info.name, Some("lente_core".to_string()));
    }

    #[test]
    fn parse_manifest_unions_deps_and_dev_deps_normalized() {
        let toml = "[package]\nname = \"a\"\n\
                    [dependencies]\nserde = \"1\"\ntree-sitter = \"0.23\"\n\
                    [dev-dependencies]\nlente-filtro = { path = \"../filtro\" }\n";
        let info = parse_manifest(toml).unwrap();
        assert!(info.deps.contains("serde"));
        assert!(info.deps.contains("tree_sitter"));
        assert!(info.deps.contains("lente_filtro")); // dev-dep conta (caso 0050)
    }

    #[test]
    fn parse_manifest_workspace_only_has_no_name() {
        // Manifesto de workspace puro — sem [package].
        let info = parse_manifest("[workspace]\nmembers = [\"a\", \"b\"]\n").unwrap();
        assert_eq!(info.name, None);
    }

    // ── member_layer ────────────────────────────────────────────────────────

    #[test]
    fn member_layer_resolves_first_party_by_name() {
        let reg = CrateRegistry::from_members(vec![
            member("lente_core", "/p/core", Layer::L1, &[]),
            member("lente_wiring", "/p/wiring", Layer::L4, &[]),
        ]);
        assert_eq!(reg.member_layer("lente_wiring"), Some(Layer::L4));
        assert_eq!(reg.member_layer("lente_core"), Some(Layer::L1));
    }

    #[test]
    fn member_layer_unknown_for_non_member() {
        let reg = CrateRegistry::from_members(vec![member("lente_core", "/p/core", Layer::L1, &[])]);
        assert_eq!(reg.member_layer("serde"), None);
    }

    #[test]
    fn empty_registry_has_no_members() {
        let reg = CrateRegistry::empty();
        assert_eq!(reg.member_layer("anything"), None);
        assert!(reg.owner_of(Path::new("/p/core/src/lib.rs")).is_none());
    }

    // ── owner_of ────────────────────────────────────────────────────────────

    #[test]
    fn owner_of_matches_containing_member() {
        let reg = CrateRegistry::from_members(vec![
            member("lente_core", "/p/core", Layer::L1, &["serde"]),
            member("lente_wiring", "/p/wiring", Layer::L4, &[]),
        ]);
        let owner = reg.owner_of(Path::new("/p/core/src/lib.rs")).unwrap();
        assert_eq!(owner.name, "lente_core");
        assert!(owner.deps.contains("serde"));
    }

    #[test]
    fn owner_of_longest_prefix_wins_for_nested_member() {
        // Raiz cobre tudo; membro aninhado deve ganhar.
        let reg = CrateRegistry::from_members(vec![
            member("root", "/p", Layer::Unknown, &[]),
            member("nested", "/p/crates/nested", Layer::L3, &[]),
        ]);
        let owner = reg.owner_of(Path::new("/p/crates/nested/src/x.rs")).unwrap();
        assert_eq!(owner.name, "nested");
    }

    #[test]
    fn owner_of_none_when_outside_all_members() {
        let reg = CrateRegistry::from_members(vec![member("a", "/p/a", Layer::L1, &[])]);
        assert!(reg.owner_of(Path::new("/other/x.rs")).is_none());
    }
}
