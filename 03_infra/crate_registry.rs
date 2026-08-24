//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/crate-registry.md
//! @prompt-hash b4ca6455
//! @layer L3
//! @updated 2026-06-06

use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::io::ErrorKind;
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

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("cannot read Cargo manifest {path}: {source}")]
    ManifestRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid Cargo manifest {path}: {detail}")]
    InvalidManifest { path: PathBuf, detail: String },
    #[error("conflicting workspace members normalize to crate name `{0}`")]
    ConflictingName(String),
    #[error("canonical member directory `{0}` belongs to different workspace members")]
    ConflictingDirectory(PathBuf),
    #[error("dependency keys `{first}` and `{second}` normalize to `{normalized}` with different definitions")]
    NormalizedDependencyCollision {
        first: String,
        second: String,
        normalized: String,
    },
}

impl CrateRegistry {
    /// Registro vazio — classificação idêntica ao legado.
    pub fn empty() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    /// Constrói a partir de uma lista de membros (puro — usado por `from_root` e testes).
    pub fn from_members(members: Vec<MemberCrate>) -> Result<Self, RegistryError> {
        let mut by_name: HashMap<String, MemberCrate> = HashMap::new();
        let mut by_dir: HashMap<PathBuf, MemberCrate> = HashMap::new();
        for member in members {
            let member = normalize_member(member)?;
            if let Some(existing) = by_name.get(&member.name) {
                if existing != &member {
                    return Err(RegistryError::ConflictingName(member.name));
                }
                continue;
            }
            if let Some(existing) = by_dir.get(&member.dir) {
                if existing != &member {
                    return Err(RegistryError::ConflictingDirectory(member.dir));
                }
                continue;
            }
            by_dir.insert(member.dir.clone(), member.clone());
            by_name.insert(member.name.clone(), member);
        }
        let mut members: Vec<MemberCrate> = by_name.into_values().collect();
        members.sort_by(|left, right| (&left.name, &left.dir).cmp(&(&right.name, &right.dir)));
        Ok(Self { members })
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
        let canonical_file = file.canonicalize().ok();
        let file = canonical_file.as_deref().unwrap_or(file);
        self.members
            .iter()
            .filter(|m| file.starts_with(&m.dir))
            .max_by_key(|m| m.dir.components().count())
    }

    /// Constrói o registro lendo o workspace do projeto-alvo (I/O).
    /// `Cargo.toml` ausente/ilegível ⇒ registro vazio (projeto não-cargo) ⇒ legado.
    pub fn from_root(root: &Path, config: &CrystallineConfig) -> Result<Self, RegistryError> {
        let canonical_root = root
            .canonicalize()
            .map_err(|source| RegistryError::ManifestRead {
                path: root.to_path_buf(),
                source,
            })?;
        let root_manifest = canonical_root.join("Cargo.toml");
        let content = match std::fs::read_to_string(&root_manifest) {
            Ok(c) => c,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::empty()),
            Err(source) => {
                return Err(RegistryError::ManifestRead {
                    path: root_manifest,
                    source,
                })
            }
        };
        let value: toml::Value =
            toml::from_str(&content).map_err(|error| RegistryError::InvalidManifest {
                path: root_manifest.clone(),
                detail: error.to_string(),
            })?;

        // Diretórios relativos dos membros (workspace) ou a própria raiz (package único).
        let member_dirs = match workspace_member_patterns(&value, &root_manifest)? {
            Some(patterns) => patterns
                .iter()
                .map(|p| expand_member_pattern(&canonical_root, p))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            None => vec![canonical_root.clone()],
        };

        let mut members = Vec::new();
        for dir in member_dirs {
            let manifest = dir.join("Cargo.toml");
            let text = std::fs::read_to_string(&manifest).map_err(|source| {
                RegistryError::ManifestRead {
                    path: manifest.clone(),
                    source,
                }
            })?;
            let info = parse_manifest(&text).map_err(|error| match error {
                RegistryError::InvalidManifest { detail, .. } => RegistryError::InvalidManifest {
                    path: manifest.clone(),
                    detail,
                },
                other => other,
            })?;
            let name = info.name.ok_or_else(|| RegistryError::InvalidManifest {
                path: manifest.clone(),
                detail: "workspace member requires [package].name".to_string(),
            })?;
            let dir = dir
                .canonicalize()
                .map_err(|source| RegistryError::ManifestRead {
                    path: dir.clone(),
                    source,
                })?;
            members.push(MemberCrate {
                name,
                layer: resolve_file_layer(&dir, &canonical_root, config),
                dir,
                deps: info.deps,
                renames: info.renames,
            });
        }

