//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/fix-hashes.md
//! @prompt-hash 929d9c47
//! @layer L2
//! @updated 2026-03-20

use std::path::{Path, PathBuf};

use colored::Colorize;

use crate::entities::violation::Violation;

// ── Outbound port (implemented by L4 adapter wrapping L3) ────────────────────

/// L2-defined contract for reading and writing hashes in source files.
/// L3 provides the concrete I/O implementation.
/// L4 creates the adapter — L2 never imports L3 directly.
pub trait HashRewriter {
    /// Read the `@prompt` path and current `@prompt-hash` from a source file header.
    /// Returns None if the file cannot be read or has no header.
    fn read_header(&self, source_path: &Path) -> Option<(String, String)>;

    /// Compute SHA256[0..8] of the prompt file at the given path.
    /// Returns None if the prompt file does not exist.
    fn compute_hash(&self, prompt_path: &str) -> Option<String>;

    /// Atomically replace `@prompt-hash <old>` with `@prompt-hash <new>` in source file.
    fn write_hash(&self, source_path: &Path, new_hash: &str) -> Result<(), String>;
}

// ── Data types ────────────────────────────────────────────────────────────────

pub struct FixEntry {
    pub source_path: PathBuf,
    /// Hash currently written in the file header. Empty when header is unreadable.
    pub old_hash: String,
    /// Real hash of the L0 prompt file. None if prompt file is missing.
    pub new_hash: Option<String>,
    /// Set when the file header could not be read. Entry is skipped in execute().
    pub unreadable_reason: Option<String>,
}

pub struct FixResult {
    pub source_path: PathBuf,
    pub old_hash: String,
    pub new_hash: String,
    pub success: bool,
    pub error: Option<String>,
}

// ── Core functions ────────────────────────────────────────────────────────────

