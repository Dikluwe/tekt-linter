//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/sarif-formatter.md
//! @prompt-hash 4384383f
//! @layer L2
//! @updated 2026-03-13

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use serde_json::json;

use crate::entities::violation::{Violation, ViolationLevel};

// ── CLI args ──────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "crystalline-lint", about = "Crystalline Architecture Linter")]
pub struct Cli {
    /// Project root to analyse
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Minimum violation level that triggers exit code 1
    #[arg(long, default_value = "error")]
    pub fail_on: FailLevel,

    /// Comma-separated list of checks to run (e.g. v1,v2,v3,v4,v5)
    #[arg(long, default_value = "v1,v2,v3,v4,v5")]
    pub checks: String,

    /// Disable V5 drift detection
    #[arg(long)]
    pub no_drift: bool,

    /// Only emit exit code, no output
    #[arg(long)]
    pub quiet: bool,

    /// Path to crystalline.toml config
    #[arg(long, default_value = "crystalline.toml")]
    pub config: PathBuf,

    /// Rewrite @prompt-hash headers that diverge from the real L0 hash
    #[arg(long)]
    pub fix_hashes: bool,

    /// Preview changes without rewriting any file (requires --fix-hashes)
    #[arg(long)]
    pub dry_run: bool,
}

/// Returns Err with a user-facing message if the arg combination is invalid.
pub fn validate_args(cli: &Cli) -> Result<(), String> {
    if cli.dry_run && !cli.fix_hashes {
        return Err("--dry-run requires --fix-hashes".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Sarif,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FailLevel {
    Error,
    Warning,
}

// ── Enabled checks ────────────────────────────────────────────────────────────

pub struct EnabledChecks {
    pub v1: bool,
    pub v2: bool,
    pub v3: bool,
    pub v4: bool,
    pub v5: bool,
}

impl EnabledChecks {
    pub fn from_cli(checks: &str, no_drift: bool) -> Self {
        let lower = checks.to_lowercase();
        Self {
            v1: lower.contains("v1"),
            v2: lower.contains("v2"),
            v3: lower.contains("v3"),
            v4: lower.contains("v4"),
            v5: lower.contains("v5") && !no_drift,
        }
    }
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_text(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return format!("{}\n", "✓ No violations found".green().bold());
    }

    let mut out = String::new();
    for v in violations {
        let level_str = match v.level {
            ViolationLevel::Error => "error".red().bold().to_string(),
            ViolationLevel::Warning => "warning".yellow().bold().to_string(),
        };
        out.push_str(&format!(
            "{}: {} [{}]\n   --> {}:{}\n\n",
            level_str,
            v.message,
            v.rule_id.cyan(),
            v.location.path.display(),
            v.location.line,
        ));
    }
    out
}

pub fn format_sarif(violations: &[Violation]) -> String {
    let rules = sarif_rules();

    let results: Vec<serde_json::Value> = violations
        .iter()
        .map(|v| {
            json!({
                "ruleId": v.rule_id,
                "level": sarif_level(&v.level),
                "message": { "text": v.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": v.location.path.to_string_lossy()
                        },
                        "region": {
                            "startLine": v.location.line,
                            "startColumn": v.location.column + 1
                        }
                    }
                }]
            })
        })
        .collect();

    let output = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "crystalline-lint",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/crystalline/lint",
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn sarif_level(level: &ViolationLevel) -> &'static str {
    match level {
        ViolationLevel::Error => "error",
        ViolationLevel::Warning => "warning",
    }
}

fn sarif_rules() -> Vec<serde_json::Value> {
    vec![
        sarif_rule("V1", "MissingPromptHeader", "Missing @prompt lineage header", "error"),
        sarif_rule("V2", "MissingTestFile", "Missing test coverage for L1 module", "error"),
        sarif_rule("V3", "ForbiddenImport", "Import violates layer dependency direction", "error"),
        sarif_rule("V4", "ImpureCore", "I/O operation detected in L1 core", "error"),
        sarif_rule("V5", "PromptDrift", "Prompt hash mismatch — implementation drifted", "warning"),
    ]
}

