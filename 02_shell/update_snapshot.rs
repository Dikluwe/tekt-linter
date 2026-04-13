//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/fix-hashes.md
//! @prompt-hash 8b5b716b
//! @layer L2
//! @updated 2026-03-20

use std::path::PathBuf;

use colored::Colorize;

use crate::entities::parsed_file::{ParsedFile, PublicInterface};
use crate::entities::violation::Violation;

// ── Outbound port (implemented by L4 adapter wrapping L3) ────────────────────

/// L2-defined contract for serializing and writing Interface Snapshots.
/// L3 provides the concrete I/O. L4 creates the adapter.
pub trait SnapshotRewriter {
    /// Serialize a PublicInterface to the canonical snapshot section format.
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String;

    /// Atomically write the snapshot section to the prompt file.
    fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String>;
}

// ── Data types ────────────────────────────────────────────────────────────────

pub struct SnapshotEntry {
    pub source_path: PathBuf,
    /// Empty when unreadable_reason is set.
    pub prompt_path: String,
    /// Empty when unreadable_reason is set.
    pub new_snapshot: String,
    /// Set when the file has no parsed record or no prompt header.
    pub unreadable_reason: Option<String>,
}

pub struct SnapshotResult {
    pub source_path: PathBuf,
    pub prompt_path: String,
    pub success: bool,
    pub error: Option<String>,
}

// ── Core functions ────────────────────────────────────────────────────────────

/// Build snapshot entries from V6 violations + the corresponding ParsedFiles.
/// Entries where the ParsedFile or PromptHeader cannot be found are included with
/// `unreadable_reason` set, rather than silently discarded.
pub fn plan<'a>(
    violations: &[Violation<'a>],
    parsed_files: &[ParsedFile<'a>],
    rewriter: &dyn SnapshotRewriter,
) -> Vec<SnapshotEntry> {
    violations
        .iter()
        .filter(|v| v.rule_id == "V6")
        .map(|v| {
            let Some(parsed) = parsed_files.iter().find(|p| p.path == v.location.path.as_ref())
            else {
                return SnapshotEntry {
                    source_path: v.location.path.to_path_buf(),
                    prompt_path: String::new(),
                    new_snapshot: String::new(),
                    unreadable_reason: Some("no parsed file found for violation path".to_string()),
                };
            };
            let Some(header) = parsed.prompt_header.as_ref() else {
                return SnapshotEntry {
                    source_path: v.location.path.to_path_buf(),
                    prompt_path: String::new(),
                    new_snapshot: String::new(),
                    unreadable_reason: Some("file has no @prompt header".to_string()),
                };
            };
            let new_snapshot = rewriter.serialize_snapshot(&parsed.public_interface);
            SnapshotEntry {
                source_path: v.location.path.to_path_buf(),
                prompt_path: header.prompt_path.to_string(),
                new_snapshot,
                unreadable_reason: None,
            }
        })
        .collect()
}

/// Execute or dry-run the snapshot updates.
pub fn execute(
    entries: &[SnapshotEntry],
    rewriter: &dyn SnapshotRewriter,
    dry_run: bool,
) -> Vec<SnapshotResult> {
    entries
        .iter()
        .filter_map(|entry| {
            // Entradas com unreadable_reason não podem ser actualizadas — saltar
            if entry.unreadable_reason.is_some() {
                return None;
            }
            if dry_run {
                return Some(SnapshotResult {
                    source_path: entry.source_path.clone(),
                    prompt_path: entry.prompt_path.clone(),
                    success: true,
                    error: None,
                });
            }
            let outcome = rewriter.write_snapshot(&entry.prompt_path, &entry.new_snapshot);
            Some(SnapshotResult {
                source_path: entry.source_path.clone(),
                prompt_path: entry.prompt_path.clone(),
                success: outcome.is_ok(),
                error: outcome.err(),
            })
        })
        .collect()
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_plan(entries: &[SnapshotEntry]) -> String {
    let actionable: Vec<_> = entries.iter().filter(|e| e.unreadable_reason.is_none()).collect();
    let unreadable: Vec<_> = entries.iter().filter(|e| e.unreadable_reason.is_some()).collect();

    if entries.is_empty() {
        return format!("{}\n", "Nothing to update".green().bold());
    }

    let mut out = String::new();

    if !actionable.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Would update snapshot in".cyan().bold(),
            actionable.len(),
            if actionable.len() == 1 { "file" } else { "files" }
        ));
        for entry in &actionable {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                entry.source_path.display(),
                entry.prompt_path
            ));
        }
    }

    if !unreadable.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{} {} (no parsed record or header):\n",
            "Skipped".yellow().bold(),
            unreadable.len()
        ));
        for entry in unreadable {
            out.push_str(&format!(
                "  {} — {}\n",
                entry.source_path.display(),
                entry.unreadable_reason.as_deref().unwrap_or("unknown"),
            ));
        }
    }

    out
}

