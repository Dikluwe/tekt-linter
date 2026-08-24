//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-reader.md
//! @prompt-hash 48a4329e
//! @layer L1
//! @updated 2026-03-13

/// Contract for reading prompt files from 00_nucleo/.
/// L3 implements (FsPromptReader with std::fs + sha2).
/// L1 declares — zero I/O, zero sha2.
///
/// Used by:
/// - V1: exists() to verify prompt_file_exists
/// - V5: read_hash() to detect drift
pub trait PromptReader {
    /// Returns SHA256[0..8] of the prompt file at 00_nucleo/<prompt_path>.
    /// Returns None if the file does not exist.
    fn read_hash(&self, prompt_path: &str) -> Option<String>;

    /// Returns true if the prompt file exists in 00_nucleo/.
    fn exists(&self, prompt_path: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPromptReader {
        hash: Option<String>,
        file_exists: bool,
    }

    impl PromptReader for MockPromptReader {
        fn read_hash(&self, _prompt_path: &str) -> Option<String> {
            self.hash.clone()
        }

        fn exists(&self, _prompt_path: &str) -> bool {
            self.file_exists
        }
    }

    #[test]
    fn read_hash_returns_eight_char_hex() {
        let reader = MockPromptReader {
            hash: Some("a3f8c2d1".to_string()),
            file_exists: true,
        };
        let hash = reader.read_hash("prompts/auth.md");
        assert_eq!(hash, Some("a3f8c2d1".to_string()));
        assert_eq!(hash.unwrap().len(), 8);
    }

    #[test]
    fn read_hash_returns_none_when_file_missing() {
        let reader = MockPromptReader { hash: None, file_exists: false };
        assert_eq!(reader.read_hash("prompts/missing.md"), None);
    }

    #[test]
    fn exists_returns_true_when_present() {
        let reader = MockPromptReader { hash: Some("00000000".to_string()), file_exists: true };
        assert!(reader.exists("prompts/auth.md"));
    }

    #[test]
    fn exists_returns_false_when_absent() {
        let reader = MockPromptReader { hash: None, file_exists: false };
        assert!(!reader.exists("prompts/missing.md"));
    }

    #[test]
    fn v5_uses_mock_without_disk_access() {
        // V5 operates purely on hashes — no I/O when using mock
        let reader = MockPromptReader {
            hash: Some("b9e4f7a2".to_string()),
            file_exists: true,
        };
        let declared = "a3f8c2d1";
        let current = reader.read_hash("prompts/linter-core.md").unwrap();
        assert_ne!(declared, current); // drift detected
    }
}
