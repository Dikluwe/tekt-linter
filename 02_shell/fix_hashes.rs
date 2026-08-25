//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/fix-hashes.md
//! @prompt-hash 809049ad
//! @layer L2
//! @updated 2026-03-20

use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, fmt};

use colored::Colorize;

use crate::entities::violation::Violation;
use crate::shell::path_encoding::human_path;

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

    /// Compute SHA256[0..8] of the source file, ignoring its own @prompt-hash line.
    fn compute_source_hash(&self, source_path: &Path) -> Option<String>;

    /// Atomically replace `@prompt-hash <old>` with `@prompt-hash <new>` in source file.
    fn write_hash(&self, source_path: &Path, new_hash: &str) -> Result<(), String>;

    /// Inject "Hash do Código: <hash>" into the prompt file.
    fn write_prompt_meta(&self, prompt_path: &str, code_hash: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairSnapshot {
    pub source_bytes: Vec<u8>,
    pub prompt_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BijectivePair {
    pub source_path: PathBuf,
    pub prompt_path: String,
    pub old_prompt_hash: String,
    pub new_prompt_hash: String,
    pub new_source_hash: String,
    pub new_source_bytes: Vec<u8>,
    pub new_prompt_bytes: Vec<u8>,
}

pub trait TransactionalHashRewriter {
    fn preflight(&self, pair: &BijectivePair) -> Result<PairSnapshot, String>;
    fn apply_pair(&self, pair: &BijectivePair) -> Result<(), String>;
    fn rollback_pair(&self, pair: &BijectivePair, snapshot: &PairSnapshot) -> Result<(), String>;
    fn validate_pair(&self, pair: &BijectivePair) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixBatchPlan {
    pub pairs: Vec<BijectivePair>,
    snapshots: Vec<PairSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixBatchError {
    OwnershipCollisions {
        collisions: Vec<(String, Vec<PathBuf>)>,
    },
    Preflight {
        source_path: PathBuf,
        reason: String,
    },
    Validation {
        source_path: PathBuf,
        reason: String,
    },
}

impl fmt::Display for FixBatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnershipCollisions { collisions } => {
                write!(formatter, "ownership collisions:")?;
                for (prompt, paths) in collisions {
                    write!(formatter, " {prompt}=[")?;
                    for (index, path) in paths.iter().enumerate() {
                        if index > 0 {
                            write!(formatter, ", ")?;
                        }
                        write!(formatter, "{}", path.display())?;
                    }
                    write!(formatter, "]")?;
                }
                Ok(())
            }
            Self::Preflight {
                source_path,
                reason,
            } => write!(formatter, "preflight {}: {reason}", source_path.display()),
            Self::Validation {
                source_path,
                reason,
            } => write!(formatter, "validation {}: {reason}", source_path.display()),
        }
    }
}

impl std::error::Error for FixBatchError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyFailure {
    RollbackFailed {
        source_path: PathBuf,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixBatchResult {
    DryRun { count: usize },
    Applied { count: usize },
    RolledBack { reason: String },
    Fatal(ApplyFailure),
}

impl FixBatchResult {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::DryRun { .. } | Self::Applied { .. })
    }
}

pub fn plan_bijective(
    pairs: &[BijectivePair],
    rewriter: &dyn TransactionalHashRewriter,
) -> Result<FixBatchPlan, FixBatchError> {
    let mut groups: BTreeMap<&str, Vec<PathBuf>> = BTreeMap::new();
    for pair in pairs {
        groups
            .entry(&pair.prompt_path)
            .or_default()
            .push(pair.source_path.clone());
    }
    let collisions = groups
        .into_iter()
        .filter_map(|(prompt, mut paths)| {
            paths.sort();
            paths.dedup();
            (paths.len() > 1).then(|| (prompt.to_owned(), paths))
        })
        .collect::<Vec<_>>();
    if !collisions.is_empty() {
        return Err(FixBatchError::OwnershipCollisions { collisions });
    }

    let mut ordered = pairs.to_vec();
    ordered.sort_by(|left, right| left.source_path.cmp(&right.source_path));
    ordered.dedup_by(|left, right| {
        left.source_path == right.source_path && left.prompt_path == right.prompt_path
    });
    let mut snapshots = Vec::with_capacity(ordered.len());
    for pair in &ordered {
        snapshots.push(
            rewriter
                .preflight(pair)
                .map_err(|reason| FixBatchError::Preflight {
                    source_path: pair.source_path.clone(),
                    reason,
                })?,
        );
    }
    Ok(FixBatchPlan {
        pairs: ordered,
        snapshots,
    })
}

pub fn execute_bijective(
    plan: &FixBatchPlan,
    rewriter: &dyn TransactionalHashRewriter,
    dry_run: bool,
) -> FixBatchResult {
    if dry_run {
        return FixBatchResult::DryRun {
            count: plan.pairs.len(),
        };
    }
    for (index, pair) in plan.pairs.iter().enumerate() {
        if let Err(reason) = rewriter.apply_pair(pair) {
            for rollback_index in (0..index).rev() {
                let rollback_pair = &plan.pairs[rollback_index];
                if let Err(rollback_reason) =
                    rewriter.rollback_pair(rollback_pair, &plan.snapshots[rollback_index])
                {
                    return FixBatchResult::Fatal(ApplyFailure::RollbackFailed {
                        source_path: rollback_pair.source_path.clone(),
                        reason: rollback_reason,
                    });
                }
            }
            return FixBatchResult::RolledBack { reason };
        }
    }
    FixBatchResult::Applied {
        count: plan.pairs.len(),
    }
}

pub fn validate_bijective(
    plan: &FixBatchPlan,
    rewriter: &dyn TransactionalHashRewriter,
) -> Result<(), FixBatchError> {
    for pair in &plan.pairs {
        rewriter
            .validate_pair(pair)
            .map_err(|reason| FixBatchError::Validation {
                source_path: pair.source_path.clone(),
                reason,
            })?;
    }
    Ok(())
}

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixUnavailable {
    HeaderUnreadable,
    PromptHashUnavailable {
        prompt_path: String,
        old_hash: String,
        source_hash: String,
    },
    SourceHashUnavailable {
        prompt_path: String,
        old_hash: String,
        new_hash: String,
    },
    BothHashesUnavailable {
        prompt_path: String,
        old_hash: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixEntry {
    Ready {
        source_path: PathBuf,
        prompt_path: String,
        old_hash: String,
        new_hash: String,
        source_hash: String,
    },
    Unavailable {
        source_path: PathBuf,
        reason: FixUnavailable,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixResult {
    Unavailable {
        source_path: PathBuf,
        reason: FixUnavailable,
    },
    DryRun {
        source_path: PathBuf,
        prompt_path: String,
        old_hash: String,
        new_hash: String,
        source_hash: String,
    },
    Applied {
        source_path: PathBuf,
        prompt_path: String,
        new_hash: String,
        source_hash: String,
    },
    CodeWriteFailed {
        source_path: PathBuf,
        prompt_path: String,
        new_hash: String,
        source_hash: String,
        reason: String,
    },
    PartialWrite {
        source_path: PathBuf,
        prompt_path: String,
        applied_new_hash: String,
        rejected_source_hash: String,
        reason: String,
    },
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
                let source_hash = rewriter.compute_source_hash(&v.location.path);
                match (new_hash, source_hash) {
                    (Some(new_hash), Some(source_hash)) => FixEntry::Ready {
                        source_path: v.location.path.to_path_buf(),
                        prompt_path,
                        old_hash,
                        new_hash,
                        source_hash,
                    },
                    (None, Some(source_hash)) => FixEntry::Unavailable {
                        source_path: v.location.path.to_path_buf(),
                        reason: FixUnavailable::PromptHashUnavailable {
                            prompt_path,
                            old_hash,
                            source_hash,
                        },
                    },
                    (Some(new_hash), None) => FixEntry::Unavailable {
                        source_path: v.location.path.to_path_buf(),
                        reason: FixUnavailable::SourceHashUnavailable {
                            prompt_path,
                            old_hash,
                            new_hash,
                        },
                    },
                    (None, None) => FixEntry::Unavailable {
                        source_path: v.location.path.to_path_buf(),
                        reason: FixUnavailable::BothHashesUnavailable {
                            prompt_path,
                            old_hash,
                        },
                    },
                }
            }
            None => FixEntry::Unavailable {
                source_path: v.location.path.to_path_buf(),
                reason: FixUnavailable::HeaderUnreadable,
            },
        })
        .collect()
}

/// Execute or dry-run based on entries.
/// Skips entries where `new_hash` is None (prompt file missing).
pub fn execute(entries: &[FixEntry], rewriter: &dyn HashRewriter, dry_run: bool) -> Vec<FixResult> {
    entries
        .iter()
        .map(|entry| match entry {
            FixEntry::Unavailable {
                source_path,
                reason,
            } => FixResult::Unavailable {
                source_path: source_path.clone(),
                reason: reason.clone(),
            },
            FixEntry::Ready {
                source_path,
                prompt_path,
                old_hash,
                new_hash,
                source_hash,
            } if dry_run => FixResult::DryRun {
                source_path: source_path.clone(),
                prompt_path: prompt_path.clone(),
                old_hash: old_hash.clone(),
                new_hash: new_hash.clone(),
                source_hash: source_hash.clone(),
            },
            FixEntry::Ready {
                source_path,
                prompt_path,
                new_hash,
                source_hash,
                ..
            } => {
                if let Err(reason) = rewriter.write_hash(source_path, new_hash) {
                    return FixResult::CodeWriteFailed {
                        source_path: source_path.clone(),
                        prompt_path: prompt_path.clone(),
                        new_hash: new_hash.clone(),
                        source_hash: source_hash.clone(),
                        reason,
                    };
                }
                match rewriter.write_prompt_meta(prompt_path, source_hash) {
                    Ok(()) => FixResult::Applied {
                        source_path: source_path.clone(),
                        prompt_path: prompt_path.clone(),
                        new_hash: new_hash.clone(),
                        source_hash: source_hash.clone(),
                    },
                    Err(reason) => FixResult::PartialWrite {
                        source_path: source_path.clone(),
                        prompt_path: prompt_path.clone(),
                        applied_new_hash: new_hash.clone(),
                        rejected_source_hash: source_hash.clone(),
                        reason,
                    },
                }
            }
        })
        .collect()
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_plan(entries: &[FixEntry]) -> String {
    if entries.is_empty() {
        return format!("{}\n", "Nothing to fix".green().bold());
    }
    let mut out = String::new();
    for entry in entries {
        match entry {
            FixEntry::Ready {
                source_path,
                prompt_path,
                old_hash,
                new_hash,
                source_hash,
            } => out.push_str(&format!(
                "Would fix {} prompt={} old={} hash-a={} hash-b={}\n",
                human_path(source_path),
                prompt_path,
                old_hash,
                new_hash,
                source_hash
            )),
            FixEntry::Unavailable {
                source_path,
                reason,
            } => out.push_str(&format!(
                "Cannot fix {} — {}\n",
                human_path(source_path),
                describe_unavailable(reason)
            )),
        }
    }
    out
}

fn describe_unavailable(reason: &FixUnavailable) -> String {
    match reason {
        FixUnavailable::HeaderUnreadable => "header unreadable".into(),
        FixUnavailable::PromptHashUnavailable {
            prompt_path,
            old_hash,
            source_hash,
        } => format!(
            "prompt hash unavailable: prompt={prompt_path} old={old_hash} hash-b={source_hash}"
        ),
        FixUnavailable::SourceHashUnavailable {
            prompt_path,
            old_hash,
            new_hash,
        } => format!(
            "source hash unavailable: prompt={prompt_path} old={old_hash} hash-a={new_hash}"
        ),
        FixUnavailable::BothHashesUnavailable {
            prompt_path,
            old_hash,
        } => format!("both hashes unavailable: prompt={prompt_path} old={old_hash}"),
    }
}

pub fn format_results(results: &[FixResult], unfixable: usize, remaining_v5: usize) -> String {
    if results.is_empty() && unfixable == 0 {
        return format!("{}\n", "Nothing to fix".green().bold());
    }

    let mut out = String::new();
    for result in results {
        match result {
            FixResult::Unavailable { source_path, reason } => out.push_str(&format!("Unavailable {} — {}\n", human_path(source_path), describe_unavailable(reason))),
            FixResult::DryRun { source_path, prompt_path, old_hash, new_hash, source_hash } => out.push_str(&format!("Dry-run {} prompt={} old={} hash-a={} hash-b={}\n", human_path(source_path), prompt_path, old_hash, new_hash, source_hash)),
            FixResult::Applied { source_path, prompt_path, new_hash, source_hash } => out.push_str(&format!("Applied {} prompt={} hash-a={} hash-b={}\n", human_path(source_path), prompt_path, new_hash, source_hash)),
            FixResult::CodeWriteFailed { source_path, prompt_path, new_hash, source_hash, reason } => out.push_str(&format!("Code write failed {} prompt={} hash-a={} hash-b={} reason={}\n", human_path(source_path), prompt_path, new_hash, source_hash, reason)),
            FixResult::PartialWrite { source_path, prompt_path, applied_new_hash, rejected_source_hash, reason } => out.push_str(&format!("Partial write: prompt metadata failed {} prompt={} applied-hash-a={} rejected-hash-b={} reason={}\n", human_path(source_path), prompt_path, applied_new_hash, rejected_source_hash, reason)),
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
        prompt_hash: Option<String>,
        source_hash: Option<String>,
        write_calls: RefCell<Vec<(PathBuf, String)>>,
        meta_calls: RefCell<Vec<(String, String)>>,
        write_result: Result<(), String>,
    }

    impl MockRewriter {
        fn new(
            header: Option<(&str, &str)>,
            prompt_hash: Option<&str>,
            source_hash: Option<&str>,
            write_result: Result<(), String>,
        ) -> Self {
            Self {
                header: header.map(|(p, h)| (p.to_string(), h.to_string())),
                prompt_hash: prompt_hash.map(str::to_string),
                source_hash: source_hash.map(str::to_string),
                write_calls: RefCell::new(vec![]),
                meta_calls: RefCell::new(vec![]),
                write_result,
            }
        }
    }

    impl HashRewriter for MockRewriter {
        fn read_header(&self, _: &Path) -> Option<(String, String)> {
            self.header.clone()
        }
        fn compute_hash(&self, _: &str) -> Option<String> {
            self.prompt_hash.clone()
        }
        fn compute_source_hash(&self, _: &Path) -> Option<String> {
            self.source_hash.clone()
        }
        fn write_hash(&self, path: &Path, new_hash: &str) -> Result<(), String> {
            self.write_calls
                .borrow_mut()
                .push((path.to_path_buf(), new_hash.to_string()));
            self.write_result.clone()
        }
        fn write_prompt_meta(&self, prompt_path: &str, code_hash: &str) -> Result<(), String> {
            self.meta_calls
                .borrow_mut()
                .push((prompt_path.to_string(), code_hash.to_string()));
            Ok(())
        }
    }

    fn v5_violation(path: &'static str) -> Violation<'static> {
        Violation {
            rule_id: "V5".to_string(),
            level: ViolationLevel::Warning,
            message: "drift".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new(path)),
                line: 1,
                column: 0,
            },
        }
    }

    // ── plan() ────────────────────────────────────────────────────────────────

    #[test]
    fn plan_builds_entry_for_v5_violation() {
        let rewriter = MockRewriter::new(
            Some(("00_nucleo/prompts/foo.md", "00000000")),
            Some("a3f8c2d1"),
            Some("b9e4f7a2"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(
            matches!(&entries[0], FixEntry::Ready { old_hash, new_hash, source_hash, .. }
            if old_hash == "00000000" && new_hash == "a3f8c2d1" && source_hash == "b9e4f7a2")
        );
    }

    #[test]
    fn plan_ignores_non_v5_violations() {
        let rewriter =
            MockRewriter::new(Some(("p.md", "00000000")), Some("a1b2c3d4"), None, Ok(()));
        let violations = vec![Violation {
            rule_id: "V1".to_string(),
            level: ViolationLevel::Error,
            message: "header missing".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new("foo.rs")),
                line: 1,
                column: 0,
            },
        }];
        let entries = plan(&violations, &rewriter);
        assert!(entries.is_empty());
    }

    #[test]
    fn plan_marks_unfixable_when_prompt_missing() {
        let rewriter = MockRewriter::new(
            Some(("00_nucleo/prompts/missing.md", "00000000")),
            None, // prompt doesn't exist
            None,
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0], FixEntry::Unavailable { .. }));
    }

    // ── execute() ────────────────────────────────────────────────────────────

    #[test]
    fn execute_writes_back_code_hash_to_prompt() {
        let rewriter = MockRewriter::new(
            Some(("00_nucleo/prompts/foo.md", "00000000")),
            Some("a3f8c2d1"),
            Some("b9e4f7a2"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let _ = execute(&entries, &rewriter, false);

        assert_eq!(rewriter.write_calls.borrow().len(), 1);
        assert_eq!(rewriter.meta_calls.borrow().len(), 1);
        assert_eq!(rewriter.meta_calls.borrow()[0].1, "b9e4f7a2");
    }

    #[test]
    fn execute_does_not_write_on_dry_run() {
        let rewriter = MockRewriter::new(
            Some(("p.md", "00000000")),
            Some("a3f8c2d1"),
            Some("b9e4f7a2"),
            Ok(()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, true);

        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], FixResult::DryRun { .. }));
        assert_eq!(rewriter.write_calls.borrow().len(), 0); // no writes
        assert_eq!(rewriter.meta_calls.borrow().len(), 0);
    }

    #[test]
    fn execute_skips_unfixable_entries() {
        let rewriter = MockRewriter::new(Some(("p.md", "00000000")), None, None, Ok(()));
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, false);
        assert!(matches!(results[0], FixResult::Unavailable { .. }));
    }

    #[test]
    fn execute_records_write_error() {
        let rewriter = MockRewriter::new(
            Some(("p.md", "00000000")),
            Some("a3f8c2d1"),
            Some("b9e4f7a2"),
            Err("permission denied".to_string()),
        );
        let violations = vec![v5_violation("01_core/foo.rs")];
        let entries = plan(&violations, &rewriter);
        let results = execute(&entries, &rewriter, false);

        assert_eq!(results.len(), 1);
        assert!(
            matches!(&results[0], FixResult::CodeWriteFailed { reason, .. } if reason.contains("permission"))
        );
    }

    // ── format ────────────────────────────────────────────────────────────────

    #[test]
    fn format_plan_shows_nothing_to_fix_when_empty() {
        let out = format_plan(&[]);
        assert!(out.contains("Nothing to fix"));
    }

    #[test]
    fn format_results_shows_zero_remaining() {
        let result = FixResult::Applied {
            source_path: PathBuf::from("01_core/foo.rs"),
            prompt_path: "00_nucleo/prompts/foo.md".to_string(),
            new_hash: "a3f8c2d1".to_string(),
            source_hash: "b9e4f7a2".to_string(),
        };
        let out = format_results(&[result], 0, 0);
        assert!(out.contains("0 drift warnings remaining"));
    }

    /// Quando read_header devolve None, a entrada deve ser incluída com
    /// unreadable_reason definido — não silenciosamente descartada.
    #[test]
    fn plan_reports_unreadable_header_instead_of_silencing() {
        let rewriter = MockRewriter::new(None, None, None, Ok(()));
        let violations = vec![v5_violation("01_core/unreadable.rs")];
        let entries = plan(&violations, &rewriter);
        assert_eq!(entries.len(), 1);
        assert!(matches!(
            entries[0],
            FixEntry::Unavailable {
                reason: FixUnavailable::HeaderUnreadable,
                ..
            }
        ));
    }
}
