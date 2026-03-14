//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 53a76454
//! @layer L4
//! @updated 2026-03-13

use std::path::{Path, PathBuf};
use std::process;

use clap::Parser as ClapParser;

use crystalline_lint::contracts::file_provider::FileProvider;
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::entities::parsed_file::ParsedFile;
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::hash_writer;
use crystalline_lint::infra::prompt_reader::FsPromptReader;
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::infra::walker::FileWalker;
use crystalline_lint::rules::{forbidden_import, impure_core, prompt_drift, prompt_header, test_file};
use crystalline_lint::shell::cli::{validate_args, Cli, EnabledChecks, OutputFormat};
use crystalline_lint::shell::fix_hashes::{self, HashRewriter};

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
    let enabled = EnabledChecks::from_cli(&cli.checks, cli.no_drift);

    let all_violations = {
        let parser = RustParser::new(
            FsPromptReader { nucleo_root: nucleo_root.clone() },
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

        // Re-run analysis to count remaining V5 after fixes
        let remaining_v5 = {
            let reparser = RustParser::new(
                FsPromptReader { nucleo_root: nucleo_root.clone() },
                config.clone(),
            );
            let rewalker = FileWalker::new(cli.path.clone(), config);
            let v5_only = EnabledChecks { v1: false, v2: false, v3: false, v4: false, v5: true };
            run_pipeline(&rewalker, &reparser, &v5_only)
                .into_iter()
                .filter(|v| v.rule_id == "V5")
                .count()
        };

        if !cli.quiet {
            print!("{}", fix_hashes::format_results(&results, unfixable, remaining_v5));
        }

        if remaining_v5 > 0 {
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

// ── L4 adapter: implements L2's HashRewriter using L3 functions ───────────────
// Pure composition — no business logic, just plugs L3 into L2's port.

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

// ── Pipeline helper ───────────────────────────────────────────────────────────

fn run_pipeline(
    walker: &FileWalker,
    parser: &RustParser<FsPromptReader>,
    enabled: &EnabledChecks,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for source_file in walker.files() {
        match parser.parse(source_file) {
            Ok(parsed) => violations.extend(run_checks(&parsed, enabled)),
            Err(err) => violations.push(parse_error_to_violation(err)),
        }
    }
    violations
}

// ── Rule dispatcher ───────────────────────────────────────────────────────────

fn run_checks(file: &ParsedFile, enabled: &EnabledChecks) -> Vec<Violation> {
    let mut violations = Vec::new();
    if enabled.v1 { violations.extend(prompt_header::check(file)); }
    if enabled.v2 { violations.extend(test_file::check(file)); }
    if enabled.v3 { violations.extend(forbidden_import::check(file)); }
    if enabled.v4 { violations.extend(impure_core::check(file)); }
    if enabled.v5 { violations.extend(prompt_drift::check(file)); }
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
