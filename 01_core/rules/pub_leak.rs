//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/pub-leak.md
//! @prompt-hash 8d146498
//! @layer L1
//! @updated 2026-03-15

use std::borrow::Cow;
use std::collections::HashSet;

use crate::entities::rule_traits::HasPubLeak;
use crate::entities::layer::Layer;
use crate::entities::violation::{Location, Violation, ViolationLevel};

// ── L1Ports ───────────────────────────────────────────────────────────────────

/// Conjunto de subdiretórios de L1 acessíveis externamente.
/// Declarados em crystalline.toml [l1_ports], construídos por L3 (config),
/// injetados em V9 via L4. V9 nunca lê o toml diretamente.
pub struct L1Ports {
    ports: HashSet<String>,
}

impl L1Ports {
    pub fn new(ports: HashSet<String>) -> Self {
        Self { ports }
    }

    pub fn contains(&self, subdir: &str) -> bool {
        self.ports.contains(subdir)
    }
}

// ── V9 check ──────────────────────────────────────────────────────────────────

/// V9 — Pub Leak (Solução Preguiçosa).
///
/// Imports de L2 ou L3 apontando para subdiretório interno de L1
/// não listado em [l1_ports] geram violação Error.
///
/// Aplica-se apenas a L2 e L3 importando L1.
/// L1 importando L1 não é V9 (é V3 se inválido).
/// L4 pode importar qualquer porta sem restrição de V9.
///
/// Inspeciona Import.target_subdir — resolvido por L3 (RustParser).
/// target_subdir = None → crate externa → não é V9.
/// target_subdir = Some(subdir) não em ports → V9 Error.
pub fn check<'a, T: HasPubLeak<'a>>(file: &T, ports: &L1Ports) -> Vec<Violation<'a>> {
    if !matches!(file.layer(), Layer::L2 | Layer::L3) {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| import.target_layer == Layer::L1)
        .filter(|import| {
            import
                .target_subdir
                .map(|subdir| !ports.contains(subdir))
                .unwrap_or(false)
        })
        .map(|import| Violation {
            rule_id: "V9".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Vazamento de encapsulamento: import '{}' acessa \
                 subdiretório interno de L1. \
                 Use apenas as portas declaradas em [l1_ports].",
                import.path
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::parsed_file::{Import, ImportKind};
    use std::path::Path;

    struct MockFile {
        layer: Layer,
        imports: Vec<Import<'static>>,
        path: &'static Path,
    }

    impl HasPubLeak<'static> for MockFile {
        fn layer(&self) -> &Layer { &self.layer }
        fn imports(&self) -> &[Import<'static>] { &self.imports }
        fn path(&self) -> &'static Path { self.path }
    }

    fn base_file(layer: Layer) -> MockFile {
        MockFile { layer, imports: vec![], path: Path::new("src/foo.rs") }
    }

    fn default_ports() -> L1Ports {
        L1Ports::new(
            ["entities".to_string(), "contracts".to_string(), "rules".to_string()]
                .into_iter()
                .collect(),
        )
    }

    fn import_to_l1(path: &'static str, line: usize, subdir: Option<&'static str>) -> Import<'static> {
        Import {
            path,
            line,
            kind: ImportKind::Use,
            target_layer: Layer::L1,
            target_subdir: subdir,
        }
    }

    #[test]
    fn l2_importing_internal_subdir_returns_v9() {
        let mut file = base_file(Layer::L2);
        file.imports.push(import_to_l1("crate::core::internal::helper", 5, Some("internal")));
        let violations = check(&file, &default_ports());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V9");
        assert_eq!(violations[0].level, ViolationLevel::Error);
        assert_eq!(violations[0].location.line, 5);
    }

    #[test]
    fn l2_importing_entities_port_returns_empty() {
        let mut file = base_file(Layer::L2);
        file.imports.push(import_to_l1("crate::entities::Layer", 3, Some("entities")));
        assert!(check(&file, &default_ports()).is_empty());
    }

    #[test]
    fn l3_importing_contracts_port_returns_empty() {
        let mut file = base_file(Layer::L3);
        file.imports.push(import_to_l1("crate::contracts::FileProvider", 7, Some("contracts")));
        assert!(check(&file, &default_ports()).is_empty());
    }

    #[test]
    fn l1_file_is_exempt_from_v9() {
        let mut file = base_file(Layer::L1);
        file.imports.push(import_to_l1("crate::core::internal::foo", 1, Some("internal")));
        assert!(check(&file, &default_ports()).is_empty());
    }

    #[test]
    fn l4_file_is_exempt_from_v9() {
        let mut file = base_file(Layer::L4);
        file.imports.push(import_to_l1("crate::core::internal::foo", 1, Some("internal")));
        assert!(check(&file, &default_ports()).is_empty());
    }

    #[test]
    fn external_crate_target_subdir_none_is_not_v9() {
        let mut file = base_file(Layer::L2);
        file.imports.push(Import {
            path: "reqwest::Client",
            line: 2,
            kind: ImportKind::Use,
            target_layer: Layer::Unknown,
            target_subdir: None,
        });
        assert!(check(&file, &default_ports()).is_empty());
    }

    #[test]
    fn violation_message_contains_import_path() {
        let mut file = base_file(Layer::L2);
        file.imports.push(import_to_l1("crate::core::secret::impl_detail", 10, Some("secret")));
        let violations = check(&file, &default_ports());
        assert!(violations[0].message.contains("crate::core::secret::impl_detail"));
    }
}
