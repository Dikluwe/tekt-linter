//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/file-provider.md
//! @prompt-hash 52fcd52b
//! @layer L1
//! @updated 2026-03-13

use std::path::PathBuf;

use crate::entities::layer::{Language, Layer};

/// Unit of transfer between FileWalker (L3) and LanguageParser (L3).
/// All fields are populated by L3 before delivery — parser never accesses disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: Language,
    /// Resolved by FileWalker via crystalline.toml prefix matching.
    pub layer: Layer,
    /// true if foo_test.rs exists in the same directory as foo.rs.
    /// Checked by walker at discovery time.
    pub has_adjacent_test: bool,
}

pub trait FileProvider {
    fn files(&self) -> impl Iterator<Item = SourceFile>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source_file(path: &str, layer: Layer) -> SourceFile {
        SourceFile {
            path: PathBuf::from(path),
            content: "fn main() {}".to_string(),
            language: Language::Rust,
            layer,
            has_adjacent_test: false,
        }
    }

    #[test]
    fn source_file_clone_and_eq() {
        let f = make_source_file("01_core/foo.rs", Layer::L1);
        assert_eq!(f.clone(), f);
    }

    #[test]
    fn source_file_layer_l1() {
        let f = make_source_file("01_core/foo.rs", Layer::L1);
        assert_eq!(f.layer, Layer::L1);
    }

    #[test]
    fn source_file_has_adjacent_test_true() {
        let mut f = make_source_file("01_core/foo.rs", Layer::L1);
        f.has_adjacent_test = true;
        assert!(f.has_adjacent_test);
    }

    #[test]
    fn source_file_has_adjacent_test_false() {
        let f = make_source_file("01_core/bar.rs", Layer::L1);
        assert!(!f.has_adjacent_test);
    }

    struct MockProvider {
        items: Vec<SourceFile>,
    }

    impl FileProvider for MockProvider {
        fn files(&self) -> impl Iterator<Item = SourceFile> {
            self.items.clone().into_iter()
        }
    }

    #[test]
    fn file_provider_returns_correct_count() {
        let provider = MockProvider {
            items: vec![
                make_source_file("01_core/a.rs", Layer::L1),
                make_source_file("01_core/b.rs", Layer::L1),
            ],
        };
        assert_eq!(provider.files().count(), 2);
    }

    #[test]
    fn file_provider_no_disk_access_in_tests() {
        // Mock provider delivers fixed SourceFiles — zero I/O
        let provider = MockProvider { items: vec![] };
        assert_eq!(provider.files().count(), 0);
    }
}
