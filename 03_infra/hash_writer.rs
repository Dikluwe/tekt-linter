//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/fix-hashes.md
//! @prompt-hash 929d9c47
//! @layer L3
//! @updated 2026-03-13

use std::path::Path;

// ── Public API ────────────────────────────────────────────────────────────────

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
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let new_content = replace_hash_line(&content, new_hash)
        .ok_or_else(|| "No @prompt-hash line found in file".to_string())?;

    let dir = path.parent().ok_or_else(|| "File has no parent directory".to_string())?;
    let tmp_path = dir.join(format!(".crystalline-tmp-{}", std::process::id()));

    std::fs::write(&tmp_path, new_content.as_bytes()).map_err(|e| e.to_string())?;

    std::fs::rename(&tmp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp_path);
        e.to_string()
    })?;

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
