//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/project-index.md
//! @prompt-hash 6c266cff
//! @layer L1
//! @updated 2026-03-15

use std::collections::HashSet;
use std::path::Path;

use crate::entities::parsed_file::ParsedFile;

// ── LocalIndex ────────────────────────────────────────────────────────────────

/// Contribuição de um único arquivo para o índice global.
/// Produzido durante a fase Map do pipeline paralelo.
/// Deve ser barato de construir e de fundir.
#[derive(Debug, Clone)]
pub struct LocalIndex<'a> {
    /// prompt_path referenciado pelo @prompt header deste arquivo.
    /// None se arquivo não tem header (V1 já cobre esse caso).
    pub referenced_prompt: Option<&'a str>,

    /// Se este arquivo tem Layer::Unknown e não está em excluídos.
    /// None se layer é conhecida. Some(path) se é alien.
    pub alien_file: Option<&'a Path>,
}

impl<'a> LocalIndex<'a> {
    pub fn empty() -> Self {
        Self { referenced_prompt: None, alien_file: None }
    }

    pub fn from_parsed(file: &ParsedFile<'a>) -> Self {
        Self {
            referenced_prompt: file.prompt_header
                .as_ref()
                .map(|h| h.prompt_path),
            alien_file: None, // arquivo parseado tem layer conhecida
        }
    }

    pub fn from_alien(path: &'a Path) -> Self {
        Self {
            referenced_prompt: None,
            alien_file: Some(path),
        }
    }

    pub fn from_source_error() -> Self {
        Self::empty() // V0 já cobre, não contribui para o índice
    }
}

// ── ProjectIndex ──────────────────────────────────────────────────────────────

/// Índice global construído por fusão de todos os LocalIndex.
/// Entregue a V7 e V8 após o pipeline paralelo completar.
#[derive(Debug, Default)]
pub struct ProjectIndex<'a> {
    /// Todos os prompt_paths referenciados por @prompt headers
    /// em arquivos válidos de L1–L4.
    pub referenced_prompts: HashSet<&'a str>,

    /// Arquivos com Layer::Unknown fora de diretórios excluídos.
    pub alien_files: Vec<&'a Path>,
}

impl<'a> ProjectIndex<'a> {
    pub fn new() -> Self {
        Self {
            referenced_prompts: HashSet::new(),
            alien_files: Vec::new(),
        }
    }

    /// Absorve um LocalIndex — operação da fase Reduce.
    /// Associativa e comutativa — segura para rayon::fold.
    pub fn merge_local(&mut self, local: LocalIndex<'a>) {
        if let Some(prompt) = local.referenced_prompt {
            self.referenced_prompts.insert(prompt);
        }
        if let Some(path) = local.alien_file {
            self.alien_files.push(path);
        }
    }

    /// Funde dois ProjectIndex — para rayon::reduce.
    pub fn merge(mut self, other: ProjectIndex<'a>) -> ProjectIndex<'a> {
        self.referenced_prompts.extend(other.referenced_prompts);
        self.alien_files.extend(other.alien_files);
        self
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{ParsedFile, PromptHeader, PublicInterface};

    fn base_parsed(path: &'static Path, prompt_path: &'static str) -> ParsedFile<'static> {
        ParsedFile {
            path,
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: Some(PromptHeader {
                prompt_path,
                prompt_hash: None,
                current_hash: None,
                layer: Layer::L1,
                updated: None,
            }),
            prompt_file_exists: true,
            has_test_coverage: true,
            imports: vec![],
            tokens: vec![],
            public_interface: PublicInterface::empty(),
            prompt_snapshot: None,
        }
    }

    #[test]
    fn merge_two_locals_with_distinct_prompts() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex { referenced_prompt: Some("prompts/a.md"), alien_file: None });
        index.merge_local(LocalIndex { referenced_prompt: Some("prompts/b.md"), alien_file: None });
        assert!(index.referenced_prompts.contains("prompts/a.md"));
        assert!(index.referenced_prompts.contains("prompts/b.md"));
    }

    #[test]
    fn from_alien_adds_path_to_index() {
        let mut index = ProjectIndex::new();
        let path = Path::new("src/utils/helper.rs");
        index.merge_local(LocalIndex::from_alien(path));
        assert_eq!(index.alien_files, vec![path]);
    }

    #[test]
    fn from_parsed_with_header_adds_prompt() {
        let mut index = ProjectIndex::new();
        let parsed = base_parsed(
            Path::new("01_core/rules/auth.rs"),
            "00_nucleo/prompts/rules/auth.md",
        );
        index.merge_local(LocalIndex::from_parsed(&parsed));
        assert!(index.referenced_prompts.contains("00_nucleo/prompts/rules/auth.md"));
    }

    #[test]
    fn empty_local_does_not_change_index() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex::empty());
        assert!(index.referenced_prompts.is_empty());
        assert!(index.alien_files.is_empty());
    }

    #[test]
    fn merge_is_commutative() {
        let mut a = ProjectIndex::new();
        a.merge_local(LocalIndex { referenced_prompt: Some("prompts/x.md"), alien_file: None });

        let mut b = ProjectIndex::new();
        b.merge_local(LocalIndex { referenced_prompt: Some("prompts/y.md"), alien_file: None });

        let merged_ab = a.merge(b);

        let mut c = ProjectIndex::new();
        c.merge_local(LocalIndex { referenced_prompt: Some("prompts/x.md"), alien_file: None });
        let mut d = ProjectIndex::new();
        d.merge_local(LocalIndex { referenced_prompt: Some("prompts/y.md"), alien_file: None });
        let merged_dc = d.merge(c);

        assert_eq!(merged_ab.referenced_prompts, merged_dc.referenced_prompts);
    }

    #[test]
    fn from_source_error_does_not_contribute() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex::from_source_error());
        assert!(index.referenced_prompts.is_empty());
        assert!(index.alien_files.is_empty());
    }
}
