//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/infra/hash-writer.md
//! @prompt-hash c1a72345
//! @layer L3
//! @updated 2026-03-13

use crate::entities::hash_pair::{BijectivePair, PairSnapshot};
use crate::infra::prompt_io::{atomic_replace, eight_hex, replace_meta_line, without_meta_line};
use sha2::{Digest, Sha256};
use std::path::Path;

// ── Public API ────────────────────────────────────────────────────────────────

/// Compute SHA256[0..8] of the source file, ignoring its own `@prompt-hash` line.
/// Returns None if the file cannot be read.
pub fn compute_source_hash(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let cleaned = without_meta_line(&bytes, b"//! @prompt-hash ", true).ok()?;
    Some(hex::encode(Sha256::digest(cleaned))[..8].to_string())
}

/// Read the `@prompt` path and current `@prompt-hash` value from a source file header.
/// Scans only the leading `//!` block — stops at the first non-`//!` line.
/// Returns None if the file cannot be read or has no `@prompt` line.
pub fn read_header(path: &Path) -> Option<(String, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    parse_header(&content)
}

/// Atomically replace `//! @prompt-hash <old>` with `//! @prompt-hash <new>` in a source file.
///
/// Atomic strategy: write to a sibling temp file, then `std::fs::rename`.
/// If rename fails, the temp file is cleaned up and the original is untouched.
pub fn write_hash(path: &Path, new_hash: &str) -> Result<(), String> {
    if !eight_hex(new_hash) {
        return Err("hash must be exactly eight lowercase hex digits".to_string());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let permissions = std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions();
    let replaced = replace_meta_line(&bytes, b"//! @prompt-hash ", new_hash.as_bytes(), true)?;
    atomic_replace(path, &replaced, permissions)
}

/// Atomically replace the authorized "Hash do Código: <hash>" prompt metadata.
pub fn write_prompt_meta(path: &Path, code_hash: &str) -> Result<(), String> {
    if !eight_hex(code_hash) {
        return Err("hash must be exactly eight lowercase hex digits".to_string());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let permissions = std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions();
    let marker = "Hash do Código: ".as_bytes();
    let replaced = replace_meta_line(&bytes, marker, code_hash.as_bytes(), true)?;
    atomic_replace(path, &replaced, permissions)
}

pub fn prepare_pair(
    root: &Path,
    source_path: &Path,
    prompt_path: &str,
    full_prompt_path: &Path,
    old_prompt_hash: &str,
    _new_prompt_hash: &str,
    new_source_hash: &str,
) -> Result<BijectivePair, String> {
    let source_bytes = std::fs::read(source_path).map_err(|error| error.to_string())?;
    let prompt_bytes = std::fs::read(full_prompt_path).map_err(|error| error.to_string())?;
    let (prompt_with_current_pins, nucleus_dependencies) =
        crate::infra::nucleus::refresh_prompt_nucleus_pins(root, &prompt_bytes)?;
    let final_prompt_hash = crate::infra::nucleus::effective_prompt_hash(
        &prompt_with_current_pins,
        &nucleus_dependencies,
    )?;
    let new_source_bytes = replace_meta_line(
        &source_bytes,
        b"//! @prompt-hash ",
        final_prompt_hash.as_bytes(),
        true,
    )?;
    let new_prompt_bytes = replace_meta_line(
        &prompt_with_current_pins,
        "Hash do Código: ".as_bytes(),
        new_source_hash.as_bytes(),
        true,
    )?;
    Ok(BijectivePair {
        source_path: source_path.to_path_buf(),
        prompt_path: prompt_path.to_owned(),
        old_prompt_hash: old_prompt_hash.to_owned(),
        new_prompt_hash: final_prompt_hash,
        new_source_hash: new_source_hash.to_owned(),
        new_source_bytes,
        new_prompt_bytes,
    })
}

pub fn snapshot_pair(source_path: &Path, prompt_path: &Path) -> Result<PairSnapshot, String> {
    Ok(PairSnapshot {
        source_bytes: std::fs::read(source_path).map_err(|error| error.to_string())?,
        prompt_bytes: std::fs::read(prompt_path).map_err(|error| error.to_string())?,
    })
}

pub fn write_pair(
    source_path: &Path,
    prompt_path: &Path,
    snapshot: &PairSnapshot,
) -> Result<(), String> {
    let source_permissions = std::fs::metadata(source_path)
        .map_err(|error| error.to_string())?
        .permissions();
    let prompt_permissions = std::fs::metadata(prompt_path)
        .map_err(|error| error.to_string())?
        .permissions();
    atomic_replace(source_path, &snapshot.source_bytes, source_permissions)?;
    if let Err(reason) = atomic_replace(prompt_path, &snapshot.prompt_bytes, prompt_permissions) {
        return Err(reason);
    }
    Ok(())
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn parse_header(source: &str) -> Option<(String, String)> {
    let mut prompt_path: Option<String> = None;
    let mut old_hash = String::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//!") {
            break;
        }
        let content = trimmed.trim_start_matches("//!").trim();

        if let Some(val) = content.strip_prefix("@prompt-hash ") {
            old_hash = val.trim().to_string();
        } else if let Some(val) = content.strip_prefix("@prompt ") {
            prompt_path = Some(val.trim().to_string());
        }
    }

    Some((prompt_path?, old_hash))
}

#[cfg(test)]
fn replace_hash_line(content: &str, new_hash: &str) -> Option<String> {
    let mut found = false;

    let replaced: Vec<String> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if !found && trimmed.starts_with("//!") && trimmed.contains("@prompt-hash") {
                found = true;
                // Preserve original leading whitespace so indented string literals survive
                let indent_len = line.len() - line.trim_start().len();
                format!("{}//! @prompt-hash {}", &line[..indent_len], new_hash)
            } else {
                line.to_string()
            }
        })
        .collect();

    if !found {
        return None;
    }

    let trailing_newline = if content.ends_with('\n') { "\n" } else { "" };
    Some(format!("{}{}", replaced.join("\n"), trailing_newline))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_temp(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    const HEADER: &str = "//! Crystalline Lineage\n\
//! @prompt 00_nucleo/prompts/linter-core.md\n\
//! @prompt-hash 7ed43b44\n\
//! @layer L1\n\
//! @updated 2026-03-13\n\
\n\
fn foo() {}\n";

    // ── parse_header ──────────────────────────────────────────────────────────

    #[test]
    fn parse_header_extracts_prompt_path_and_hash() {
        let result = parse_header(HEADER).unwrap();
        assert_eq!(result.0, "00_nucleo/prompts/linter-core.md");
        assert_eq!(result.1, "7ed43b44");
    }

    #[test]
    fn parse_header_returns_none_without_prompt_line() {
        let source = "//! @prompt-hash 00000000\nfn foo() {}\n";
        assert!(parse_header(source).is_none());
    }

    #[test]
    fn parse_header_empty_hash_when_no_hash_line() {
        let source = "//! @prompt 00_nucleo/prompts/foo.md\nfn foo() {}\n";
        let result = parse_header(source).unwrap();
        assert_eq!(result.0, "00_nucleo/prompts/foo.md");
        assert_eq!(result.1, ""); // no hash line → empty string
    }

    #[test]
    fn parse_header_stops_at_non_doc_comment() {
        // @prompt after a blank line is NOT part of the header
        let source = "//! @prompt foo.md\n\nfn bar() {}\n//! @prompt should-not-parse.md\n";
        let result = parse_header(source).unwrap();
        assert_eq!(result.0, "foo.md");
    }

    // ── replace_hash_line ─────────────────────────────────────────────────────

    #[test]
    fn replace_hash_line_substitutes_correctly() {
        let new = replace_hash_line(HEADER, "a3f8c2d1").unwrap();
        assert!(new.contains("//! @prompt-hash a3f8c2d1"));
        assert!(!new.contains("00000000"));
    }

    #[test]
    fn replace_hash_line_preserves_trailing_newline() {
        let new = replace_hash_line(HEADER, "a3f8c2d1").unwrap();
        assert!(new.ends_with('\n'));
    }

    #[test]
    fn replace_hash_line_no_trailing_newline_when_absent() {
        let source = "//! @prompt-hash 00000000\nfn foo() {}";
        let new = replace_hash_line(source, "a3f8c2d1").unwrap();
        assert!(!new.ends_with('\n'));
    }

    #[test]
    fn replace_hash_line_returns_none_when_no_hash_line() {
        let source = "fn foo() {}\n";
        assert!(replace_hash_line(source, "a3f8c2d1").is_none());
    }

    #[test]
    fn replace_hash_line_only_changes_hash_value() {
        let new = replace_hash_line(HEADER, "a3f8c2d1").unwrap();
        assert!(new.contains("@prompt 00_nucleo/prompts/linter-core.md"));
        assert!(new.contains("@layer L1"));
        assert!(new.contains("fn foo()"));
    }

    // ── write_hash (disk) ─────────────────────────────────────────────────────

    #[test]
    fn write_hash_updates_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(&dir, "layer.rs", HEADER);

        write_hash(&path, "a3f8c2d1").unwrap();

        let updated = fs::read_to_string(&path).unwrap();
        assert!(updated.contains("//! @prompt-hash a3f8c2d1"));
        assert!(!updated.contains("00000000"));
    }

    #[test]
    fn write_hash_preserves_bom_crlf_and_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("layer.rs");
        let original =
            b"\xEF\xBB\xBF//! Crystalline Lineage\r\n//! @prompt-hash 12345678\r\nfn main() {}\r\n";
        fs::write(&path, original).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        }
        write_hash(&path, "abcdef12").unwrap();
        let updated = fs::read(&path).unwrap();
        assert_eq!(
            updated,
            b"\xEF\xBB\xBF//! Crystalline Lineage\r\n//! @prompt-hash abcdef12\r\nfn main() {}\r\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
    }

    #[test]
    fn write_prompt_meta_preserves_body_decoy() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            &dir,
            "prompt.md",
            "# Prompt\nHash do Código: 12345678\n\nA frase Hash do Código: aqui é conteúdo.\n",
        );
        write_prompt_meta(&path, "abcdef12").unwrap();
        let updated = fs::read_to_string(path).unwrap();
        assert!(updated.contains("Hash do Código: abcdef12"));
        assert!(updated.contains("A frase Hash do Código: aqui é conteúdo."));
    }

    #[test]
    fn write_prompt_meta_rejects_absent_meta_despite_body_substring_decoy() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            &dir,
            "prompt.md",
            "# Prompt\n\nA frase Hash do Código: aqui é conteúdo.\n",
        );
        let original = fs::read(&path).unwrap();
        #[cfg(unix)]
        let original_mode = {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(&path).unwrap().permissions().mode()
        };
        assert!(write_prompt_meta(&path, "abcdef12").is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode(),
                original_mode
            );
        }
    }

    #[test]
    fn write_prompt_meta_preserves_crlf_exactly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("prompt.md");
        fs::write(
            &path,
            b"# Prompt\r\nHash do C\xC3\xB3digo: 12345678\r\n\r\nBody\r\n",
        )
        .unwrap();
        write_prompt_meta(&path, "abcdef12").unwrap();
        assert_eq!(
            fs::read(path).unwrap(),
            b"# Prompt\r\nHash do C\xC3\xB3digo: abcdef12\r\n\r\nBody\r\n"
        );
    }

    #[test]
    fn write_hash_never_replaces_body_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(
            &dir,
            "layer.rs",
            "//! @prompt-hash 12345678\nfn main() {}\n//! @prompt-hash deadbeef\n",
        );
        write_hash(&path, "abcdef12").unwrap();
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            "//! @prompt-hash abcdef12\nfn main() {}\n//! @prompt-hash deadbeef\n"
        );
    }

    #[test]
    fn write_hash_is_atomic_original_intact_when_no_hash_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(&dir, "plain.rs", "fn foo() {}\n");

        let result = write_hash(&path, "a3f8c2d1");
        assert!(result.is_err());

        // Original untouched
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "fn foo() {}\n");
    }

    #[test]
    fn write_hash_leaves_no_temp_file_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(&dir, "layer.rs", HEADER);

        write_hash(&path, "a3f8c2d1").unwrap();

        let remaining: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(".crystalline-tmp-")
            })
            .collect();

        assert!(remaining.is_empty(), "Temp file was not cleaned up");
    }

    #[test]
    fn read_header_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp(&dir, "layer.rs", HEADER);

        let (prompt_path, old_hash) = read_header(&path).unwrap();
        assert_eq!(prompt_path, "00_nucleo/prompts/linter-core.md");
        assert_eq!(old_hash, "7ed43b44");
    }

    #[test]
    fn read_header_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.rs");
        assert!(read_header(&path).is_none());
    }
}
