//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-provider.md
//! @prompt-hash 5e1fdc46
//! @layer L1
//! @updated 2026-03-15

use std::collections::HashSet;
use std::path::PathBuf;

// ── PromptEntry ───────────────────────────────────────────────────────────────

/// Um prompt descoberto em 00_nucleo/prompts/.
/// Carrega apenas o path relativo à raiz do projeto —
/// suficiente para comparação com @prompt headers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PromptEntry<'a> {
    /// Path relativo à raiz do projeto.
    /// Exemplo: "00_nucleo/prompts/rules/forbidden-import.md"
    /// Comparável diretamente com @prompt headers.
    pub relative_path: &'a str,
}

// ── AllPrompts ────────────────────────────────────────────────────────────────

/// Conjunto de todos os prompts existentes em 00_nucleo/prompts/,
/// excluindo as exceções declaradas em [orphan_exceptions].
/// Construído por L3 (FsPromptWalker) antes do pipeline paralelo.
/// Imutável durante toda a execução — seguro para acesso concorrente.
#[derive(Debug)]
pub struct AllPrompts<'a> {
    pub entries: HashSet<PromptEntry<'a>>,
}

impl<'a> AllPrompts<'a> {
    pub fn contains(&self, path: &str) -> bool {
        self.entries.iter().any(|e| e.relative_path == path)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── PromptScanError ───────────────────────────────────────────────────────────

/// Erro ao varrer 00_nucleo/prompts/.
/// Distinto de SourceError — ocorre antes do pipeline paralelo.
#[derive(Debug)]
pub enum PromptScanError {
    NucleoUnreadable { reason: String },
    InvalidUtf8 { path: PathBuf },
}

// ── PromptProvider (trait) ────────────────────────────────────────────────────

pub trait PromptProvider {
    /// Varre 00_nucleo/prompts/ e retorna todos os prompts
    /// existentes, excluindo as exceções configuradas.
    ///
    /// Invocado sequencialmente uma única vez antes do pipeline
    /// paralelo. O resultado é passado como referência imutável
    /// a V7 após a fase Reduce.
    ///
    /// Erros de leitura de diretório são propagados — não
    /// silenciados. Se 00_nucleo/ não puder ser lido, o linter
    /// não pode garantir completude e deve falhar.
    fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_all_prompts(paths: &[&'static str]) -> AllPrompts<'static> {
        AllPrompts {
            entries: paths.iter().map(|p| PromptEntry { relative_path: p }).collect(),
        }
    }

    #[test]
    fn contains_returns_true_for_known_prompt() {
        let ap = make_all_prompts(&["00_nucleo/prompts/auth.md"]);
        assert!(ap.contains("00_nucleo/prompts/auth.md"));
    }

    #[test]
    fn contains_returns_false_for_missing_prompt() {
        let ap = make_all_prompts(&["00_nucleo/prompts/auth.md"]);
        assert!(!ap.contains("00_nucleo/prompts/missing.md"));
    }

    #[test]
    fn len_matches_entry_count() {
        let ap = make_all_prompts(&["00_nucleo/prompts/a.md", "00_nucleo/prompts/b.md"]);
        assert_eq!(ap.len(), 2);
    }

    #[test]
    fn is_empty_true_for_empty_set() {
        let ap = AllPrompts { entries: HashSet::new() };
        assert!(ap.is_empty());
    }

    struct MockProvider {
        result: Vec<&'static str>,
    }

    impl PromptProvider for MockProvider {
        fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError> {
            Ok(AllPrompts {
                entries: self.result.iter()
                    .map(|p| PromptEntry { relative_path: p })
                    .collect(),
            })
        }
    }

    #[test]
    fn mock_provider_no_disk_access() {
        let provider = MockProvider {
            result: vec!["00_nucleo/prompts/rules/v3.md"],
        };
        let all = provider.scan().unwrap();
        assert!(all.contains("00_nucleo/prompts/rules/v3.md"));
    }

    #[test]
    fn scan_error_nucleounreadable_is_debug() {
        let err = PromptScanError::NucleoUnreadable {
            reason: "permission denied".to_string(),
        };
        let msg = format!("{:?}", err);
        assert!(msg.contains("NucleoUnreadable"));
    }
}
