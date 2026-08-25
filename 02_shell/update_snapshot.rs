//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/update-snapshot.md
//! @prompt-hash fe40a13f
//! @layer L2
//! @updated 2026-03-20

use std::path::PathBuf;

use colored::Colorize;

use crate::entities::parsed_file::{ParsedFile, PublicInterface};
use crate::entities::violation::Violation;
use crate::shell::path_encoding::human_path;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotUnreadable {
    MissingParsedFile,
    MissingPromptHeader,
}

impl SnapshotUnreadable {
    fn message(&self) -> &'static str {
        match self {
            Self::MissingParsedFile => "no parsed file found for violation path",
            Self::MissingPromptHeader => "file has no @prompt header",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotEntry {
    Ready {
        source_path: PathBuf,
        prompt_path: String,
        snapshot: String,
    },
    Unreadable {
        source_path: PathBuf,
        reason: SnapshotUnreadable,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotResult {
    DryRun {
        source_path: PathBuf,
        prompt_path: String,
        snapshot: String,
    },
    Written {
        source_path: PathBuf,
        prompt_path: String,
    },
    WriteFailed {
        source_path: PathBuf,
        prompt_path: String,
        reason: String,
    },
    Unreadable {
        source_path: PathBuf,
        reason: SnapshotUnreadable,
    },
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
            let Some(parsed) = parsed_files
                .iter()
                .find(|p| p.path == v.location.path.as_ref())
            else {
                return SnapshotEntry::Unreadable {
                    source_path: v.location.path.to_path_buf(),
                    reason: SnapshotUnreadable::MissingParsedFile,
                };
            };
            let Some(header) = parsed.prompt_header.as_ref() else {
                return SnapshotEntry::Unreadable {
                    source_path: v.location.path.to_path_buf(),
                    reason: SnapshotUnreadable::MissingPromptHeader,
                };
            };
            let snapshot = rewriter.serialize_snapshot(&parsed.public_interface);
            SnapshotEntry::Ready {
                source_path: v.location.path.to_path_buf(),
                prompt_path: header.prompt_path.to_string(),
                snapshot,
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
        .map(|entry| match entry {
            SnapshotEntry::Unreadable {
                source_path,
                reason,
            } => SnapshotResult::Unreadable {
                source_path: source_path.clone(),
                reason: reason.clone(),
            },
            SnapshotEntry::Ready {
                source_path,
                prompt_path,
                snapshot,
            } if dry_run => SnapshotResult::DryRun {
                source_path: source_path.clone(),
                prompt_path: prompt_path.clone(),
                snapshot: snapshot.clone(),
            },
            SnapshotEntry::Ready {
                source_path,
                prompt_path,
                snapshot,
            } => match rewriter.write_snapshot(prompt_path, snapshot) {
                Ok(()) => SnapshotResult::Written {
                    source_path: source_path.clone(),
                    prompt_path: prompt_path.clone(),
                },
                Err(reason) => SnapshotResult::WriteFailed {
                    source_path: source_path.clone(),
                    prompt_path: prompt_path.clone(),
                    reason,
                },
            },
        })
        .collect()
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_plan(entries: &[SnapshotEntry]) -> String {
    let actionable: Vec<_> = entries
        .iter()
        .filter_map(|entry| match entry {
            SnapshotEntry::Ready {
                source_path,
                prompt_path,
                snapshot,
            } => Some((source_path, prompt_path, snapshot)),
            SnapshotEntry::Unreadable { .. } => None,
        })
        .collect();
    let unreadable: Vec<_> = entries
        .iter()
        .filter_map(|entry| match entry {
            SnapshotEntry::Unreadable {
                source_path,
                reason,
            } => Some((source_path, reason)),
            SnapshotEntry::Ready { .. } => None,
        })
        .collect();

    if entries.is_empty() {
        return format!("{}\n", "Nothing to update".green().bold());
    }

    let mut out = String::new();

    if !actionable.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Would update snapshot in".cyan().bold(),
            actionable.len(),
            if actionable.len() == 1 {
                "file"
            } else {
                "files"
            }
        ));
        for (source_path, prompt_path, snapshot) in &actionable {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                human_path(source_path),
                prompt_path
            ));
            out.push_str(snapshot);
            if !snapshot.ends_with('\n') {
                out.push('\n');
            }
        }
    }

    if !unreadable.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{} {} (no parsed record or header):\n",
            "Skipped".yellow().bold(),
            unreadable.len()
        ));
        for (source_path, reason) in unreadable {
            out.push_str(&format!(
                "  {} — {}\n",
                human_path(source_path),
                reason.message(),
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
    let written: Vec<_> = results
        .iter()
        .filter_map(|result| match result {
            SnapshotResult::Written {
                source_path,
                prompt_path,
            } => Some((source_path, prompt_path)),
            SnapshotResult::DryRun { .. }
            | SnapshotResult::WriteFailed { .. }
            | SnapshotResult::Unreadable { .. } => None,
        })
        .collect();
    let dry_runs: Vec<_> = results
        .iter()
        .filter_map(|result| match result {
            SnapshotResult::DryRun {
                source_path,
                prompt_path,
                snapshot,
            } => Some((source_path, prompt_path, snapshot)),
            SnapshotResult::Written { .. }
            | SnapshotResult::WriteFailed { .. }
            | SnapshotResult::Unreadable { .. } => None,
        })
        .collect();
    let failed: Vec<_> = results
        .iter()
        .filter(|result| {
            matches!(
                result,
                SnapshotResult::WriteFailed { .. } | SnapshotResult::Unreadable { .. }
            )
        })
        .collect();

    if !written.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Updated snapshot in".green().bold(),
            written.len(),
            if written.len() == 1 { "file" } else { "files" }
        ));
        for (source_path, prompt_path) in &written {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                human_path(source_path),
                prompt_path
            ));
        }
    }

    if !dry_runs.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!(
            "{} {} {}:\n",
            "Dry-run — would update snapshot in".cyan().bold(),
            dry_runs.len(),
            if dry_runs.len() == 1 { "file" } else { "files" }
        ));
        for (source_path, prompt_path, snapshot) in &dry_runs {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                human_path(source_path),
                prompt_path
            ));
            out.push_str(snapshot);
            if !snapshot.ends_with('\n') {
                out.push('\n');
            }
        }
    }

    if !failed.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{} {} failed:\n",
            "Error".red().bold(),
            failed.len()
        ));
        for result in &failed {
            match result {
                SnapshotResult::WriteFailed {
                    source_path,
                    reason,
                    ..
                } => out.push_str(&format!("  {} — {}\n", human_path(source_path), reason,)),
                SnapshotResult::Unreadable {
                    source_path,
                    reason,
                } => out.push_str(&format!(
                    "  {} — {}\n",
                    human_path(source_path),
                    reason.message(),
                )),
                SnapshotResult::DryRun { .. } | SnapshotResult::Written { .. } => unreachable!(),
            }
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
            Self {
                write_calls: RefCell::new(vec![]),
                write_result,
            }
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
            location: Location {
                path: Cow::Borrowed(Path::new(path)),
                line: 1,
                column: 0,
            },
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
            prompt_refs: vec![],
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
            decision_exprs: vec![],
            constants: vec![],
            semantic_observations: vec![],
        }
    }

    #[test]
    fn plan_builds_entry_for_v6_violation() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/foo.rs")];
        let files = vec![parsed_file_for("01_core/foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        assert_eq!(
            entries,
            vec![SnapshotEntry::Ready {
                source_path: PathBuf::from("01_core/foo.rs"),
                prompt_path: "00_nucleo/prompts/foo.md".to_string(),
                snapshot: "## Interface Snapshot\n<!-- crystalline-snapshot: {} -->".to_string(),
            }]
        );
    }

    #[test]
    fn plan_ignores_non_v6_violations() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![Violation {
            rule_id: "V1".to_string(),
            level: ViolationLevel::Error,
            message: "missing header".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new("foo.rs")),
                line: 1,
                column: 0,
            },
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
        assert_eq!(
            results,
            vec![SnapshotResult::Written {
                source_path: PathBuf::from("01_core/foo.rs"),
                prompt_path: "00_nucleo/prompts/foo.md".to_string(),
            }]
        );
        assert_eq!(rewriter.write_calls.borrow().len(), 1);
    }

    #[test]
    fn execute_does_not_write_on_dry_run() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/foo.rs")];
        let files = vec![parsed_file_for("01_core/foo.rs")];
        let entries = plan(&violations, &files, &rewriter);
        let results = execute(&entries, &rewriter, true);
        assert!(matches!(
            results.as_slice(),
            [SnapshotResult::DryRun { .. }]
        ));
        assert_eq!(rewriter.write_calls.borrow().len(), 0);
    }

    #[test]
    fn format_plan_shows_nothing_when_empty() {
        let out = format_plan(&[]);
        assert!(out.contains("Nothing to update"));
    }

    #[test]
    fn format_results_shows_zero_remaining() {
        let result = SnapshotResult::Written {
            source_path: PathBuf::from("01_core/foo.rs"),
            prompt_path: "00_nucleo/prompts/foo.md".to_string(),
        };
        let out = format_results(&[result], 0);
        assert!(out.contains("0 stale warnings remaining"));
    }

    /// Quando não existe ParsedFile correspondente a uma violação V6, a entrada
    /// deve ser incluída como Unreadable — não silenciosamente descartada.
    #[test]
    fn plan_reports_missing_parsed_file_instead_of_silencing() {
        let rewriter = MockRewriter::new(Ok(()));
        let violations = vec![v6_violation("01_core/ghost.rs")];
        // Nenhum ParsedFile para o path da violação
        let entries = plan(&violations, &[], &rewriter);
        assert_eq!(
            entries,
            vec![SnapshotEntry::Unreadable {
                source_path: PathBuf::from("01_core/ghost.rs"),
                reason: SnapshotUnreadable::MissingParsedFile,
            }]
        );
    }
}
