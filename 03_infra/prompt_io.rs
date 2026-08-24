//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/file-walker.md
//! @prompt-hash b60b4c20
//! @layer L3
//! @updated 2026-08-24

use std::fs::{self, OpenOptions, Permissions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn validate_local_root(root: &Path) -> Result<(), String> {
    if root.as_os_str().is_empty() {
        return Err("prompt root must not be empty".to_string());
    }
    let mut current = if root.is_absolute() {
        PathBuf::new()
    } else {
        PathBuf::from(".")
    };
    for component in root.components() {
        match component {
            Component::CurDir => continue,
            Component::Prefix(_)
            | Component::RootDir
            | Component::ParentDir
            | Component::Normal(_) => {
                current.push(component.as_os_str());
            }
        }
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            format!("cannot inspect prompt root {}: {error}", current.display())
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "prompt root component must be a local non-symlink directory: {}",
                current.display()
            ));
        }
    }
    Ok(())
}

pub fn confined_regular_file(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || !relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(format!("unsafe prompt path `{}`", relative.display()));
    }
    validate_local_root(root)?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve prompt root: {error}"))?;
    let mut current = root.clone();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            format!("cannot inspect prompt path {}: {error}", current.display())
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "prompt path contains symlink: {}",
                current.display()
            ));
        }
    }
    let canonical = current
        .canonicalize()
        .map_err(|error| format!("cannot resolve prompt path: {error}"))?;
    if !canonical.starts_with(&root) {
        return Err("prompt path escapes root".to_string());
    }
    let metadata = fs::symlink_metadata(&canonical).map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file() {
        return Err("prompt path is not a regular local file".to_string());
    }
    Ok(canonical)
}

pub fn read_confined(root: &Path, relative: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let path = confined_regular_file(root, relative)?;
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() > limit {
        return Err("prompt input budget exceeded".to_string());
    }
    Ok(bytes)
}

