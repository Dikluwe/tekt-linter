//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/prompt-reader.md
//! @prompt-hash 1d71982b
//! @layer L3
//! @updated 2026-03-13

use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::contracts::prompt_reader::{PromptHashResolution, PromptPresence, PromptReader};
use crate::infra::prompt_io::{probe_confined_regular_file, read_confined, without_meta_line};

use std::collections::HashMap;
use std::sync::Mutex;

/// L3 implementation of PromptReader.
/// Reads prompt files from 00_nucleo/ and returns SHA256[0..8].
/// std::io::Error is absorbed — never crosses to L1.
pub struct FsPromptReader {
    pub nucleo_root: PathBuf,
}

impl PromptReader for FsPromptReader {
    fn presence(&self, prompt_path: &str) -> PromptPresence {
        let relative = PathBuf::from(prompt_path);
        match probe_confined_regular_file(&self.nucleo_root, &relative) {
            Ok(result) => {
                if result.is_some() {
                    PromptPresence::Present
                } else {
                    PromptPresence::Missing
                }
            }
            Err(reason) => PromptPresence::InfrastructureFailure { reason },
        }
    }

    fn read_hash(&self, prompt_path: &str) -> PromptHashResolution {
        match self.presence(prompt_path) {
            PromptPresence::Missing => return PromptHashResolution::Missing,
            PromptPresence::InfrastructureFailure { reason } => {
                return PromptHashResolution::InfrastructureFailure { reason };
            }
            PromptPresence::Present => {}
        }
        if let Ok(hash) =
            crate::infra::nucleus::effective_prompt_hash_at(&self.nucleo_root, prompt_path)
        {
            return PromptHashResolution::Hash(hash);
        }
        let bytes = match read_confined(
            &self.nucleo_root,
            PathBuf::from(prompt_path).as_path(),
            10 * 1024 * 1024,
        ) {
            Ok(bytes) => bytes,
            Err(reason) => return PromptHashResolution::InfrastructureFailure { reason },
        };
        let cleaned = match without_meta_line(&bytes, b"Hash do C\xC3\xB3digo: ", false) {
            Ok(cleaned) => cleaned,
            Err(reason) => return PromptHashResolution::InfrastructureFailure { reason },
        };
        PromptHashResolution::Hash(hex::encode(Sha256::digest(cleaned))[..8].to_string())
    }
}

/// Decorator that caches SHA256 results to prevent redundant I/O.
/// Essential for V5/V6 performance when multiple files share a prompt.
pub struct CachedPromptReader<R: PromptReader> {
    pub inner: R,
    cache: Mutex<HashMap<String, PromptHashResolution>>,
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
    fn read_hash(&self, prompt_path: &str) -> PromptHashResolution {
        let mut cache = self.cache.lock().unwrap();
        if let Some(hash) = cache.get(prompt_path) {
            return hash.clone();
        }
        let hash = self.inner.read_hash(prompt_path);
        cache.insert(prompt_path.to_string(), hash.clone());
        hash
    }

    fn presence(&self, prompt_path: &str) -> PromptPresence {
        match self.read_hash(prompt_path) {
            PromptHashResolution::Hash(_) => PromptPresence::Present,
            PromptHashResolution::Missing => PromptPresence::Missing,
            PromptHashResolution::InfrastructureFailure { reason } => {
                PromptPresence::InfrastructureFailure { reason }
            }
        }
    }
}

// Support Arc for sharing the cache between parallel parser threads
impl<R: PromptReader> PromptReader for std::sync::Arc<CachedPromptReader<R>> {
    fn read_hash(&self, prompt_path: &str) -> PromptHashResolution {
        (**self).read_hash(prompt_path)
    }

