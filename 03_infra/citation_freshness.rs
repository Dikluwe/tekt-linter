//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/citation-freshness.md
//! @prompt-hash f0941333
//! @layer L3
//! @updated 2026-08-24

use std::fs::File;
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
            if let Component::Normal(part) = component {
                clean.push(part);
            } else if component != Component::CurDir {
                return CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot);
            }
        }
        if clean.as_os_str().is_empty() {
            return CitationFreshness::Unknown(CitationUnknownReason::OutsideRoot);
        }
        let file = match open_confined(&self.root, &clean) {
            Ok(file) => file,
            Err(OpenFailure::Missing) => {
                return CitationFreshness::Stale(CitationStaleReason::MissingFile)
            }
            Err(OpenFailure::Symlink) => {
                return CitationFreshness::Unknown(CitationUnknownReason::Symlink)
            }
            Err(OpenFailure::InvalidRoot) => {
                return CitationFreshness::Unknown(CitationUnknownReason::InvalidRoot)
            }
            Err(OpenFailure::Io) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        let before = match file.metadata() {
            Ok(meta) if meta.is_file() => meta,
            Ok(_) | Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        if before.len() > self.max_bytes {
            return CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded);
        }
        let mut bytes = Vec::with_capacity(before.len() as usize);
        let mut reader = file.take(self.max_bytes.saturating_add(1));
        if reader.read_to_end(&mut bytes).is_err() {
            return CitationFreshness::Unknown(CitationUnknownReason::Io);
        }
        if bytes.len() as u64 > self.max_bytes {
            return CitationFreshness::Unknown(CitationUnknownReason::BudgetExceeded);
        }
        let after = match reader.get_ref().metadata() {
            Ok(meta) => meta,
            Err(_) => return CitationFreshness::Unknown(CitationUnknownReason::Io),
        };
        if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
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

enum OpenFailure {
    Missing,
    Symlink,
    InvalidRoot,
    Io,
}

#[cfg(target_os = "linux")]
fn open_confined(root: &Path, relative: &Path) -> Result<File, OpenFailure> {
    use std::fs;
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;

    const O_NOFOLLOW: i32 = 0x20000;
    const O_DIRECTORY: i32 = 0x10000;
    match fs::symlink_metadata(root) {
        Ok(meta) if meta.file_type().is_symlink() => return Err(OpenFailure::Symlink),
        Ok(meta) if !meta.is_dir() => return Err(OpenFailure::InvalidRoot),
        Err(_) => return Err(OpenFailure::InvalidRoot),
        _ => {}
    }
    let mut directory = File::options()
        .read(true)
        .custom_flags(O_NOFOLLOW | O_DIRECTORY)
        .open(root)
        .map_err(|error| classify_open(error, true))?;
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        let candidate = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
            .join(component.as_os_str());
        let final_component = index + 1 == components.len();
        if fs::symlink_metadata(&candidate).is_ok_and(|meta| meta.file_type().is_symlink()) {
            return Err(OpenFailure::Symlink);
        }
        let mut options = File::options();
        options.read(true).custom_flags(if final_component {
            O_NOFOLLOW
        } else {
            O_NOFOLLOW | O_DIRECTORY
        });
        let opened = options
            .open(candidate)
            .map_err(|error| classify_open(error, false))?;
        if final_component {
            return Ok(opened);
        }
        directory = opened;
    }
    Err(OpenFailure::Io)
}

#[cfg(target_os = "linux")]
fn classify_open(error: std::io::Error, root: bool) -> OpenFailure {
    match error.raw_os_error() {
        Some(40) => OpenFailure::Symlink,
        Some(2) if root => OpenFailure::InvalidRoot,
        Some(2) => OpenFailure::Missing,
        _ if root => OpenFailure::InvalidRoot,
        _ => OpenFailure::Io,
    }
}

#[cfg(not(target_os = "linux"))]
fn open_confined(_root: &Path, _relative: &Path) -> Result<File, OpenFailure> {
    Err(OpenFailure::Io)
}
