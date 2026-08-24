//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/contracts/citation-freshness.md
//! @prompt-hash 5a68af8d
//! @layer L3
//! @updated 2026-08-24

use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::contracts::citation_freshness::{
    CitationFreshness, CitationFreshnessResolver, CitationStaleReason, CitationUnknownReason,
};

pub struct FsCitationFreshnessResolver {
    root: PathBuf,
    max_bytes: u64,
}

impl FsCitationFreshnessResolver {
    pub fn new(root: PathBuf, max_bytes: u64) -> Self {
        Self { root, max_bytes }
    }
}

impl CitationFreshnessResolver for FsCitationFreshnessResolver {
    fn resolve(&self, path: &str, line: usize) -> CitationFreshness {
        if self.max_bytes == 0 {
            return CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded);
        }
        let relative = Path::new(path);
        if path.is_empty() || relative.is_absolute() {
            return CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot);
        }
        let mut clean = PathBuf::new();
        for component in relative.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                _ => return CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot),
            }
        }
        if clean.as_os_str().is_empty() {
            return CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot);
        }
        let root_meta = match fs::symlink_metadata(&self.root) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return CitationFreshness::Unknown(CitationUnknownReason::Symlink)
            }
            Ok(meta) if meta.is_dir() => meta,
            Ok(_) | Err(_) => {
                return CitationFreshness::Unknown(CitationUnknownReason::InvalidRoot)
            }
        };
        let mut cursor = self.root.clone();
        for component in clean.components() {
            cursor.push(component.as_os_str());
            match fs::symlink_metadata(&cursor) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    return CitationFreshness::Unknown(CitationUnknownReason::Symlink)
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return CitationFreshness::Stale(CitationStaleReason::MissingFile)
                }
                Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
            }
        }
        let before = match fs::metadata(&cursor) {
            Ok(meta) if meta.is_file() => meta,
            Ok(_) | Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        if before.len() > self.max_bytes {
            return CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded);
        }
        let mut bytes = Vec::with_capacity(before.len() as usize);
        let mut reader = match File::open(&cursor) {
            Ok(file) => file.take(self.max_bytes.saturating_add(1)),
            Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        if reader.read_to_end(&mut bytes).is_err() {
            return CitationFreshness::Unknown(CitationUnknownReason::Io);
        }
        if bytes.len() as u64 > self.max_bytes {
            return CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded);
        }
        let after = match fs::metadata(&cursor) {
            Ok(meta) => meta,
            Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        if before.len() != after.len()
            || before.modified().ok() != after.modified().ok()
            || root_meta.modified().ok()
                != fs::metadata(&self.root)
                    .ok()
                    .and_then(|m| m.modified().ok())
        {
            return CitationFreshness::Unknown(CitationUnknownReason::ConcurrentMutation);
        }
        let content = match std::str::from_utf8(&bytes) {
            Ok(content) => content,
            Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::InvalidUtf8),
        };
        if line == 0 {
            return CitationFreshness::Stale(CitationStaleReason::InvalidLine);
        }
        match content.lines().nth(line - 1) {
            None => CitationFreshness::Stale(CitationStaleReason::InvalidLine),
            Some(value) if value.trim().is_empty() => {
                CitationFreshness::Stale(CitationStaleReason::EmptyLine)
            }
            Some(_) => CitationFreshness::Valid,
        }
    }
}
