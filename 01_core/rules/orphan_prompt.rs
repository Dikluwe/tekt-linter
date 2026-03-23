//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/orphan-prompt.md
//! @prompt-hash 4b4bc5a1
//! @layer L1
//! @updated 2026-03-23

use std::borrow::Cow;
use std::path::PathBuf;

use crate::contracts::prompt_provider::AllPrompts;
use crate::entities::project_index::ProjectIndex;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V7 — Orphan Prompt (Semente Estéril).
///
/// Para cada prompt em AllPrompts, verifica se existe pelo menos um
/// arquivo em L1–L4 com @prompt header referenciando-o.
/// Prompts sem materialização são sementes estéreis.
///
/// Opera sobre ProjectIndex (referenced_prompts) e AllPrompts (entries).
/// Não opera sobre ParsedFile individual — requer visão global.
///
/// Warning por padrão — não quebra projetos existentes na adoção inicial.
pub fn check_orphans<'a>(
    index: &ProjectIndex<'a>,
    all_prompts: &AllPrompts<'a>,
    level: ViolationLevel,
) -> Vec<Violation<'a>> {
    all_prompts
        .entries
        .iter()
        .filter(|entry| !index.referenced_prompts.contains(entry.relative_path))
        .map(|entry| Violation {
            rule_id: "V7".to_string(),
            level: level.clone(),
            message: format!(
                "Prompt órfão: '{}' não é referenciado por nenhum \
                 arquivo em L1–L4. Materializar ou remover.",
                entry.relative_path
            ),
            location: Location {
                path: Cow::Owned(PathBuf::from(entry.relative_path)),
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
    use std::collections::HashSet;

    use crate::contracts::prompt_provider::{AllPrompts, PromptEntry};
    use crate::entities::project_index::ProjectIndex;

    fn make_all_prompts(paths: &[&'static str]) -> AllPrompts<'static> {
        AllPrompts {
            entries: paths.iter().map(|p| PromptEntry { relative_path: p }).collect(),
        }
    }

    fn make_index(referenced: &[&'static str]) -> ProjectIndex<'static> {
        let mut index = ProjectIndex::new();
        for path in referenced {
            index.referenced_prompts.insert(path);
        }
        index
    }

    #[test]
    fn orphan_prompt_not_referenced_returns_v7() {
        let all = make_all_prompts(&["00_nucleo/prompts/novo-contrato.md"]);
        let index = make_index(&[]);
        let violations = check_orphans(&index, &all, ViolationLevel::Warning);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V7");
        assert_eq!(violations[0].level, ViolationLevel::Warning);
        assert!(violations[0].message.contains("novo-contrato.md"));
    }

    #[test]
    fn referenced_prompt_does_not_return_v7() {
        let all = make_all_prompts(&["00_nucleo/prompts/auth.md"]);
        let index = make_index(&["00_nucleo/prompts/auth.md"]);
        assert!(check_orphans(&index, &all, ViolationLevel::Warning).is_empty());
    }

    #[test]
    fn all_prompts_equal_referenced_returns_empty() {
        let paths = ["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"];
        let all = make_all_prompts(&paths);
        let index = make_index(&paths);
        assert!(check_orphans(&index, &all, ViolationLevel::Warning).is_empty());
    }

    #[test]
    fn partial_orphans_returns_only_unreferenced() {
        let all = make_all_prompts(&[
            "00_nucleo/prompts/a.md",
            "00_nucleo/prompts/b.md",
        ]);
        let index = make_index(&["00_nucleo/prompts/a.md"]);
        let violations = check_orphans(&index, &all, ViolationLevel::Warning);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("b.md"));
    }

    #[test]
    fn violation_location_path_is_prompt_path() {
        let all = make_all_prompts(&["00_nucleo/prompts/orphan.md"]);
        let index = make_index(&[]);
        let violations = check_orphans(&index, &all, ViolationLevel::Warning);
        let path = violations[0].location.path.as_ref();
        assert_eq!(path, std::path::Path::new("00_nucleo/prompts/orphan.md"));
    }

    // Exceções (orphan_exceptions) são excluídas por L3 antes de construir
    // AllPrompts. V7 nunca as vê — o teste abaixo confirma que AllPrompts
    // vazio não produz violações.
    #[test]
    fn empty_all_prompts_returns_no_violations() {
        let all = AllPrompts { entries: HashSet::new() };
        let index = make_index(&[]);
        assert!(check_orphans(&index, &all, ViolationLevel::Warning).is_empty());
    }

    #[test]
    fn level_error_propagates_to_violation() {
        let all = make_all_prompts(&["00_nucleo/prompts/orphan.md"]);
        let index = make_index(&[]);
        let violations = check_orphans(&index, &all, ViolationLevel::Error);
        assert_eq!(violations[0].level, ViolationLevel::Error);
    }
}
