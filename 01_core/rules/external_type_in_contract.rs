//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/external-type-in-contract.md
//! @prompt-hash 5952cd0b
//! @layer L1
//! @updated 2026-06-24

use std::borrow::Cow;

use crate::entities::l1_allowed_external::L1AllowedExternal;
use crate::entities::layer::Layer;
use crate::entities::rule_traits::HasImports;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V14 — External Type In Contract.
///
/// L1 é fechada por defeito para dependências externas. Apenas
/// pacotes (e, quando configurado, itens) explicitamente autorizados em
/// `[l1_allowed_external]` são permitidos. Stdlib (std, core, alloc) está
/// sempre isenta.
///
/// Error — aplica-se apenas a arquivos com `layer == L1`.
///
/// `check_test_imports` (default false, 0061): imports externos nascidos em
/// `#[cfg(test)]` são pulados — um dev-dep só de teste não contamina o contrato
/// de produção de L1.
pub fn check<'a, T: HasImports<'a>>(
    file: &T,
    allowed: &L1AllowedExternal,
    check_test_imports: bool,
) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| check_test_imports || !import.is_test_origin)
        .filter(|import| import.target_layer == Layer::Unknown)
        .filter(|import| !import_is_allowed(import.path, allowed))
        .map(|import| Violation {
            rule_id: "V14".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Dependência externa não autorizada em L1: '{}' não está em \
                 [l1_allowed_external]. Adicionar ao crystalline.toml se necessário, \
                 ou mover a dependência para L3.",
                import.path,
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}

fn import_is_allowed(import_path: &str, allowed: &L1AllowedExternal) -> bool {
    let crate_name = package_name(import_path);
    if !allowed.is_crate_allowed(crate_name) {
        return false;
    }
    // Crate autorizado: verifica cada item importado. Conjunto vazio de itens
    // autorizados (legacy) permite qualquer item.
    imported_items(import_path)
        .iter()
        .all(|item| allowed.is_allowed(crate_name, Some(item)))
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

/// Extrai os itens importados de um path de use ou referência qualificada.
///
/// Exemplos:
/// - `ecow::EcoString` → `["EcoString"]`
/// - `ecow::{EcoString, EcoVec}` → `["EcoString", "EcoVec"]`
/// - `ecow::*` → `["*"]` (glob; rejeitado a menos que whitelist seja vazia)
/// - `hayagriva::citationberg::IndependentStyle` → `["citationberg::IndependentStyle"]`
/// - `hayagriva::{Entry, citationberg::IndependentStyle}` → `["Entry", "citationberg::IndependentStyle"]`
fn imported_items(import_path: &str) -> Vec<String> {
    let path = import_path.trim_start_matches('{').trim();
    let path = path.split(" as ").next().unwrap_or(path).trim();
    let segments: Vec<&str> = path.split("::").collect();
    if segments.len() < 2 {
        return vec![];
    }

    let rest = segments[1..].join("::");

    if rest == "*" {
        // `crate::*` — glob da raiz; tratado como item especial.
        return vec!["*".to_string()];
    }

    if rest.ends_with("::*") {
        // `crate::module::*` — glob de módulo; verifica o prefixo do módulo.
        let prefix = rest.trim_end_matches("::*");
        return vec![prefix.to_string()];
    }

    // `crate::{A, B}` ou `crate::module::{A, B}`.
    if let Some(brace_open) = rest.find('{') {
        if rest.ends_with('}') {
            let prefix = rest[..brace_open].trim_end_matches("::");
            let inner = &rest[brace_open + 1..rest.len() - 1];
            return inner
                .split(',')
                .map(|s| {
                    let s = s.trim();
                    let s = s.split(" as ").next().unwrap_or(s).trim();
                    if prefix.is_empty() {
                        s.to_string()
                    } else {
                        format!("{}::{}", prefix, s)
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    vec![rest]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
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
            is_test_origin: false,
        }
    }

    fn whitelist(packages: &[&str]) -> L1AllowedExternal {
        let mut map = HashMap::new();
        for p in packages {
            map.insert(p.to_string(), HashSet::new());
        }
        L1AllowedExternal::for_rust(map)
    }

    fn whitelist_type_level(items: &[(&str, &[&str])]) -> L1AllowedExternal {
        let mut map = HashMap::new();
        for (crate_name, types) in items {
            let set: HashSet<String> = types.iter().map(|s| s.to_string()).collect();
            map.insert(crate_name.to_string(), set);
        }
        L1AllowedExternal::for_rust(map)
    }

    #[test]
    fn unlisted_external_in_l1_triggers_v14() {
        let file = l1_file_with(vec![external_import("comemo::Tracked", 3)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V14");
        assert_eq!(violations[0].level, ViolationLevel::Error);
        assert!(violations[0].message.contains("comemo::Tracked"));
    }

    #[test]
    fn listed_external_in_l1_is_allowed() {
        let file = l1_file_with(vec![external_import("thiserror::Error", 5)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn type_level_unlisted_item_from_allowed_crate_triggers_v14() {
        let file = l1_file_with(vec![external_import("ecow::EcoMap", 7)]);
        let allowed = whitelist_type_level(&[("ecow", &["EcoString", "EcoVec"])]);
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ecow::EcoMap"));
    }

    #[test]
    fn type_level_listed_item_from_allowed_crate_is_allowed() {
        let file = l1_file_with(vec![external_import("ecow::EcoString", 7)]);
        let allowed = whitelist_type_level(&[("ecow", &["EcoString", "EcoVec"])]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn type_level_named_import_allows_only_listed_items() {
        let file = l1_file_with(vec![external_import("ecow::{EcoString, EcoMap}", 7)]);
        let allowed = whitelist_type_level(&[("ecow", &["EcoString", "EcoVec"])]);
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("ecow::{EcoString, EcoMap}"));
    }

    #[test]
    fn type_level_nested_path_matches_last_segment() {
        let file = l1_file_with(vec![external_import("hayagriva::citationberg::IndependentStyle", 9)]);
        let allowed = whitelist_type_level(&[("hayagriva", &["Entry", "IndependentStyle"])]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn std_import_with_empty_whitelist_is_allowed() {
        // std is always exempt (stdlib)
        let file = l1_file_with(vec![external_import("std::collections::HashMap", 2)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn core_import_with_empty_whitelist_is_allowed() {
        let file = l1_file_with(vec![external_import("core::fmt::Display", 2)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn tokio_not_in_whitelist_triggers_v14() {
        let file = l1_file_with(vec![external_import("tokio::sync::Mutex", 8)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("tokio"));
    }

    #[test]
    fn serde_with_empty_whitelist_triggers_v14() {
        let file = l1_file_with(vec![external_import("serde::Serialize", 4)]);
        let allowed = L1AllowedExternal::empty_for_rust();
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("serde"));
    }

    #[test]
    fn serde_in_whitelist_is_allowed() {
        let file = l1_file_with(vec![external_import("serde::Serialize", 4)]);
        let allowed = whitelist(&["serde", "thiserror"]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn l3_file_with_external_import_is_ignored() {
        let file = l3_file_with(vec![external_import("rayon::prelude", 1)]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn l1_without_external_imports_returns_empty() {
        let file = l1_file_with(vec![]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn two_unlisted_externals_produce_two_violations() {
        let file = l1_file_with(vec![
            external_import("comemo::Tracked", 3),
            external_import("tokio::runtime::Runtime", 7),
        ]);
        let allowed = whitelist(&["thiserror"]);
        let violations = check(&file, &allowed, false);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn package_name_extracts_first_segment() {
        assert_eq!(super::package_name("serde::Serialize"), "serde");
        assert_eq!(super::package_name("std::collections::HashMap"), "std");
        assert_eq!(super::package_name("tokio"), "tokio");
    }
}
