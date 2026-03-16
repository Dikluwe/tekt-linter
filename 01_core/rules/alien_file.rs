//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/rules/alien-file.md
//! @prompt-hash a00a1ed2
//! @layer L1
//! @updated 2026-03-15

use std::borrow::Cow;

use crate::entities::project_index::ProjectIndex;
use crate::entities::violation::{Location, Violation, ViolationLevel};

/// V8 — Alien File (Vácuo Topológico).
///
/// Todo arquivo de código com Layer::Unknown fora de diretórios
/// excluídos gera violação Fatal. A gaiola arquitetural é hermética —
/// não existe arquivo fora da topologia.
///
/// Opera sobre ProjectIndex.alien_files — lista de paths construída
/// pelo walker a partir de arquivos com Layer::Unknown.
///
/// Fatal — não pode ser suprimido por --checks, mesmo comportamento de V0.
pub fn check_aliens<'a>(index: &ProjectIndex<'a>) -> Vec<Violation<'a>> {
    index
        .alien_files
        .iter()
        .map(|path| Violation {
            rule_id: "V8".to_string(),
            level: ViolationLevel::Fatal,
            message: format!(
                "Arquivo fora da topologia: '{}' não pertence a \
                 nenhuma camada mapeada em crystalline.toml. \
                 Mapear o diretório ou mover o arquivo.",
                path.display()
            ),
            location: Location {
                path: Cow::Owned(path.to_path_buf()),
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
    use std::path::Path;

    use crate::entities::project_index::ProjectIndex;

    #[test]
    fn unknown_layer_file_returns_v8_fatal() {
        let mut index = ProjectIndex::new();
        index.alien_files.push(Path::new("src/utils/helper.rs"));
        let violations = check_aliens(&index);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, "V8");
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
    }

    #[test]
    fn violation_message_contains_path() {
        let mut index = ProjectIndex::new();
        index.alien_files.push(Path::new("scripts/gen.rs"));
        let violations = check_aliens(&index);
        assert!(violations[0].message.contains("scripts/gen.rs"));
    }

    #[test]
    fn empty_alien_files_returns_no_violations() {
        let index = ProjectIndex::new();
        assert!(check_aliens(&index).is_empty());
    }

    #[test]
    fn multiple_alien_files_produce_one_violation_each() {
        let mut index = ProjectIndex::new();
        index.alien_files.push(Path::new("scripts/a.rs"));
        index.alien_files.push(Path::new("scripts/b.rs"));
        assert_eq!(check_aliens(&index).len(), 2);
    }

    #[test]
    fn violation_level_is_fatal_not_error() {
        let mut index = ProjectIndex::new();
        index.alien_files.push(Path::new("foo/bar.rs"));
        let violations = check_aliens(&index);
        assert_eq!(violations[0].level, ViolationLevel::Fatal);
        assert_ne!(violations[0].level, ViolationLevel::Error);
    }
}