pub fn atomic_replace(path: &Path, bytes: &[u8], permissions: Permissions) -> Result<(), String> {
    let parent = path.parent().ok_or("output has no parent")?;
    let name = path
        .file_name()
        .ok_or("output has no name")?
        .to_string_lossy();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), sequence));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        file.set_permissions(permissions)
            .map_err(|e| e.to_string())?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn eight_hex(value: &str) -> bool {
    value.len() == 8
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn without_meta_line(bytes: &[u8], marker: &[u8], required: bool) -> Result<Vec<u8>, String> {
    rewrite_meta_line(bytes, marker, None, required)
}

pub fn replace_meta_line(
    bytes: &[u8],
    marker: &[u8],
    hash: &[u8],
    required: bool,
) -> Result<Vec<u8>, String> {
    rewrite_meta_line(bytes, marker, Some(hash), required)
}

fn rewrite_meta_line(
    bytes: &[u8],
    marker: &[u8],
    replacement: Option<&[u8]>,
    required: bool,
) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut count = 0;
    let prompt_meta = marker.starts_with("Hash do C".as_bytes());
    let mut prompt_preamble = true;
    let mut source_header = true;
    for (index, line) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
        let body = without_lf.strip_suffix(b"\r").unwrap_or(without_lf);
        let (bom, inspected) = if index == 0 {
            body.strip_prefix(b"\xEF\xBB\xBF")
                .map_or((&[][..], body), |rest| (&b"\xEF\xBB\xBF"[..], rest))
        } else {
            (&[][..], body)
        };
        let authorized = if prompt_meta {
            prompt_preamble
                && !inspected.trim_ascii_start().starts_with(b"```")
                && !inspected.trim_ascii_start().starts_with(b"~~~")
        } else {
            source_header && inspected.starts_with(b"//!")
        };
        let label = marker.strip_suffix(b" ").unwrap_or(marker);
        if authorized && inspected.starts_with(label) && !inspected.starts_with(marker) {
            return Err("malformed canonical hash metadata".to_string());
        }
        if authorized && inspected.starts_with(marker) {
            let hash = &inspected[marker.len()..];
            if hash.len() != 8
                || !hash
                    .iter()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("malformed canonical hash metadata".to_string());
            }
            count += 1;
            if let Some(replacement) = replacement {
                output.extend_from_slice(bom);
                output.extend_from_slice(marker);
                output.extend_from_slice(replacement);
                if without_lf.ends_with(b"\r") {
                    output.push(b'\r');
                }
                if line.ends_with(b"\n") {
                    output.push(b'\n');
                }
            }
        } else {
            output.extend_from_slice(line);
        }
        if prompt_meta {
            if inspected.is_empty()
                || inspected.trim_ascii_start().starts_with(b"```")
                || inspected.trim_ascii_start().starts_with(b"~~~")
            {
                prompt_preamble = false;
            }
        } else if !inspected.starts_with(b"//!") {
            source_header = false;
        }
    }
    if count > 1 || (required && count != 1) {
        return Err("canonical hash metadata must occur exactly once".to_string());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_filter_preserves_crlf_and_rejects_duplicate_metadata() {
        let bytes = b"# title\r\nHash do C\xC3\xB3digo: abcdef12\r\nbody\r\n";
        assert_eq!(
            without_meta_line(bytes, b"Hash do C\xC3\xB3digo: ", false).unwrap(),
            b"# title\r\nbody\r\n"
        );
        let duplicate = [bytes.as_slice(), b"Hash do C\xC3\xB3digo: 12345678\n"].concat();
        assert!(without_meta_line(&duplicate, b"Hash do C\xC3\xB3digo: ", false).is_err());
    }

    #[test]
    fn body_decoy_is_preserved_as_ordinary_bytes() {
        let bytes = b"# Title\nHash do C\xC3\xB3digo: abcdef12\n\nBody mentions Hash do C\xC3\xB3digo: as prose.\n";
        assert_eq!(
            without_meta_line(bytes, b"Hash do C\xC3\xB3digo: ", false).unwrap(),
            b"# Title\n\nBody mentions Hash do C\xC3\xB3digo: as prose.\n"
        );
    }

    #[test]
    fn metadata_in_body_fence_is_not_canonical() {
        let bytes = b"# Title\n\n~~~text\nHash do C\xC3\xB3digo: abcdef12\n~~~\n";
        assert_eq!(
            without_meta_line(bytes, b"Hash do C\xC3\xB3digo: ", false).unwrap(),
            bytes
        );
    }

    #[test]
    fn source_metadata_after_header_is_not_replaced() {
        let bytes = b"//! @prompt-hash abcdef12\nfn main() {}\n//! @prompt-hash 12345678\n";
        let replaced = replace_meta_line(bytes, b"//! @prompt-hash ", b"87654321", true).unwrap();
        assert_eq!(
            replaced,
            b"//! @prompt-hash 87654321\nfn main() {}\n//! @prompt-hash 12345678\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn confined_file_rejects_symlink() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        symlink(outside.path(), root.path().join("prompt.md")).unwrap();
        assert!(confined_regular_file(root.path(), Path::new("prompt.md")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn confined_file_rejects_symlink_root() {
        use std::os::unix::fs::symlink;
        let parent = tempfile::tempdir().unwrap();
        let real = tempfile::tempdir().unwrap();
        std::fs::write(real.path().join("prompt.md"), b"prompt").unwrap();
        let linked = parent.path().join("root");
        symlink(real.path(), &linked).unwrap();
        assert!(confined_regular_file(&linked, Path::new("prompt.md")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn confined_file_rejects_symlink_in_root_ancestor() {
        use std::os::unix::fs::symlink;
        let parent = tempfile::tempdir().unwrap();
        let real = tempfile::tempdir().unwrap();
        std::fs::create_dir(real.path().join("nucleo")).unwrap();
        std::fs::write(real.path().join("nucleo/prompt.md"), b"prompt").unwrap();
        let linked = parent.path().join("linked");
        symlink(real.path(), &linked).unwrap();
        assert!(confined_regular_file(&linked.join("nucleo"), Path::new("prompt.md")).is_err());
    }

    #[test]
    fn current_directory_root_remains_valid() {
        let relative = Path::new("Cargo.toml");
        assert!(confined_regular_file(Path::new("."), relative).is_ok());
    }
}
