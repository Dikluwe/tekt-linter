//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 68d61185
//! @layer L4
//! @updated 2026-03-16

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser as ClapParser;

use crystalline_lint::contracts::file_provider::{FileProvider, SourceError, SourceFile};
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::contracts::prompt_provider::PromptProvider;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::parsed_file::{ParsedFile, PublicInterface};
use crystalline_lint::entities::project_index::{LocalIndex, ProjectIndex};
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::hash_writer;
use crystalline_lint::infra::prompt_reader::FsPromptReader;
use crystalline_lint::infra::prompt_snapshot_reader::FsPromptSnapshotReader;
use crystalline_lint::infra::prompt_walker::FsPromptWalker;
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::infra::snapshot_writer;
use crystalline_lint::infra::walker::FileWalker;
use crystalline_lint::rules::{
    alien_file, forbidden_import, impure_core, orphan_prompt, prompt_drift, prompt_header,
    prompt_stale, pub_leak, test_file,
};
use crystalline_lint::rules::pub_leak::L1Ports;
use crystalline_lint::shell::cli::{validate_args, Cli, EnabledChecks, OutputFormat};
use crystalline_lint::shell::fix_hashes::{self, HashRewriter};
use crystalline_lint::shell::update_snapshot::{self, SnapshotRewriter};

