//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/project-index.md
//! @prompt-hash 1abce1fc
//! @layer L1
//! @updated 2026-03-15

use std::collections::HashSet;
use std::path::Path;

use crate::entities::layer::Layer;
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

    /// Traits públicas declaradas neste arquivo em L1/contracts/.
    /// Vazio para arquivos fora de L1 ou fora de subdir "contracts".
    /// Populado pelo RustParser a partir de nós `trait_item` com `pub`.
    pub declared_traits: Vec<&'a str>,

    /// Traits implementadas neste arquivo via `impl Trait for Type`.
    /// Vazio para arquivos fora de L2 e L3.
    /// Populado pelo RustParser a partir de nós `impl_item` com trait.
    pub implemented_traits: Vec<&'a str>,
}

impl<'a> LocalIndex<'a> {
    pub fn empty() -> Self {
        Self {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec![],
            implemented_traits: vec![],
        }
    }

    /// Constrói LocalIndex a partir de um ParsedFile.
    ///
    /// Detecta aliens internamente: se layer == Layer::Unknown,
    /// popula alien_file com o path do arquivo.
    ///
    /// `declared_traits` e `implemented_traits` são lidos dos campos
    /// homônimos de ParsedFile, populados por RustParser.
    /// from_parsed não os deriva — apenas os transporta.
    pub fn from_parsed(file: &ParsedFile<'a>) -> Self {
        Self {
            referenced_prompt: file.prompt_header
                .as_ref()
                .map(|h| h.prompt_path),
            // Layer::Unknown em arquivo parseado → alien (ADR-0006)
            alien_file: if file.layer == Layer::Unknown { Some(file.path) } else { None },
            declared_traits: file.declared_traits.clone(),
            implemented_traits: file.implemented_traits.clone(),
        }
    }

    /// Constrói LocalIndex para arquivo que falhou no parse.
    /// Não é alien — arquivo tem layer conhecida mas conteúdo inválido.
    pub fn from_parse_error() -> Self {
        Self::empty()
    }

    pub fn from_source_error() -> Self {
        Self::empty() // V0 já cobre, não contribui para o índice
    }
}

// ── ProjectIndex ──────────────────────────────────────────────────────────────

/// Índice global construído por fusão de todos os LocalIndex.
/// Entregue a V7, V8 e V11 após o pipeline paralelo completar.
#[derive(Debug, Default)]
pub struct ProjectIndex<'a> {
    /// Todos os prompt_paths referenciados por @prompt headers
    /// em arquivos válidos de L1–L4.
    pub referenced_prompts: HashSet<&'a str>,

    /// Arquivos com Layer::Unknown fora de diretórios excluídos.
    pub alien_files: Vec<&'a Path>,

    /// Todas as traits públicas declaradas em L1/contracts/.
    /// Agregado de LocalIndex.declared_traits de todos os arquivos L1.
    /// Usado por V11 para detectar contratos sem implementação.
    pub all_declared_traits: HashSet<&'a str>,

    /// Todas as traits implementadas em L2 ou L3.
    /// Agregado de LocalIndex.implemented_traits de todos os arquivos L2/L3.
    /// Usado por V11 para fechar o circuito contrato → implementação.
    pub all_implemented_traits: HashSet<&'a str>,
}

impl<'a> ProjectIndex<'a> {
    pub fn new() -> Self {
        Self {
            referenced_prompts: HashSet::new(),
            alien_files: Vec::new(),
            all_declared_traits: HashSet::new(),
            all_implemented_traits: HashSet::new(),
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
        self.all_declared_traits.extend(local.declared_traits);
        self.all_implemented_traits.extend(local.implemented_traits);
    }

    /// Funde dois ProjectIndex — para rayon::reduce.
    pub fn merge(mut self, other: ProjectIndex<'a>) -> ProjectIndex<'a> {
        self.referenced_prompts.extend(other.referenced_prompts);
        self.alien_files.extend(other.alien_files);
        self.all_declared_traits.extend(other.all_declared_traits);
        self.all_implemented_traits.extend(other.all_implemented_traits);
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
            declared_traits: vec![],
            implemented_traits: vec![],
            declarations: vec![],
            static_declarations: vec![],
            module_decls: vec![],
        }
    }

    fn local(prompt: Option<&'static str>, alien: Option<&'static Path>) -> LocalIndex<'static> {
        LocalIndex {
            referenced_prompt: prompt,
            alien_file: alien,
            declared_traits: vec![],
            implemented_traits: vec![],
        }
    }

    #[test]
    fn merge_two_locals_with_distinct_prompts() {
        let mut index = ProjectIndex::new();
        index.merge_local(local(Some("prompts/a.md"), None));
        index.merge_local(local(Some("prompts/b.md"), None));
        assert!(index.referenced_prompts.contains("prompts/a.md"));
        assert!(index.referenced_prompts.contains("prompts/b.md"));
    }

