//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/prompt-walker.md
//! @prompt-hash b618888c
//! @layer L3
//! @updated 2026-03-15

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::contracts::prompt_provider::{AllPrompts, PromptEntry, PromptProvider, PromptScanError};

// ── FsPromptWalker ────────────────────────────────────────────────────────────

/// Implementação L3 de PromptProvider.
///
/// Varre 00_nucleo/prompts/ recursivamente, constrói AllPrompts excluindo
/// as exceções declaradas em crystalline.toml [orphan_exceptions].
///
/// Timing: invocado sequencialmente em L4 antes do pipeline paralelo.
/// Não participa do Map-Reduce — AllPrompts é imutável após construção.
///
/// Convenção de path: paths relativos à raiz do projeto
/// (ex: "00_nucleo/prompts/linter-core.md"), comparáveis diretamente
/// com @prompt headers nos arquivos de código.
pub struct FsPromptWalker {
    /// Raiz do projeto (ex: PathBuf::from(".")).
    /// Padrão idêntico ao FsPromptReader — join(prompt_path) dá o path absoluto.
    pub project_root: PathBuf,
    /// Paths relativos ao projeto das exceções de orphan_exceptions.
    /// Exemplo: "00_nucleo/prompts/template.md"
    pub orphan_exceptions: HashSet<String>,
    /// Buffer interno para paths internados.
    /// Usa Box<str> para que o dado heap não se mova durante realocações do Vec.
    /// Garante que &'a str retornado vive no walker.
    paths_buffer: RefCell<Vec<Box<str>>>,
}

impl FsPromptWalker {
    pub fn new(project_root: PathBuf, orphan_exceptions: HashSet<String>) -> Self {
        Self {
            project_root,
            orphan_exceptions,
            paths_buffer: RefCell::new(Vec::new()),
        }
    }

    /// Interna um String no buffer e retorna &'a str vinculado ao lifetime do walker.
    ///
    /// Usa Box<str> para que os dados no heap não se movam quando o Vec realoca.
    /// A referência retornada aponta para o dado heap do Box, não para o Box em si.
    ///
    /// SAFETY: O dado heap do Box<str> sobrevive enquanto o Box existir dentro de
    /// self.paths_buffer (lifetime 'a). Realoções do Vec movem o Box (o fat pointer),
    /// não o dado heap apontado — portanto o raw pointer permanece válido.
    fn intern<'a>(&'a self, path: String) -> &'a str {
        let mut buf = self.paths_buffer.borrow_mut();
        let boxed: Box<str> = path.into_boxed_str();
        let raw: *const str = &*boxed as *const str;
        buf.push(boxed);
        // SAFETY: raw aponta para dado heap que vive em self.paths_buffer ('a).
        unsafe { &*raw }
    }
}

impl PromptProvider for FsPromptWalker {
    fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError> {
        let prompts_dir = self.project_root.join("00_nucleo").join("prompts");

        if !prompts_dir.exists() {
            return Err(PromptScanError::NucleoUnreadable {
                path: prompts_dir.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} não existe ou não pode ser lido", prompts_dir.display()),
                ),
            });
        }

        let mut entries: HashSet<PromptEntry<'a>> = HashSet::new();

        for result in WalkDir::new(&prompts_dir) {
            let entry = result.map_err(|e| PromptScanError::NucleoUnreadable {
                path: prompts_dir.clone(),
                source: std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
            })?;

            if !entry.file_type().is_file() {
                continue;
            }

            let ext = entry.path().extension().and_then(|e| e.to_str());
            if ext != Some("md") {
                continue;
            }

            let relative = entry
                .path()
                .strip_prefix(&self.project_root)
                .map_err(|_| PromptScanError::NucleoUnreadable {
                    path: entry.path().to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "não foi possível calcular path relativo de {}",
                            entry.path().display()
                        ),
                    ),
                })?
                .to_str()
                .ok_or_else(|| PromptScanError::InvalidUtf8 {
                    path: entry.path().to_path_buf(),
                })?
                .to_string();

            // Excluir orphan_exceptions antes de retornar — V7 nunca as vê
            if self.orphan_exceptions.contains(&relative) {
                continue;
            }

            let interned = self.intern(relative);
            entries.insert(PromptEntry { relative_path: interned });
        }

        Ok(AllPrompts { entries })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_nucleo(tmp: &TempDir, files: &[&str]) -> PathBuf {
        let root = tmp.path().to_path_buf();
        for file in files {
            let full = root.join(file);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(&full, b"# prompt content").unwrap();
        }
        root
    }

    #[test]
    fn scan_three_md_files_returns_three_entries() {
        let tmp = TempDir::new().unwrap();
        let root = setup_nucleo(&tmp, &[
            "00_nucleo/prompts/a.md",
            "00_nucleo/prompts/b.md",
            "00_nucleo/prompts/c.md",
        ]);
        let walker = FsPromptWalker::new(root, HashSet::new());
        let all = walker.scan().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn scan_excludes_orphan_exceptions() {
        let tmp = TempDir::new().unwrap();
        let root = setup_nucleo(&tmp, &[
            "00_nucleo/prompts/readme.md",
            "00_nucleo/prompts/template.md",
            "00_nucleo/prompts/linter-core.md",
        ]);
        let exceptions = ["00_nucleo/prompts/readme.md".to_string()].into_iter().collect();
        let walker = FsPromptWalker::new(root, exceptions);
        let all = walker.scan().unwrap();
        assert_eq!(all.len(), 2);
        assert!(!all.contains("00_nucleo/prompts/readme.md"));
    }

    #[test]
    fn scan_nested_md_gets_relative_path() {
        let tmp = TempDir::new().unwrap();
        let root = setup_nucleo(&tmp, &[
            "00_nucleo/prompts/rules/v3.md",
        ]);
        let walker = FsPromptWalker::new(root, HashSet::new());
        let all = walker.scan().unwrap();
        assert!(all.contains("00_nucleo/prompts/rules/v3.md"));
    }

    #[test]
    fn scan_ignores_non_md_files() {
        let tmp = TempDir::new().unwrap();
        let root = setup_nucleo(&tmp, &[
            "00_nucleo/prompts/config.toml",
            "00_nucleo/prompts/real.md",
        ]);
        let walker = FsPromptWalker::new(root, HashSet::new());
        let all = walker.scan().unwrap();
        assert_eq!(all.len(), 1);
        assert!(all.contains("00_nucleo/prompts/real.md"));
    }

    #[test]
    fn scan_missing_nucleo_returns_error() {
        let walker = FsPromptWalker::new(
            PathBuf::from("/tmp/nonexistent_project_xyz"),
            HashSet::new(),
        );
        assert!(matches!(walker.scan(), Err(PromptScanError::NucleoUnreadable { .. })));
    }

    #[test]
    fn scan_result_is_immutable_after_construction() {
        let tmp = TempDir::new().unwrap();
        let root = setup_nucleo(&tmp, &["00_nucleo/prompts/x.md"]);
        let walker = FsPromptWalker::new(root, HashSet::new());
        let all = walker.scan().unwrap();
        // AllPrompts não expõe mutação — acesso concorrente é seguro
        assert_eq!(all.len(), 1);
    }
}
