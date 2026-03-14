//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash b47a45c9
//! @layer L4
//! @updated 2026-03-14

use std::path::{Path, PathBuf};
use std::process;

use clap::Parser as ClapParser;

use crystalline_lint::contracts::file_provider::FileProvider;
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::parsed_file::{ParsedFile, PublicInterface};
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::hash_writer;
use crystalline_lint::infra::prompt_reader::FsPromptReader;
use crystalline_lint::infra::prompt_snapshot_reader::FsPromptSnapshotReader;
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::infra::snapshot_writer;
use crystalline_lint::infra::walker::FileWalker;
use crystalline_lint::rules::{
    forbidden_import, impure_core, prompt_drift, prompt_header, prompt_stale, test_file,
};
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

    // ── Instantiate L3 components ─────────────────────────────────────────────
    let nucleo_root = PathBuf::from(".");
    let enabled = EnabledChecks::from_cli(&cli.checks, cli.no_drift, cli.no_stale);

    let (all_violations, all_parsed) = {
        let parser = RustParser::new(
            FsPromptReader { nucleo_root: nucleo_root.clone() },
            FsPromptSnapshotReader { nucleo_root: nucleo_root.clone() },
            config.clone(),
        );
        let walker = FileWalker::new(cli.path.clone(), config.clone());
        run_pipeline(&walker, &parser, &enabled)
    };

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
            let rewalker = FileWalker::new(cli.path.clone(), config);
            let v5_only = EnabledChecks {
                v1: false, v2: false, v3: false, v4: false, v5: true, v6: false,
            };
            let (violations, _) = run_pipeline(&rewalker, &reparser, &v5_only);
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
            let v6_only = EnabledChecks {
                v1: false, v2: false, v3: false, v4: false, v5: false, v6: true,
            };
            let (violations, _) = run_pipeline(&rewalker, &reparser, &v6_only);
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
    fn serialize_snapshot(&self, interface: &PublicInterface) -> String {
        FsPromptSnapshotReader { nucleo_root: self.nucleo_root.clone() }
            .serialize_snapshot(interface)
    }

    fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String> {
        let full_path = self.nucleo_root.join(prompt_path);
        snapshot_writer::write_snapshot(&full_path, snapshot)
    }
}

// ── Pipeline helper ───────────────────────────────────────────────────────────

fn run_pipeline(
    walker: &FileWalker,
    parser: &RustParser<FsPromptReader, FsPromptSnapshotReader>,
    enabled: &EnabledChecks,
) -> (Vec<Violation>, Vec<ParsedFile>) {
    let mut violations = Vec::new();
    let mut parsed_files = Vec::new();
    for source_file in walker.files() {
        match parser.parse(source_file) {
            Ok(parsed) => {
                violations.extend(run_checks(&parsed, enabled));
                parsed_files.push(parsed);
            }
            Err(err) => violations.push(parse_error_to_violation(err)),
        }
    }
    (violations, parsed_files)
}

// ── Rule dispatcher ───────────────────────────────────────────────────────────

fn run_checks(file: &ParsedFile, enabled: &EnabledChecks) -> Vec<Violation> {
    let mut violations = Vec::new();
    if enabled.v1 { violations.extend(prompt_header::check(file)); }
    if enabled.v2 { violations.extend(test_file::check(file)); }
    if enabled.v3 { violations.extend(forbidden_import::check(file)); }
    if enabled.v4 { violations.extend(impure_core::check(file)); }
    if enabled.v5 { violations.extend(prompt_drift::check(file)); }
    if enabled.v6 { violations.extend(prompt_stale::check(file)); }
    violations
}

// ── ParseError → Violation ────────────────────────────────────────────────────

fn parse_error_to_violation(err: ParseError) -> Violation {
    match err {
        ParseError::SyntaxError { path, line, column, message } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Syntax error: {message}"),
            location: Location { path, line, column },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Unsupported language: {language:?}"),
            location: Location { path, line: 0, column: 0 },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Empty source file skipped".to_string(),
            location: Location { path, line: 0, column: 0 },
        },
    }
}