    fn presence(&self, prompt_path: &str) -> PromptPresence {
        (**self).presence(prompt_path)
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

    struct MissingReader {
        count: AtomicUsize,
    }

    impl PromptReader for MissingReader {
        fn read_hash(&self, _path: &str) -> PromptHashResolution {
            self.count.fetch_add(1, Ordering::SeqCst);
            PromptHashResolution::Missing
        }
        fn presence(&self, _path: &str) -> PromptPresence {
            PromptPresence::Missing
        }
    }

    impl PromptReader for CountingReader {
        fn read_hash(&self, _path: &str) -> PromptHashResolution {
            self.count.fetch_add(1, Ordering::SeqCst);
            PromptHashResolution::Hash("hash".to_string())
        }
        fn presence(&self, _path: &str) -> PromptPresence {
            PromptPresence::Present
        }
    }

    #[test]
    fn cache_prevents_multiple_reads() {
        let counter = CountingReader {
            count: AtomicUsize::new(0),
        };
        let cached = CachedPromptReader::new(counter);

        assert_eq!(
            cached.read_hash("foo"),
            PromptHashResolution::Hash("hash".to_string())
        );
        assert_eq!(
            cached.read_hash("foo"),
            PromptHashResolution::Hash("hash".to_string())
        );

        // Should only have called inner once
        assert_eq!(cached.inner.count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cache_stabilizes_missing_result() {
        let cached = CachedPromptReader::new(MissingReader {
            count: AtomicUsize::new(0),
        });
        assert_eq!(cached.read_hash("missing"), PromptHashResolution::Missing);
        assert_eq!(cached.read_hash("missing"), PromptHashResolution::Missing);
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
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        (dir, reader)
    }

    #[test]
    fn read_hash_returns_eight_hex_chars() {
        let (_dir, reader) = setup_temp_nucleo("# some prompt content", "prompts/auth.md");
        let PromptHashResolution::Hash(hash) = reader.read_hash("prompts/auth.md") else {
            panic!("hash expected")
        };
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn read_hash_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert_eq!(
            reader.read_hash("prompts/missing.md"),
            PromptHashResolution::Missing
        );
    }

    #[test]
    fn exists_returns_true_when_present() {
        let (_dir, reader) = setup_temp_nucleo("content", "prompts/test.md");
        assert_eq!(reader.presence("prompts/test.md"), PromptPresence::Present);
    }

    #[test]
    fn exists_returns_false_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert_eq!(
            reader.presence("prompts/missing.md"),
            PromptPresence::Missing
        );
    }

    #[test]
    fn same_content_produces_same_hash() {
        let (_dir, reader) = setup_temp_nucleo("deterministic content", "prompts/a.md");
        let PromptHashResolution::Hash(h1) = reader.read_hash("prompts/a.md") else {
            panic!("hash expected")
        };
        let PromptHashResolution::Hash(h2) = reader.read_hash("prompts/a.md") else {
            panic!("hash expected")
        };
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_content_produces_different_hash() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        std::fs::write(dir.path().join("prompts/a.md"), b"content A").unwrap();
        std::fs::write(dir.path().join("prompts/b.md"), b"content B").unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        let PromptHashResolution::Hash(ha) = reader.read_hash("prompts/a.md") else {
            panic!("hash expected")
        };
        let PromptHashResolution::Hash(hb) = reader.read_hash("prompts/b.md") else {
            panic!("hash expected")
        };
        assert_ne!(ha, hb);
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_root_is_infrastructure_failure_never_missing() {
        use std::os::unix::fs::symlink;

        let parent = tempfile::tempdir().unwrap();
        let real = parent.path().join("real");
        std::fs::create_dir_all(real.join("prompts")).unwrap();
        std::fs::write(real.join("prompts/a.md"), b"prompt").unwrap();
        let alias = parent.path().join("alias");
        symlink(&real, &alias).unwrap();
        let reader = FsPromptReader { nucleo_root: alias };

        assert!(matches!(
            reader.presence("prompts/a.md"),
            PromptPresence::InfrastructureFailure { .. }
        ));
        assert!(matches!(
            reader.read_hash("prompts/a.md"),
            PromptHashResolution::InfrastructureFailure { .. }
        ));
    }

    #[test]
    fn missing_leaf_under_valid_root_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert_eq!(
            reader.presence("prompts/missing.md"),
            PromptPresence::Missing
        );
        assert_eq!(
            reader.read_hash("prompts/missing.md"),
            PromptHashResolution::Missing
        );
    }

    #[test]
    fn missing_ancestor_is_infrastructure_not_missing() {
        let dir = tempfile::tempdir().unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert!(matches!(
            reader.presence("prompts/missing.md"),
            PromptPresence::InfrastructureFailure { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_prompt_leaf_is_infrastructure_not_missing() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        symlink("absent.md", dir.path().join("prompts/link.md")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert!(matches!(
            reader.presence("prompts/link.md"),
            PromptPresence::InfrastructureFailure { .. }
        ));
    }

    #[test]
    fn unsafe_missing_paths_are_infrastructure_not_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        for path in [
            "../missing.md",
            "/tmp/missing-p0117.md",
            "prompts/../missing.md",
        ] {
            assert!(
                matches!(
                    reader.presence(path),
                    PromptPresence::InfrastructureFailure { .. }
                ),
                "unsafe path classified incorrectly: {path}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn missing_leaf_below_symbolic_ancestor_is_infrastructure() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        symlink(outside.path(), dir.path().join("prompts/link")).unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert!(matches!(
            reader.presence("prompts/link/missing.md"),
            PromptPresence::InfrastructureFailure { .. }
        ));
    }

    #[test]
    fn integrated_cache_never_diverges_after_filesystem_change() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let cached = CachedPromptReader::new(FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        });
        assert_eq!(cached.presence("prompts/later.md"), PromptPresence::Missing);
        std::fs::write(dir.path().join("prompts/later.md"), b"created later").unwrap();
        assert_eq!(
            cached.read_hash("prompts/later.md"),
            PromptHashResolution::Missing
        );
        assert_eq!(cached.presence("prompts/later.md"), PromptPresence::Missing);
    }

    #[test]
    fn integrated_cache_freezes_present_before_filesystem_removal() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        let path = dir.path().join("prompts/present.md");
        std::fs::write(&path, b"present now").unwrap();
        let cached = CachedPromptReader::new(FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        });
        assert_eq!(
            cached.presence("prompts/present.md"),
            PromptPresence::Present
        );
        std::fs::remove_file(path).unwrap();
        assert!(matches!(
            cached.read_hash("prompts/present.md"),
            PromptHashResolution::Hash(_)
        ));
        assert_eq!(
            cached.presence("prompts/present.md"),
            PromptPresence::Present
        );
    }

    #[test]
    fn hash_budget_is_exactly_ten_mebibytes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("prompts")).unwrap();
        std::fs::write(
            dir.path().join("prompts/exact.md"),
            vec![b'a'; 10 * 1024 * 1024],
        )
        .unwrap();
        std::fs::write(
            dir.path().join("prompts/over.md"),
            vec![b'a'; 10 * 1024 * 1024 + 1],
        )
        .unwrap();
        let reader = FsPromptReader {
            nucleo_root: dir.path().to_path_buf(),
        };
        assert!(matches!(
            reader.read_hash("prompts/exact.md"),
            PromptHashResolution::Hash(_)
        ));
        assert!(matches!(
            reader.read_hash("prompts/over.md"),
            PromptHashResolution::InfrastructureFailure { .. }
        ));

        // Invalid UTF-8 makes the nucleus-aware route decline the input; the
        // byte-oriented legacy route must retain the same ten-MiB budget.
        let mut legacy = vec![b'a'; 2 * 1024 * 1024];
        legacy[0] = 0xff;
        std::fs::write(dir.path().join("prompts/legacy.md"), legacy).unwrap();
        assert!(matches!(
            reader.read_hash("prompts/legacy.md"),
            PromptHashResolution::Hash(_)
        ));
    }
}