fn main() {
    let cli = Cli::parse();

    // ── Arg validation ────────────────────────────────────────────────────────
    if let Err(e) = validate_args(&cli) {
        eprintln!("crystalline-lint: {e}");
        process::exit(1);
    }

    // ── Config ────────────────────────────────────────────────────────────────
    let config = if cli.config.exists() {
        CrystallineConfig::load(&cli.config).unwrap_or_else(|e| {
            eprintln!("crystalline-lint: config error: {e}");
            process::exit(1);
        })
    } else {
        CrystallineConfig::default()
    };

    // ── Phase 0: scan all prompts (sequential, before pipeline) ───────────────
    let nucleo_root = PathBuf::from(".");
    let enabled = EnabledChecks::from_cli(&cli.checks, cli.no_drift, cli.no_stale);

    let orphan_exceptions: std::collections::HashSet<String> =
        config.orphan_exceptions.keys().cloned().collect();
    let prompt_walker = FsPromptWalker::new(cli.path.clone(), orphan_exceptions);
    // Scan prompts only when V7 is enabled — avoids I/O when check is suppressed.
    let all_prompts = if enabled.v7 {
        match prompt_walker.scan() {
            Ok(ap) => Some(ap),
            Err(e) => {
                eprintln!("crystalline-lint: prompt scan error: {e:?}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    // ── L1Ports from config ───────────────────────────────────────────────────
    let l1_ports = L1Ports::new(config.l1_ports.keys().cloned().collect());

    // ── Instantiate L3 components ─────────────────────────────────────────────
    let parser = RustParser::new(
        FsPromptReader { nucleo_root: nucleo_root.clone() },
        FsPromptSnapshotReader { nucleo_root: nucleo_root.clone() },
        config.clone(),
    );
    let walker = FileWalker::new(cli.path.clone(), config.clone());

    // Collect source files and errors separately so ParsedFile<'a> can borrow
    // from source_files (zero-copy, ADR-0004). all_prompts is declared above
    // source_files so it outlives them — required for V7's check_orphans.
    let (source_files, source_errors) = collect_walker_results(walker.files());

    let (mut all_violations, all_parsed, project_index) =
        run_pipeline(&source_files, &source_errors, &parser, &enabled, &l1_ports);

    // ── V7/V8 post-reduce ─────────────────────────────────────────────────────
    if enabled.v7 {
        if let Some(ref ap) = all_prompts {
            all_violations.extend(orphan_prompt::check_orphans(&project_index, ap));
        }
    }
    if enabled.v8 {
        all_violations.extend(alien_file::check_aliens(&project_index));
    }

    // ── --fix-hashes branch ───────────────────────────────────────────────────
    if cli.fix_hashes {
        let rewriter = L3HashRewriter { nucleo_root: nucleo_root.clone() };
        let entries = fix_hashes::plan(&all_violations, &rewriter);

        if cli.dry_run {
            if !cli.quiet {
                print!("{}", fix_hashes::format_plan(&entries));
            }
            return;
        }

        let unfixable = entries.iter().filter(|e| e.new_hash.is_none()).count();
        let results = fix_hashes::execute(&entries, &rewriter, false);

        // Re-run analysis to count remaining V5
        let remaining_v5 = {
            let reparser = RustParser::new(
                FsPromptReader { nucleo_root: nucleo_root.clone() },
                FsPromptSnapshotReader { nucleo_root: nucleo_root.clone() },
                config.clone(),
            );
            let rewalker = FileWalker::new(cli.path.clone(), config.clone());
            let (re_files, re_errors) = collect_walker_results(rewalker.files());
            let v5_only = EnabledChecks {
                v1: false, v2: false, v3: false, v4: false, v5: true, v6: false,
                v7: false, v8: false, v9: false,
            };
            let (violations, _, _) = run_pipeline(&re_files, &re_errors, &reparser, &v5_only, &l1_ports);
            violations.iter().filter(|v| v.rule_id == "V5").count()
        };

        if !cli.quiet {
            print!("{}", fix_hashes::format_results(&results, unfixable, remaining_v5));
        }

        if remaining_v5 > 0 {
            process::exit(1);
        }
        return;
    }

    // ── --update-snapshot branch ──────────────────────────────────────────────
    if cli.update_snapshot {
        let rewriter = L3SnapshotWriter { nucleo_root: nucleo_root.clone() };
        let entries = update_snapshot::plan(&all_violations, &all_parsed, &rewriter);

        if cli.dry_run {
            if !cli.quiet {
                print!("{}", update_snapshot::format_plan(&entries));
            }
            return;
        }

        let results = update_snapshot::execute(&entries, &rewriter, false);

        // Re-run to count remaining V6
        let remaining_v6 = {
            let reparser = RustParser::new(
                FsPromptReader { nucleo_root: nucleo_root.clone() },
                FsPromptSnapshotReader { nucleo_root: nucleo_root.clone() },
                config.clone(),
            );
            let rewalker = FileWalker::new(cli.path.clone(), config);
            let (re_files, re_errors) = collect_walker_results(rewalker.files());
            let v6_only = EnabledChecks {
                v1: false, v2: false, v3: false, v4: false, v5: false, v6: true,
                v7: false, v8: false, v9: false,
            };
            let (violations, _, _) = run_pipeline(&re_files, &re_errors, &reparser, &v6_only, &l1_ports);
            violations.iter().filter(|v| v.rule_id == "V6").count()
        };

        if !cli.quiet {
            print!("{}", update_snapshot::format_results(&results, remaining_v6));
        }

        if remaining_v6 > 0 {
            process::exit(1);
        }
        return;
    }

    // ── Normal output ─────────────────────────────────────────────────────────
    if !cli.quiet {
        let output = match cli.format {
            OutputFormat::Text => crystalline_lint::shell::cli::format_text(&all_violations),
            OutputFormat::Sarif => crystalline_lint::shell::cli::format_sarif(&all_violations),
        };
        print!("{output}");
    }

    // ── Exit code ─────────────────────────────────────────────────────────────
    if crystalline_lint::shell::cli::should_fail(&all_violations, &cli.fail_on) {
        process::exit(1);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Separates walker results into Ok (SourceFile) and Err (SourceError) buckets.
fn collect_walker_results(
    iter: impl Iterator<Item = Result<SourceFile, SourceError>>,
) -> (Vec<SourceFile>, Vec<SourceError>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for result in iter {
        match result {
            Ok(sf) => files.push(sf),
            Err(e) => errors.push(e),
        }
    }
    (files, errors)
}

// ── L4 adapter: HashRewriter (L3 hash_writer → L2 port) ──────────────────────

struct L3HashRewriter {
    nucleo_root: PathBuf,
}

impl HashRewriter for L3HashRewriter {
    fn read_header(&self, source_path: &Path) -> Option<(String, String)> {
        hash_writer::read_header(source_path)
    }

    fn compute_hash(&self, prompt_path: &str) -> Option<String> {
        FsPromptReader { nucleo_root: self.nucleo_root.clone() }.read_hash(prompt_path)
    }

    fn write_hash(&self, source_path: &Path, new_hash: &str) -> Result<(), String> {
        hash_writer::write_hash(source_path, new_hash)
    }
}

// ── L4 adapter: SnapshotRewriter (L3 snapshot_writer → L2 port) ──────────────

struct L3SnapshotWriter {
    nucleo_root: PathBuf,
}

impl SnapshotRewriter for L3SnapshotWriter {
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String {
        FsPromptSnapshotReader { nucleo_root: self.nucleo_root.clone() }
            .serialize_snapshot(interface)
    }

    fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String> {
        let full_path = self.nucleo_root.join(prompt_path);
        snapshot_writer::write_snapshot(&full_path, snapshot)
    }
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

/// Parses all source files, runs per-file checks (V1–V6, V9), and builds the
/// ProjectIndex for post-reduce checks (V7, V8).
///
/// `source_files` must outlive the returned vecs — `ParsedFile<'a>` and
/// `Violation<'a>` borrow from them (zero-copy, ADR-0004).
fn run_pipeline<'a>(
    source_files: &'a [SourceFile],
    source_errors: &'a [SourceError],
    parser: &RustParser<FsPromptReader, FsPromptSnapshotReader>,
    enabled: &EnabledChecks,
    l1_ports: &L1Ports,
) -> (Vec<Violation<'a>>, Vec<ParsedFile<'a>>, ProjectIndex<'a>) {
    let mut violations: Vec<Violation<'a>> = Vec::new();
    let mut parsed_files = Vec::new();
    let mut project_index = ProjectIndex::new();

    // V0: unreadable files — Fatal, never silenced
    for err in source_errors {
        violations.push(source_error_to_violation(err));
        project_index.merge_local(LocalIndex::from_source_error());
    }

    // V1–V9 per file
    for source_file in source_files {
        match parser.parse(source_file) {
            Ok(parsed) => {
                violations.extend(run_checks(&parsed, enabled, l1_ports));
                project_index.merge_local(LocalIndex::from_parsed(&parsed));
                parsed_files.push(parsed);
            }
            Err(err) => {
                violations.push(parse_error_to_violation(err));
            }
        }
    }

    (violations, parsed_files, project_index)
}

// ── Rule dispatcher ───────────────────────────────────────────────────────────

fn run_checks<'a>(
    file: &ParsedFile<'a>,
    enabled: &EnabledChecks,
    l1_ports: &L1Ports,
) -> Vec<Violation<'a>> {
    let mut violations = Vec::new();
    if enabled.v1 { violations.extend(prompt_header::check(file)); }
    if enabled.v2 { violations.extend(test_file::check(file)); }
    if enabled.v3 { violations.extend(forbidden_import::check(file)); }
    if enabled.v4 { violations.extend(impure_core::check(file)); }
    if enabled.v5 { violations.extend(prompt_drift::check(file)); }
    if enabled.v6 { violations.extend(prompt_stale::check(file)); }
    if enabled.v9 { violations.extend(pub_leak::check(file, l1_ports)); }
    violations
}

// ── Error → Violation converters ─────────────────────────────────────────────

/// V0 — Unreadable source. Fatal, uses Cow::Owned (path not in any SourceFile).
fn source_error_to_violation(err: &SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location {
                path: Cow::Owned(path.clone()),
                line: 0,
                column: 0,
            },
        },
    }
}

/// Converts a parse error into a `Violation<'static>`.
fn parse_error_to_violation(err: ParseError) -> Violation<'static> {
    match err {
        ParseError::SyntaxError { path, line, column, message } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Syntax error: {message}"),
            location: Location { path: Cow::Owned(path), line, column },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Unsupported language: {language:?}"),
            location: Location { path: Cow::Owned(path), line: 0, column: 0 },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Empty source file skipped".to_string(),
            location: Location { path: Cow::Owned(path), line: 0, column: 0 },
        },
    }
}
