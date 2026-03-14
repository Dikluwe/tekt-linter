//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-reader.md
//! @prompt-hash 48a4329e
//! @layer L3
//! @updated 2026-03-13

use std::path::PathBuf;

use hex::encode;
use sha2::{Digest, Sha256};

use crate::contracts::prompt_reader::PromptReader;

/// L3 implementation of PromptReader.
/// Reads prompt files from 00_nucleo/ and returns SHA256[0..8].
/// std::io::Error is absorbed — never crosses to L1.
pub struct FsPromptReader {
    pub nucleo_root: PathBuf,
}

impl PromptReader for FsPromptReader {
    fn read_hash(&self, prompt_path: &str) -> Option<String> {
        let full_path = self.nucleo_root.join(prompt_path);
        let content = std::fs::read(&full_path).ok()?;
        let hash = Sha256::digest(&content);
        Some(encode(hash)[..8].to_string())
    }

    fn exists(&self, prompt_path: &str) -> bool {
        self.nucleo_root.join(prompt_path).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_temp_nucleo(content: &str, relative_path: &str) -> (TempDir, FsPromptReader) {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join(relative_path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        let reader = FsPromptReader { nucleo_root: dir.path().to_path_buf() };
        (dir, reader)
    }

    #[test]
    fn read_hash_returns_eight_hex_chars() {
        let (_dir, reader) = setup_temp_nucleo("# some prompt content", "prompts/auth.md");
        let hash = reader.read_hash("prompts/auth.md").unwrap();
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn read_hash_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let reader = FsPromptReader { nucleo_root: dir.path().to_path_buf() };
        assert!(reader.read_hash("prompts/missing.md").is_none());
    }

    #[test]
    fn exists_returns_true_when_present() {
        let (_dir, reader) = setup_temp_nucleo("content", "prompts/test.md");
        assert!(reader.exists("prompts/test.md"));
    }

    #[test]
    fn exists_returns_false_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let reader = FsPromptReader { nucleo_root: dir.path().to_path_buf() };
        assert!(!reader.exists("prompts/missing.md"));
    }

    #[test]
    fn same_content_produces_same_hash() {
        let (_dir, reader) = setup_temp_nucleo("deterministic content", "prompts/a.md");
        let h1 = reader.read_hash("prompts/a.md").unwrap();
        let h2 = reader.read_hash("prompts/a.md").unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_content_produces_different_hash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        std::fs::write(dir.path().join("prompts/a.md"), b"content A").unwrap();
        std::fs::write(dir.path().join("prompts/b.md"), b"content B").unwrap();
        let reader = FsPromptReader { nucleo_root: dir.path().to_path_buf() };
        let ha = reader.read_hash("prompts/a.md").unwrap();
        let hb = reader.read_hash("prompts/b.md").unwrap();
        assert_ne!(ha, hb);
    }
}
