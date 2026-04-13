//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/external-type-in-contract.md
//! @prompt-hash 814afa50
//! @layer L1
//! @updated 2026-03-22

use std::borrow::Cow;

use crate::entities::l1_allowed_external::L1AllowedExternal;
use crate::entities::layer::Layer;
use crate::entities::rule_traits::HasImports;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V14 — External Type In Contract.
///
/// L1 é fechada por defeito para dependências externas. Apenas
/// pacotes explicitamente autorizados em `[l1_allowed_external]`
/// são permitidos. Stdlib (std, core, alloc) está sempre isenta.
///
/// Error — aplica-se apenas a arquivos com `layer == L1`.
pub fn check<'a, T: HasImports<'a>>(file: &T, allowed: &L1AllowedExternal) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| import.target_layer == Layer::Unknown)
        .filter(|import| !allowed.is_allowed(package_name(import.path)))
        .map(|import| Violation {
            rule_id: "V14".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Dependência externa não autorizada em L1: '{}' não está em \
                 [l1_allowed_external]. Adicionar ao crystalline.toml se necessário, \
                 ou mover a dependência para L3.",
                package_name(import.path),
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}

fn package_name(import_path: &str) -> &str {
    // Rust: "serde::Serialize" → "serde"
    //       "std::collections::HashMap" → "std" (isento)
    // TypeScript e Python: o path já é o nome do pacote
    import_path
        .split("::")
        .next()
        .unwrap_or(import_path)
        .split('/')
        .next()
        .unwrap_or(import_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::Path;

    use crate::entities::layer::Layer;
    use crate::entities::parsed_file::{Import, ImportKind};

    struct MockFile {
        layer: Layer,
        imports: Vec<Import<'static>>,
        path: &'static Path,
    }

    impl HasImports<'static> for MockFile {
        fn layer(&self) -> &Layer { &self.layer }
        fn imports(&self) -> &[Import<'static>] { &self.imports }
        fn path(&self) -> &'static Path { self.path }
    }

    fn l1_file_with(imports: Vec<Import<'static>>) -> MockFile {
        MockFile { layer: Layer::L1, imports, path: Path::new("01_core/foo.rs") }
    }

    fn l3_file_with(imports: Vec<Import<'static>>) -> MockFile {
        MockFile { layer: Layer::L3, imports, path: Path::new("03_infra/foo.rs") }
    }

    fn external_import(path: &'static str, line: usize) -> Import<'static> {
        Import {
            path,
            line,
            kind: ImportKind::Direct,
            target_layer: Layer::Unknown,
            target_subdir: None,
        }
    }

    fn whitelist(packages: &[&str]) -> L1AllowedExternal {
        let mut set = HashSet::new();
        for p in packages {
            set.insert(p.to_string());
        }
        L1AllowedExternal::for_rust(set)
    }

    #[test]
    fn unlisted_external_in_l1_triggers_v14() {
        let file = l1_file_with(vec![external_import("comemo::Tracked", 3)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V14");
        assert_eq!(violations[0].level, ViolationLevel::Error);
        assert!(violations[0].message.contains("comemo"));
    }

    #[test]
    fn listed_external_in_l1_is_allowed() {
        let file = l1_file_with(vec![external_import("thiserror::Error", 5)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn std_import_with_empty_whitelist_is_allowed() {
        // std is always exempt (stdlib)
        let file = l1_file_with(vec![external_import("std::collections::HashMap", 2)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn core_import_with_empty_whitelist_is_allowed() {
        let file = l1_file_with(vec![external_import("core::fmt::Display", 2)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn tokio_not_in_whitelist_triggers_v14() {
        let file = l1_file_with(vec![external_import("tokio::sync::Mutex", 8)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("tokio"));
    }

    #[test]
    fn serde_with_empty_whitelist_triggers_v14() {
        let file = l1_file_with(vec![external_import("serde::Serialize", 4)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("serde"));
    }

    #[test]
    fn serde_in_whitelist_is_allowed() {
        let file = l1_file_with(vec![external_import("serde::Serialize", 4)]);
        let allowed = whitelist(&["serde", "thiserror"]);
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn l3_file_with_external_import_is_ignored() {
        let file = l3_file_with(vec![external_import("rayon::prelude", 1)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn l1_without_external_imports_returns_empty() {
        let file = l1_file_with(vec![]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert!(violations.is_empty());
    }

    #[test]
    fn two_unlisted_externals_produce_two_violations() {
        let file = l1_file_with(vec![
            external_import("comemo::Tracked", 3),
            external_import("tokio::runtime::Runtime", 7),
        ]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn package_name_extracts_first_segment() {
        assert_eq!(super::package_name("serde::Serialize"), "serde");
        assert_eq!(super::package_name("std::collections::HashMap"), "std");
        assert_eq!(super::package_name("tokio"), "tokio");
    }
}