/// Build fix entries from V5 violations.
/// Each entry captures the old hash and the real hash (if the prompt exists).
/// Entries where the file header cannot be read are included with `unreadable_reason` set,
/// rather than silently discarded.
pub fn plan(violations: &[Violation<'_>], rewriter: &dyn HashRewriter) -> Vec<FixEntry> {
    violations
        .iter()
        .filter(|v| v.rule_id == "V5")
        .map(|v| match rewriter.read_header(&v.location.path) {
            Some((prompt_path, old_hash)) => {
                let new_hash = rewriter.compute_hash(&prompt_path);
                FixEntry {
                    source_path: v.location.path.to_path_buf(),
                    old_hash,
                    new_hash,
                    unreadable_reason: None,
                }
            }
            None => FixEntry {
                source_path: v.location.path.to_path_buf(),
                old_hash: String::new(),
                new_hash: None,
                unreadable_reason: Some("could not read file header".to_string()),
            },
        })
        .collect()
}

/// Execute or dry-run based on entries.
/// Skips entries where `new_hash` is None (prompt file missing).
pub fn execute(
    entries: &[FixEntry],
    rewriter: &dyn HashRewriter,
    dry_run: bool,
) -> Vec<FixResult> {
    entries
        .iter()
        .filter_map(|entry| {
            // Entradas com header ilegível ou prompt ausente não podem ser corrigidas
            if entry.unreadable_reason.is_some() {
                return None;
            }
            let new_hash = entry.new_hash.as_ref()?.clone();

            if dry_run {
                return Some(FixResult {
                    source_path: entry.source_path.clone(),
                    old_hash: entry.old_hash.clone(),
                    new_hash,
                    success: true,
                    error: None,
                });
            }

            let outcome = rewriter.write_hash(&entry.source_path, &new_hash);
            Some(FixResult {
                source_path: entry.source_path.clone(),
                old_hash: entry.old_hash.clone(),
                new_hash,
                success: outcome.is_ok(),
                error: outcome.err(),
            })
        })
        .collect()
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_plan(entries: &[FixEntry]) -> String {
    let fixable: Vec<_> =
        entries.iter().filter(|e| e.new_hash.is_some() && e.unreadable_reason.is_none()).collect();
    let unfixable: Vec<_> = entries
        .iter()
        .filter(|e| e.new_hash.is_none() && e.unreadable_reason.is_none())
        .collect();
    let unreadable: Vec<_> =
        entries.iter().filter(|e| e.unreadable_reason.is_some()).collect();

    if entries.is_empty() {
        return format!("{}\n", "Nothing to fix".green().bold());
    }

    let mut out = String::new();

    if !fixable.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Would fix".cyan().bold(),
            fixable.len(),
            if fixable.len() == 1 { "file" } else { "files" }
        ));
        for entry in &fixable {
            out.push_str(&format!(
                "  {:<45} {} → {}\n",
                entry.source_path.display(),
                entry.old_hash.red(),
                entry.new_hash.as_deref().unwrap_or("?").green(),
            ));
        }
    }

    if !unfixable.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{} {} (prompt file missing):\n",
            "Cannot fix".yellow().bold(),
            unfixable.len()
        ));
        for entry in unfixable {
            out.push_str(&format!("  {}\n", entry.source_path.display()));
        }
    }

    if !unreadable.is_empty() {
        out.push('\n');
        out.push_str(&format!(
            "{} {} (header unreadable):\n",
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

pub fn format_results(results: &[FixResult], unfixable: usize, remaining_v5: usize) -> String {
    if results.is_empty() && unfixable == 0 {
        return format!("{}\n", "Nothing to fix".green().bold());
    }

    let mut out = String::new();
    let succeeded: Vec<_> = results.iter().filter(|r| r.success).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.success).collect();

    if !succeeded.is_empty() {
        out.push_str(&format!(
            "{} {} {}:\n",
            "Fixed".green().bold(),
            succeeded.len(),
            if succeeded.len() == 1 { "file" } else { "files" }
        ));
        for r in &succeeded {
            out.push_str(&format!(
                "  {:<45} → {}\n",
                r.source_path.display(),
                r.new_hash.green(),
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

    if unfixable > 0 {
        out.push('\n');
        out.push_str(&format!(
            "{} ({} file(s) reference missing prompt)\n",
            "Skipped".yellow().bold(),
            unfixable,
        ));
    }

    out.push('\n');
    if remaining_v5 == 0 {
        out.push_str(&format!(
            "Re-running analysis... {} 0 drift warnings remaining\n",
            "✅".green()
        ));
    } else {
        out.push_str(&format!(
            "Re-running analysis... {} {} drift warning(s) remaining\n",
            "⚠".yellow(),
            remaining_v5,
        ));
    }

    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::violation::{Location, ViolationLevel};
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    // ── Mock ──────────────────────────────────────────────────────────────────

    struct MockRewriter {
        header: Option<(String, String)>,
        hash: Option<String>,
        write_calls: RefCell<Vec<(PathBuf, String)>>,
        write_result: Result<(), String>,
    }

    impl MockRewriter {
        fn new(
            header: Option<(&str, &str)>,
            hash: Option<&str>,
            write_result: Result<(), String>,
        ) -> Self {
            Self {
                header: header.map(|(p, h)| (p.to_string(), h.to_string())),
                hash: hash.map(str::to_string),
                write_calls: RefCell::new(vec![]),
                write_result,
            }
        }
    }

    impl HashRewriter for MockRewriter {
        fn read_header(&self, _: &Path) -> Option<(String, String)> {
            self.header.clone()
        }
        fn compute_hash(&self, _: &str) -> Option<String> {
            self.hash.clone()
        }
        fn write_hash(&self, path: &Path, new_hash: &str) -> Result<(), String> {
            self.write_calls.borrow_mut().push((path.to_path_buf(), new_hash.to_string()));
            self.write_result.clone()
        }
    }

    fn v5_violation(path: &'static str) -> Violation<'static> {
        Violation {
            rule_id: "V5".to_string(),
            level: ViolationLevel::Warning,
            message: "drift".to_string(),
            location: Location { path: Cow::Borrowed(Path::new(path)), line: 1, column: 0 },
        }
    }

    // ── plan() ────────────────────────────────────────────────────────────────

    #[test]
    fn plan_builds_entry_for_v5_violation() {
        let rewriter = MockRewriter::new(
            Some(("00_nucleo/prompts/foo.md", "00000000")),
            Some("a3f8c2d1"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].old_hash, "00000000");
        assert_eq!(entries[0].new_hash, Some("a3f8c2d1".to_string()));
    }

    #[test]
    fn plan_ignores_non_v5_violations() {
        let rewriter = MockRewriter::new(Some(("p.md", "00000000")), Some("a1b2c3d4"), Ok(()));
        let violations = vec![
            Violation {
                rule_id: "V1".to_string(),
                level: ViolationLevel::Error,
                message: "header missing".to_string(),
                location: Location { path: Cow::Borrowed(Path::new("foo.rs")), line: 1, column: 0 },
            },
        ];
        let entries = plan(&violations, &rewriter);
        assert!(entries.is_empty());
    }

    #[test]
    fn plan_marks_unfixable_when_prompt_missing() {
        let rewriter = MockRewriter::new(
            Some(("00_nucleo/prompts/missing.md", "00000000")),
            None, // prompt doesn't exist
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].new_hash.is_none());
    }

    // ── execute() ────────────────────────────────────────────────────────────

    #[test]
    fn execute_writes_when_not_dry_run() {
        let rewriter = MockRewriter::new(
            Some(("p.md", "00000000")),
            Some("a3f8c2d1"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, false);

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(rewriter.write_calls.borrow().len(), 1);
    }

    #[test]
    fn execute_does_not_write_on_dry_run() {
        let rewriter = MockRewriter::new(
            Some(("p.md", "00000000")),
            Some("a3f8c2d1"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, true);

        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(rewriter.write_calls.borrow().len(), 0); // no writes
    }

    #[test]
    fn execute_skips_unfixable_entries() {
        let rewriter = MockRewriter::new(Some(("p.md", "00000000")), None, Ok(()));
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, false);
        assert!(results.is_empty()); // unfixable = skipped
    }

    #[test]
    fn execute_records_write_error() {
        let rewriter = MockRewriter::new(
            Some(("p.md", "00000000")),
            Some("a3f8c2d1"),
            Err("permission denied".to_string()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, false);

        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert!(results[0].error.as_deref().unwrap().contains("permission"));
    }

    // ── format ────────────────────────────────────────────────────────────────

    #[test]
    fn format_plan_shows_nothing_to_fix_when_empty() {
        let out = format_plan(&[]);
        assert!(out.contains("Nothing to fix"));
    }

    #[test]
    fn format_results_shows_zero_remaining() {
        let result = FixResult {
            source_path: PathBuf::from("01_core/foo.rs"),
            old_hash: "00000000".to_string(),
            new_hash: "a3f8c2d1".to_string(),
            success: true,
            error: None,
        };
        let out = format_results(&[result], 0, 0);
        assert!(out.contains("0 drift warnings remaining"));
    }

    /// Quando read_header devolve None, a entrada deve ser incluída com
    /// unreadable_reason definido — não silenciosamente descartada.
    #[test]
    fn plan_reports_unreadable_header_instead_of_silencing() {
        let rewriter = MockRewriter::new(None, None, Ok(()));
        let violations = vec![v5_violation("01_core/unreadable.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].unreadable_reason.is_some());
    }
}