        Self::from_members(members)
    }
}

fn normalize_member(mut member: MemberCrate) -> Result<MemberCrate, RegistryError> {
    member.name = normalize(&member.name);
    member.dir = member.dir.canonicalize().unwrap_or(member.dir);

    let mut dependency_keys: HashMap<String, String> = HashMap::new();
    let mut deps = HashSet::new();
    let mut raw_dependencies: Vec<String> = member.deps.into_iter().collect();
    raw_dependencies.sort();
    for dependency in raw_dependencies {
        let normalized = normalize(dependency.as_str());
        let previous = dependency_keys.insert(normalized.clone(), dependency.clone());
        if let Some(first) = previous {
            if first != dependency {
                return Err(RegistryError::NormalizedDependencyCollision {
                    first,
                    second: dependency,
                    normalized,
                });
            }
        }
        deps.insert(normalized);
    }

    let mut renames = HashMap::new();
    let mut rename_definitions: HashMap<String, (String, String)> = HashMap::new();
    let mut raw_renames: Vec<(String, String)> = member.renames.into_iter().collect();
    raw_renames.sort();
    for (raw_key, raw_target) in raw_renames {
        let key = normalize(&raw_key);
        let target = normalize(&raw_target);
        if let Some((first, existing)) = rename_definitions.get(&key) {
            if existing != &target {
                return Err(RegistryError::NormalizedDependencyCollision {
                    first: first.clone(),
                    second: raw_key,
                    normalized: key,
                });
            }
        } else {
            rename_definitions.insert(key.clone(), (raw_key, target.clone()));
            renames.insert(key, target);
        }
    }
    member.deps = deps;
    member.renames = renames;
    Ok(member)
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

fn dependency_definition(value: &toml::Value) -> String {
    let mut canonical = value.clone();
    if let Some(table) = canonical.as_table_mut() {
        if let Some(package) = table.get_mut("package") {
            if let Some(name) = package.as_str() {
                *package = toml::Value::String(normalize(name));
            }
        }
    }
    canonical.to_string()
}

/// Extrai nome do pacote e deps declaradas de um `Cargo.toml`.
pub fn parse_manifest(content: &str) -> Result<ManifestInfo, RegistryError> {
    let value: toml::Value =
        toml::from_str(content).map_err(|error| RegistryError::InvalidManifest {
            path: PathBuf::new(),
            detail: error.to_string(),
        })?;

    let name = value
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(normalize);

    let mut definitions: HashMap<String, (String, String)> = HashMap::new();
    let mut deps = HashSet::new();
    let mut renames = HashMap::new();
    for table in ["dependencies", "dev-dependencies"] {
        if let Some(t) = value.get(table).and_then(|d| d.as_table()) {
            for (key, val) in t.iter() {
                let key_norm = normalize(key);
                let definition = dependency_definition(val);
                match definitions.entry(key_norm.clone()) {
                    Entry::Occupied(existing) if existing.get().1 != definition => {
                        return Err(RegistryError::NormalizedDependencyCollision {
                            first: existing.get().0.clone(),
                            second: key.clone(),
                            normalized: key_norm,
                        });
                    }
                    Entry::Occupied(_) => {}
                    Entry::Vacant(entry) => {
                        entry.insert((key.clone(), definition));
                    }
                }
                deps.insert(key_norm.clone());
                // `chave = { package = "real" }` → renomeação por-membro.
                if let Some(pkg) = val.get("package").and_then(|p| p.as_str()) {
                    renames.insert(key_norm, normalize(pkg));
                }
            }
        }
    }

    Ok(ManifestInfo {
        name,
        deps,
        renames,
    })
}

/// Padrões de membros de um workspace (`[workspace].members`), ou `None` se não houver workspace.
fn workspace_member_patterns(
    value: &toml::Value,
    path: &Path,
) -> Result<Option<Vec<String>>, RegistryError> {
    let Some(workspace) = value.get("workspace") else {
        return Ok(None);
    };
    let Some(members) = workspace.get("members") else {
        return Ok(None);
    };
    let members = members
        .as_array()
        .ok_or_else(|| RegistryError::InvalidManifest {
            path: path.to_path_buf(),
            detail: "workspace.members must be an array".to_string(),
        })?;
    members
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(String::from)
                .ok_or_else(|| RegistryError::InvalidManifest {
                    path: path.to_path_buf(),
                    detail: "workspace.members entries must be strings".to_string(),
                })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// Expande um padrão de membro para diretórios concretos.
/// Suporta path exato e um `*` final (ex.: "crates/*").
fn expand_member_pattern(root: &Path, pattern: &str) -> Result<Vec<PathBuf>, RegistryError> {
    if let Some(prefix) = pattern
        .strip_suffix("/*")
        .or_else(|| pattern.strip_suffix('*').map(|p| p.trim_end_matches('/')))
    {
        let base = root.join(prefix);
        let entries = std::fs::read_dir(&base).map_err(|source| RegistryError::ManifestRead {
            path: base.clone(),
            source,
        })?;
        Ok(entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.join("Cargo.toml").is_file())
            .collect())
    } else {
        Ok(vec![root.join(pattern)])
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
        ])
        .unwrap();
        assert_eq!(reg.member_layer("lente_wiring"), Some(Layer::L4));
        assert_eq!(reg.member_layer("lente_core"), Some(Layer::L1));
    }

    #[test]
    fn member_layer_unknown_for_non_member() {
        let reg =
            CrateRegistry::from_members(vec![member("lente_core", "/p/core", Layer::L1, &[])])
                .unwrap();
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
        ])
        .unwrap();
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
        ])
        .unwrap();
        let owner = reg
            .owner_of(Path::new("/p/crates/nested/src/x.rs"))
            .unwrap();
        assert_eq!(owner.name, "nested");
    }

    #[test]
    fn owner_of_none_when_outside_all_members() {
        let reg = CrateRegistry::from_members(vec![member("a", "/p/a", Layer::L1, &[])]).unwrap();
        assert!(reg.owner_of(Path::new("/other/x.rs")).is_none());
    }

    #[test]
    fn identical_members_deduplicate_but_conflicts_fail() {
        let same = member("same", "/p/same", Layer::L1, &["serde"]);
        let registry = CrateRegistry::from_members(vec![same.clone(), same]).unwrap();
        assert_eq!(registry.member_layer("same"), Some(Layer::L1));
        let error = CrateRegistry::from_members(vec![
            member("foo-bar", "/p/one", Layer::L1, &[]),
            member("foo_bar", "/p/two", Layer::L2, &[]),
        ])
        .unwrap_err();
        assert!(matches!(error, RegistryError::ConflictingName(_)));
    }

    #[test]
    fn normalized_dependency_collision_is_structural_error() {
        let error =
            parse_manifest("[package]\nname='x'\n[dependencies]\nfoo-bar='1'\nfoo_bar='2'\n")
                .unwrap_err();
        assert!(matches!(
            error,
            RegistryError::NormalizedDependencyCollision { .. }
        ));
    }

    #[test]
    fn normalized_dependency_duplicates_with_same_meaning_deduplicate() {
        let info = parse_manifest(
            "[package]\nname='x'\n[dependencies]\nfoo-bar={ package='real-name', version='1' }\nfoo_bar={ package='real_name', version='1' }\n",
        )
        .unwrap();
        assert_eq!(info.deps.len(), 1);
        assert_eq!(info.renames.get("foo_bar"), Some(&"real_name".to_string()));
    }

    #[test]
    fn direct_members_reject_normalized_dependency_and_rename_conflicts() {
        let mut dependency_collision = member("x", "/p/x", Layer::L1, &["foo-bar", "foo_bar"]);
        assert!(matches!(
            CrateRegistry::from_members(vec![dependency_collision.clone()]).unwrap_err(),
            RegistryError::NormalizedDependencyCollision { .. }
        ));

        dependency_collision.deps.clear();
        dependency_collision
            .renames
            .insert("dep-x".to_string(), "target-one".to_string());
        dependency_collision
            .renames
            .insert("dep_x".to_string(), "target-two".to_string());
        assert!(matches!(
            CrateRegistry::from_members(vec![dependency_collision]).unwrap_err(),
            RegistryError::NormalizedDependencyCollision { .. }
        ));
    }

    #[test]
    fn absent_root_manifest_is_not_cargo_but_invalid_manifest_is_error() {
        let root = tempfile::tempdir().unwrap();
        let registry =
            CrateRegistry::from_root(root.path(), &CrystallineConfig::default()).unwrap();
        assert!(registry.member_layer("anything").is_none());

        std::fs::write(root.path().join("Cargo.toml"), "[workspace\n").unwrap();
        let error =
            CrateRegistry::from_root(root.path(), &CrystallineConfig::default()).unwrap_err();
        assert!(matches!(error, RegistryError::InvalidManifest { .. }));
    }
}
