//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/prompt-reader.md
//! @prompt-hash 48a4329e
//! @layer L3
//! @updated 2026-03-13

use std::path::PathBuf;

use hex::encode;
use sha2::{Digest, Sha256};

use crate::contracts::prompt_reader::PromptReader;

use std::collections::HashMap;
use std::sync::Mutex;

/// L3 implementation of PromptReader.
/// Reads prompt files from 00_nucleo/ and returns SHA256[0..8].
/// std::io::Error is absorbed — never crosses to L1.
pub struct FsPromptReader {
    pub nucleo_root: PathBuf,
}

impl PromptReader for FsPromptReader {
    fn read_hash(&self, prompt_path: &str) -> Option<String> {
        let full_path = self.nucleo_root.join(prompt_path);
        
        // ADR-0006 compliance: prevent hashing giant files (> 10MB)
        // typically prompts are markdown files under 100KB.
        let meta = std::fs::metadata(&full_path).ok()?;
        if meta.len() > 10 * 1024 * 1024 {
            return None; 
        }

        let content = std::fs::read(&full_path).ok()?;
        let hash = Sha256::digest(&content);
        Some(encode(hash)[..8].to_string())
    }

    fn exists(&self, prompt_path: &str) -> bool {
        self.nucleo_root.join(prompt_path).exists()
    }
}

/// Decorator that caches SHA256 results to prevent redundant I/O.
/// Essential for V5/V6 performance when multiple files share a prompt.
pub struct CachedPromptReader<R: PromptReader> {
    pub inner: R,
    cache: Mutex<HashMap<String, Option<String>>>,
}

impl<R: PromptReader> CachedPromptReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            cache: Mutex::new(HashMap::new()),
        }
    }
}

impl<R: PromptReader> PromptReader for CachedPromptReader<R> {
    fn read_hash(&self, prompt_path: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(hash) = cache.get(prompt_path) {
            return hash.clone();
        }
        let hash = self.inner.read_hash(prompt_path);
        cache.insert(prompt_path.to_string(), hash.clone());
        hash
    }

    fn exists(&self, prompt_path: &str) -> bool {
        self.inner.exists(prompt_path)
    }
}

// Support Arc for sharing the cache between parallel parser threads
impl<R: PromptReader> PromptReader for std::sync::Arc<CachedPromptReader<R>> {
    fn read_hash(&self, prompt_path: &str) -> Option<String> {
        (**self).read_hash(prompt_path)
    }

    fn exists(&self, prompt_path: &str) -> bool {
        (**self).exists(prompt_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

    struct CountingReader {
        count: AtomicUsize,
    }

    impl PromptReader for CountingReader {
        fn read_hash(&self, _path: &str) -> Option<String> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Some("hash".to_string())
        }
        fn exists(&self, _path: &str) -> bool { true }
    }

    #[test]
    fn cache_prevents_multiple_reads() {
        let counter = CountingReader { count: AtomicUsize::new(0) };
        let cached = CachedPromptReader::new(counter);
        
        assert_eq!(cached.read_hash("foo"), Some("hash".to_string()));
        assert_eq!(cached.read_hash("foo"), Some("hash".to_string()));
        
        // Should only have called inner once
        assert_eq!(cached.inner.count.load(Ordering::SeqCst), 1);
    }

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