fn sarif_rule(
    id: &str,
    name: &str,
    description: &str,
    level: &str,
) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "shortDescription": { "text": description },
        "defaultConfiguration": { "level": level }
    })
}

// ── Exit code logic ───────────────────────────────────────────────────────────

pub fn should_fail(violations: &[Violation], fail_on: &FailLevel) -> bool {
    violations.iter().any(|v| match fail_on {
        FailLevel::Error => v.level == ViolationLevel::Error,
        FailLevel::Warning => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::violation::Location;
    use std::path::PathBuf;

    fn make_violation(rule_id: &str, level: ViolationLevel) -> Violation {
        Violation {
            rule_id: rule_id.to_string(),
            level,
            message: "test message".to_string(),
            location: Location { path: PathBuf::from("01_core/foo.rs"), line: 5, column: 0 },
        }
    }

    #[test]
    fn format_text_empty_is_clean() {
        let out = format_text(&[]);
        assert!(out.contains("No violations found"));
    }

    #[test]
    fn format_text_includes_rule_id_and_path() {
        let violations = vec![make_violation("V1", ViolationLevel::Error)];
        let out = format_text(&violations);
        assert!(out.contains("V1"));
        assert!(out.contains("foo.rs"));
        assert!(out.contains('5'.to_string().as_str()));
    }

    #[test]
    fn format_sarif_is_valid_json() {
        let violations = vec![make_violation("V3", ViolationLevel::Error)];
        let out = format_sarif(&violations);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V3");
    }

    #[test]
    fn format_sarif_empty_violations() {
        let out = format_sarif(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(parsed["runs"][0]["results"].as_array().unwrap().is_empty());
    }

    #[test]
    fn should_fail_on_error() {
        let v = vec![make_violation("V1", ViolationLevel::Error)];
        assert!(should_fail(&v, &FailLevel::Error));
    }

    #[test]
    fn should_not_fail_on_warning_when_fail_on_error() {
        let v = vec![make_violation("V5", ViolationLevel::Warning)];
        assert!(!should_fail(&v, &FailLevel::Error));
    }

    #[test]
    fn should_fail_on_warning_when_fail_on_warning() {
        let v = vec![make_violation("V5", ViolationLevel::Warning)];
        assert!(should_fail(&v, &FailLevel::Warning));
    }

    #[test]
    fn enabled_checks_no_drift_disables_v5() {
        let checks = EnabledChecks::from_cli("v1,v2,v3,v4,v5", true);
        assert!(!checks.v5);
        assert!(checks.v1);
    }

    #[test]
    fn dry_run_without_fix_hashes_is_error() {
        let cli = Cli {
            path: PathBuf::from("."),
            format: OutputFormat::Text,
            fail_on: FailLevel::Error,
            checks: "v1,v2,v3,v4,v5".to_string(),
            no_drift: false,
            quiet: false,
            config: PathBuf::from("crystalline.toml"),
            fix_hashes: false,
            dry_run: true,
        };
        assert!(validate_args(&cli).is_err());
    }

    #[test]
    fn dry_run_with_fix_hashes_is_ok() {
        let cli = Cli {
            path: PathBuf::from("."),
            format: OutputFormat::Text,
            fail_on: FailLevel::Error,
            checks: "v1,v2,v3,v4,v5".to_string(),
            no_drift: false,
            quiet: false,
            config: PathBuf::from("crystalline.toml"),
            fix_hashes: true,
            dry_run: true,
        };
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn fix_hashes_alone_is_ok() {
        let cli = Cli {
            path: PathBuf::from("."),
            format: OutputFormat::Text,
            fail_on: FailLevel::Error,
            checks: "v1,v2,v3,v4,v5".to_string(),
            no_drift: false,
            quiet: false,
            config: PathBuf::from("crystalline.toml"),
            fix_hashes: true,
            dry_run: false,
        };
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn enabled_checks_subset() {
        let checks = EnabledChecks::from_cli("v1,v3", false);
        assert!(checks.v1);
        assert!(!checks.v2);
        assert!(checks.v3);
        assert!(!checks.v4);
        assert!(!checks.v5);
    }
}
