//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/dangling-contract.md
//! @prompt-hash 752e9f31
//! @layer L1
//! @updated 2026-03-16

use std::borrow::Cow;
use std::path::PathBuf;

use crate::entities::project_index::ProjectIndex;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V11 — Dangling Contract.
///
/// Detecta traits públicas declaradas em L1/contracts/ que não têm
/// nenhum `impl Trait for Type` correspondente em L2 ou L3.
/// O circuito está aberto — nenhuma instância pode ser injetada em L4.
///
/// Opera sobre ProjectIndex após a fase Reduce do pipeline paralelo.
/// Comparação por nome simples da trait (limitação declarada no prompt).
///
/// Error — bloqueia CI. Sem exceções configuráveis.
pub fn check_dangling_contracts<'a>(index: &ProjectIndex<'a>) -> Vec<Violation<'a>> {
    index
        .all_declared_traits
        .iter()
        .filter(|t| !index.all_implemented_traits.contains(*t))
        .map(|trait_name| Violation {
            rule_id: "V11".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Contrato sem implementação: trait '{}' declarada em \
                 L1/contracts/ não tem impl correspondente em L2 ou L3. \
                 O circuito está aberto — nenhuma instância pode ser injetada.",
                trait_name
            ),
            location: Location {
                path: Cow::Owned(PathBuf::from("01_core/contracts")),
                line: 0,
                column: 0,
            },
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::project_index::ProjectIndex;

    fn index_with(declared: &[&'static str], implemented: &[&'static str]) -> ProjectIndex<'static> {
        let mut index = ProjectIndex::new();
        for t in declared { index.all_declared_traits.insert(t); }
        for t in implemented { index.all_implemented_traits.insert(t); }
        index
    }

    #[test]
    fn declared_without_impl_returns_v11_error() {
        let index = index_with(&["FileProvider"], &[]);
        let violations = check_dangling_contracts(&index);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V11");
        assert_eq!(violations[0].level, ViolationLevel::Error);
        assert!(violations[0].message.contains("FileProvider"));
    }

    #[test]
    fn declared_with_impl_returns_no_violation() {
        let index = index_with(&["LanguageParser"], &["LanguageParser"]);
        assert!(check_dangling_contracts(&index).is_empty());
    }

    #[test]
    fn all_implemented_returns_empty() {
        let index = index_with(&["FileProvider", "LanguageParser"], &["FileProvider", "LanguageParser"]);
        assert!(check_dangling_contracts(&index).is_empty());
    }

    #[test]
    fn two_declared_one_missing_returns_one_violation() {
        let index = index_with(&["FileProvider", "LanguageParser"], &["FileProvider"]);
        let violations = check_dangling_contracts(&index);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("LanguageParser"));
    }

    #[test]
    fn empty_declared_returns_no_violations() {
        let index = index_with(&[], &["FileProvider"]);
        assert!(check_dangling_contracts(&index).is_empty());
    }

    #[test]
    fn violation_location_points_to_contracts_dir() {
        let index = index_with(&["PromptReader"], &[]);
        let violations = check_dangling_contracts(&index);
        assert_eq!(violations[0].location.path.as_os_str(), "01_core/contracts");
        assert_eq!(violations[0].location.line, 0);
    }

    #[test]
    fn multiple_dangling_traits_produce_one_violation_each() {
        let index = index_with(&["TraitA", "TraitB", "TraitC"], &[]);
        assert_eq!(check_dangling_contracts(&index).len(), 3);
    }

    #[test]
    fn extra_implemented_without_declaration_is_not_violation() {
        // Traits implementadas sem declaração em contracts/ não disparam V11
        let index = index_with(&[], &["SomeAdapter"]);
        assert!(check_dangling_contracts(&index).is_empty());
    }
}