pub fn format_results(results: &[SnapshotResult], remaining_v6: usize) -> String {
    if results.is_empty() {
        return format!("{}\n", "Nothing to update".green().bold());
    }

    let mut out = String::new();
    let succeeded: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    if !succeeded.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Updated snapshot in".green().bold(),
            succeeded.len(),
            if succeeded.len() == 1 { "file" } else { "files" }
        ));
        for r in &succeeded {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                r.source_path.display(),
                r.prompt_path
            ));
        }
    }

    if !failed.is_empty() {
        out.push('\n');
        out.push_str(&format!("{} {} failed:\n", "Error".red().bold(), failed.len()));
        for r in &failed {
            out.push_str(&format!(
                "  {} — {}\n",
                r.source_path.display(),
                r.error.as_deref().unwrap_or("unknown error"),
            ));
        }
    }

    out.push('\n');
    if remaining_v6 == 0 {
        out.push_str(&format!(
            "Re-running analysis... {} 0 stale warnings remaining\n",
            "✅".green()
        ));
    } else {
        out.push_str(&format!(
            "Re-running analysis... {} {} stale warning(s) remaining\n",
            "⚠".yellow(),
            remaining_v6,
        ));
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::layer::{Language, Layer};
    use crate::entities::parsed_file::{PromptHeader, PublicInterface};
    use crate::entities::violation::{Location, ViolationLevel};
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    struct MockRewriter {
        write_calls: RefCell<Vec<(String, String)>>,
        write_result: Result<(), String>,
    }

    impl MockRewriter {
        fn new(write_result: Result<(), String>) -> Self {
            Self { write_calls: RefCell::new(vec![]), write_result }
        }
    }

    impl SnapshotRewriter for MockRewriter {
        fn serialize_snapshot(&self, _: &PublicInterface<'_>) -> String {
            "## Interface Snapshot\n<!-- crystalline-snapshot: {} -->".to_string()
        }
        fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String> {
            self.write_calls
                .borrow_mut()
                .push((prompt_path.to_string(), snapshot.to_string()));
            self.write_result.clone()
        }
    }

    fn v6_violation(path: &'static str) -> Violation<'static> {
        Violation {
            rule_id: "V6".to_string(),
            level: ViolationLevel::Warning,
            message: "stale".to_string(),
            location: Location { path: Cow::Borrowed(Path::new(path)), line: 1, column: 0 },
        }
    }

    fn parsed_file_for(path: &'static str) -> ParsedFile<'static> {
        ParsedFile {
            path: Path::new(path),
            layer: Layer::L1,
            language: Language::Rust,
            prompt_header: Some(PromptHeader {
                prompt_path: "00_nucleo/prompts/foo.md",
                prompt_hash: None,
                current_hash: None,
                layer: Layer::L1,
                updated: None,
            }),
            prompt_file_exists: true,
            has_test_coverage: true,
            imports: vec![],
            tokens: vec![],
            public_interface: PublicInterface::empty(),
            prompt_snapshot: None,
            declared_traits: vec![],
            implemented_traits: vec![],
            blanket_impl_traits: vec![],
            declarations: vec![],
            static_declarations: vec![],
            module_decls: vec![],
        }
    }

    #[test]
    fn plan_builds_entry_for_v6_violation() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/foo.rs")];
        let files = vec![parsed_file_for("01_core/foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].prompt_path, "00_nucleo/prompts/foo.md");
    }

    #[test]
    fn plan_ignores_non_v6_violations() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![Violation {
            rule_id: "V1".to_string(),
            level: ViolationLevel::Error,
            message: "missing header".to_string(),
            location: Location { path: Cow::Borrowed(Path::new("foo.rs")), line: 1, column: 0 },
        }];
        let files = vec![parsed_file_for("foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        assert!(entries.is_empty());
    }

    #[test]
    fn execute_writes_when_not_dry_run() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/foo.rs")];
        let files = vec![parsed_file_for("01_core/foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        let results = execute(&entries, &rewriter, false);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(rewriter.write_calls.borrow().len(), 1);
    }

    #[test]
    fn execute_does_not_write_on_dry_run() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/foo.rs")];
        let files = vec![parsed_file_for("01_core/foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        let results = execute(&entries, &rewriter, true);
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(rewriter.write_calls.borrow().len(), 0);
    }

    #[test]
    fn format_plan_shows_nothing_when_empty() {
        let out = format_plan(&[]);
        assert!(out.contains("Nothing to update"));
    }

    #[test]
    fn format_results_shows_zero_remaining() {
        let result = SnapshotResult {
            source_path: PathBuf::from("01_core/foo.rs"),
            prompt_path: "00_nucleo/prompts/foo.md".to_string(),
            success: true,
            error: None,
        };
        let out = format_results(&[result], 0);
        assert!(out.contains("0 stale warnings remaining"));
    }

    /// Quando não existe ParsedFile correspondente a uma violação V6, a entrada
    /// deve ser incluída com unreadable_reason definido — não silenciosamente descartada.
    #[test]
    fn plan_reports_missing_parsed_file_instead_of_silencing() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/ghost.rs")];
        // Nenhum ParsedFile para o path da violação
        let entries = plan(&violations, &[], &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].unreadable_reason.is_some());
    }
}