    #[test]
    fn unknown_layer_adds_alien_to_index() {
        let mut index = ProjectIndex::new();
        let path = Path::new("src/utils/helper.rs");
        index.merge_local(LocalIndex { referenced_prompt: None, alien_file: Some(path), declared_traits: vec![], implemented_traits: vec![] });
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
        assert!(index.all_declared_traits.is_empty());
        assert!(index.all_implemented_traits.is_empty());
    }

    #[test]
    fn merge_is_commutative() {
        let mut a = ProjectIndex::new();
        a.merge_local(local(Some("prompts/x.md"), None));

        let mut b = ProjectIndex::new();
        b.merge_local(local(Some("prompts/y.md"), None));

        let merged_ab = a.merge(b);

        let mut c = ProjectIndex::new();
        c.merge_local(local(Some("prompts/x.md"), None));
        let mut d = ProjectIndex::new();
        d.merge_local(local(Some("prompts/y.md"), None));
        let merged_dc = d.merge(c);

        assert_eq!(merged_ab.referenced_prompts, merged_dc.referenced_prompts);
    }

    #[test]
    fn from_source_error_does_not_contribute() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex::from_source_error());
        assert!(index.referenced_prompts.is_empty());
        assert!(index.alien_files.is_empty());
        assert!(index.all_declared_traits.is_empty());
        assert!(index.all_implemented_traits.is_empty());
    }

    // ── declared_traits / implemented_traits ──────────────────────────────────

    #[test]
    fn from_parsed_transports_declared_traits() {
        let mut parsed = base_parsed(
            Path::new("01_core/contracts/file_provider.rs"),
            "00_nucleo/prompts/file-provider.md",
        );
        parsed.declared_traits = vec!["FileProvider", "LanguageParser"];
        let local = LocalIndex::from_parsed(&parsed);
        assert_eq!(local.declared_traits, vec!["FileProvider", "LanguageParser"]);
        assert!(local.implemented_traits.is_empty());
    }

    #[test]
    fn from_parsed_transports_implemented_traits() {
        let mut parsed = base_parsed(
            Path::new("03_infra/walker.rs"),
            "00_nucleo/prompts/file-walker.md",
        );
        parsed.implemented_traits = vec!["FileProvider"];
        let local = LocalIndex::from_parsed(&parsed);
        assert_eq!(local.implemented_traits, vec!["FileProvider"]);
        assert!(local.declared_traits.is_empty());
    }

    #[test]
    fn merge_local_accumulates_declared_traits() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["FileProvider"],
            implemented_traits: vec![],
        });
        index.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["LanguageParser"],
            implemented_traits: vec![],
        });
        assert!(index.all_declared_traits.contains("FileProvider"));
        assert!(index.all_declared_traits.contains("LanguageParser"));
    }

    #[test]
    fn merge_local_accumulates_implemented_traits() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec![],
            implemented_traits: vec!["FileProvider"],
        });
        assert!(index.all_implemented_traits.contains("FileProvider"));
        assert!(index.all_declared_traits.is_empty());
    }

    #[test]
    fn merge_accumulates_trait_sets_from_both_sides() {
        let mut a = ProjectIndex::new();
        a.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["FileProvider"],
            implemented_traits: vec![],
        });

        let mut b = ProjectIndex::new();
        b.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["LanguageParser"],
            implemented_traits: vec!["FileProvider"],
        });

        let merged = a.merge(b);
        assert!(merged.all_declared_traits.contains("FileProvider"));
        assert!(merged.all_declared_traits.contains("LanguageParser"));
        assert!(merged.all_implemented_traits.contains("FileProvider"));
    }

    #[test]
    fn from_parse_error_does_not_contribute() {
        let mut index = ProjectIndex::new();
        index.merge_local(LocalIndex::from_parse_error());
        assert!(index.referenced_prompts.is_empty());
        assert!(index.alien_files.is_empty());
        assert!(index.all_declared_traits.is_empty());
        assert!(index.all_implemented_traits.is_empty());
    }

    #[test]
    fn declared_traits_from_multiple_locals_are_unioned() {
        let mut index = ProjectIndex::new();
        // Mesma trait declarada em dois arquivos de contracts/ — idempotente no HashSet
        index.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["FileProvider"],
            implemented_traits: vec![],
        });
        index.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["FileProvider"],
            implemented_traits: vec![],
        });
        assert_eq!(index.all_declared_traits.len(), 1);
    }

    #[test]
    fn merge_is_commutative_for_traits() {
        let mut a = ProjectIndex::new();
        a.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["TraitA"],
            implemented_traits: vec!["TraitB"],
        });

        let mut b = ProjectIndex::new();
        b.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["TraitB"],
            implemented_traits: vec!["TraitA"],
        });

        let ab = a.merge(b);

        let mut c = ProjectIndex::new();
        c.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["TraitA"],
            implemented_traits: vec!["TraitB"],
        });

        let mut d = ProjectIndex::new();
        d.merge_local(LocalIndex {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec!["TraitB"],
            implemented_traits: vec!["TraitA"],
        });

        let dc = d.merge(c);
        assert_eq!(ab.all_declared_traits, dc.all_declared_traits);
        assert_eq!(ab.all_implemented_traits, dc.all_implemented_traits);
    }
}
