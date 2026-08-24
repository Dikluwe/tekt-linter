//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/sarif-formatter.md
//! @prompt-hash cd9ff6ab
//! @layer L2
//! @updated 2026-03-22

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use serde_json::json;

use crate::entities::layer::Layer;
use crate::entities::parsed_file::{ImportKind, ParsedFile};
use crate::entities::violation::{Violation, ViolationLevel};
use crate::shell::path_encoding::{human_path, machine_path_uri};

// ── CLI args ──────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "crystalline-lint", about = "Crystalline Architecture Linter")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<crate::shell::refinement::Command>,

    /// Project root to analyse
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Minimum violation level that triggers exit code 1
    #[arg(long, default_value = "error")]
    pub fail_on: FailLevel,

    /// Comma-separated list of checks to run (e.g. v1,v2,...,v12)
    /// V11 (dangling-contract) is opt-in — not included in the default because
    /// rule_traits in L1/contracts/ are implemented by ParsedFile (L1), not L2/L3.
    #[arg(
        long,
        default_value = "v1,v2,v3,v4,v5,v6,v7,v8,v9,v10,v11,v12,v13,v14,v15,v16,v17,v18,v19,v20,v21,v23,v24,v25"
    )]
    pub checks: String,

    /// Disable V5 drift detection
    #[arg(long)]
    pub no_drift: bool,

    /// Disable V6 stale detection
    #[arg(long)]
    pub no_stale: bool,

    /// Only emit exit code, no output
    #[arg(long)]
    pub quiet: bool,

    /// Path to crystalline.toml config
    #[arg(long, default_value = "crystalline.toml")]
    pub config: PathBuf,

    /// Rewrite @prompt-hash headers that diverge from the real L0 hash
    #[arg(long)]
    pub fix_hashes: bool,

    /// Update Interface Snapshot in prompt files for all V6 violations
    #[arg(long)]
    pub update_snapshot: bool,

    /// Preview changes without rewriting any file (requires --fix-hashes or --update-snapshot)
    #[arg(long)]
    pub dry_run: bool,

    /// Emit the resolution of EVERY import as JSON Lines (incl. Unknown) and exit.
    /// Instrumentation for the differential oracle (0058) — outside the verdict seal:
    /// changes no rule, only dumps what `classify_import` resolved per import.
    #[arg(long)]
    pub emit_resolution: bool,
}

/// Returns Err with a user-facing message if the arg combination is invalid.
pub fn validate_args(cli: &Cli) -> Result<(), String> {
    if cli.fix_hashes && cli.update_snapshot {
        return Err("--fix-hashes and --update-snapshot cannot be used simultaneously".to_string());
    }
    if cli.dry_run && !cli.fix_hashes && !cli.update_snapshot {
        return Err("--dry-run requires --fix-hashes or --update-snapshot".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Sarif,
    #[value(name = "n16-summary")]
    N16Summary,
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
    pub v6: bool,
    pub v7: bool,
    pub v8: bool,
    pub v9: bool,
    pub v10: bool,
    /// V11 is opt-in (not in default --checks) because rule_traits in
    /// L1/contracts/ are implemented by ParsedFile (L1), not L2/L3.
    pub v11: bool,
    pub v12: bool,
    pub v13: bool,
    pub v14: bool,
    pub v15: bool,
    pub v16: bool,
    pub v17: bool,
    pub v18: bool,
    pub v19: bool,
    pub v20: bool,
    pub v21: bool,
    pub v22: bool,
    pub v23: bool,
    pub v24: bool,
    pub v25: bool,
}

impl EnabledChecks {
    pub fn from_cli(checks: &str, no_drift: bool, no_stale: bool) -> Self {
        let tokens: std::collections::HashSet<&str> = checks
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let has = |id: &str| -> bool { tokens.contains("all") || tokens.contains(id) };

        Self {
            v1: has("v1"),
            v2: has("v2"),
            v3: has("v3"),
            v4: has("v4"),
            v5: has("v5") && !no_drift,
            v6: has("v6") && !no_stale,
            v7: has("v7"),
            v8: has("v8"),
            v9: has("v9"),
            v10: has("v10"),
            v11: has("v11"),
            v12: has("v12"),
            v13: has("v13"),
            v14: has("v14"),
            v15: has("v15"),
            v16: has("v16"),
            v17: has("v17"),
            v18: has("v18"),
            v19: has("v19"),
            v20: has("v20"),
            v21: has("v21"),
            v22: has("v22"),
            v23: has("v23"),
            v24: has("v24"),
            v25: has("v25"),
        }
    }
}

// ── Formatters ────────────────────────────────────────────────────────────────

pub fn format_text(violations: &[Violation<'_>]) -> String {
    if violations.is_empty() {
        return format!("{}\n", "✓ No violations found".green().bold());
    }

    let mut out = String::new();
    for v in violations {
        let level_str = match v.level {
            ViolationLevel::Fatal => "fatal".red().bold().to_string(),
            ViolationLevel::Error => "error".red().bold().to_string(),
            ViolationLevel::Warning => "warning".yellow().bold().to_string(),
            ViolationLevel::Info => "info".blue().bold().to_string(),
        };
        out.push_str(&format!(
            "{}: {} [{}]\n   --> {}:{}\n\n",
            level_str,
            v.message,
            v.rule_id.cyan(),
            human_path(&v.location.path),
            v.location.line,
        ));
    }
    out
}

pub fn format_sarif(violations: &[Violation<'_>]) -> String {
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
                            "uri": machine_path_uri(&v.location.path)
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
                    "name": "tekt-linter",
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

/// Emite a resolução de todo import do workspace como JSON Lines (uma por linha),
/// **incluindo os `Unknown`** (que o SARIF esconde, pois não geram violação).
/// Instrumentação para o oráculo diferencial (0058) — fora do selo de veredito.
///
/// Cada linha: `{source, source_layer, import, first_segment, kind, target_layer,
/// target_subdir, is_unknown}`. O `first_segment` é o candidato a crate/módulo-alvo
/// (1º segmento do path, `-`→`_`) — a chave de aresta que o oráculo alinha contra a lente.
/// `resolve_crate(source_file, first_segment) -> crate real` aplica a resolução do
/// próprio linter (renomeação por-membro do `crate_registry`) ao nome de superfície,
/// para o emit reportar o crate resolvido — o que o oráculo alinha. A closure vem de
/// L4 (que tem o registry); L2 não depende de L3.
pub fn format_resolution(
    parsed: &[ParsedFile<'_>],
    resolve_crate: &dyn Fn(&std::path::Path, &str) -> String,
) -> String {
    let mut out = String::new();
    for file in parsed {
        let source = machine_path_uri(file.path);
        let source_layer = layer_str(&file.layer);
        for import in &file.imports {
            let surface = first_segment(import.path);
            let target_crate = resolve_crate(file.path, &surface);
            let line = json!({
                "source": source,
                "source_layer": source_layer,
                "import": import.path,
                "first_segment": surface,
                "target_crate": target_crate, // resolvido (pós-rename) — chave de aresta do oráculo
                "kind": import_kind_str(&import.kind),
                "target_layer": layer_str(&import.target_layer),
                "target_subdir": import.target_subdir,
                "is_unknown": import.target_layer == Layer::Unknown,
                // Emite tudo, marcado (segue verdadeiro, 0061) — o oráculo projeta as
                // arestas test-origin fora SIMETRICAMENTE, igual a lente as exclui.
                "is_test_origin": import.is_test_origin,
            });
            out.push_str(&line.to_string());
            out.push('\n');
        }
    }
    out
}

/// 1º segmento de um path de import, `-`→`_` (candidato a nome de crate/módulo-alvo).
/// Corta no primeiro de `::` ou ` as ` (alias de crate — cego #1, 0059), para o
/// oráculo enxergar a aresta real de `use crate_x as y;`.
fn first_segment(path: &str) -> String {
    let p = path.trim_start_matches('{').trim();
    let end = [p.find("::"), p.find(" as ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(p.len());
    p[..end].trim().replace('-', "_")
}

fn layer_str(layer: &Layer) -> &'static str {
    match layer {
        Layer::L0 => "L0",
        Layer::L1 => "L1",
        Layer::L2 => "L2",
        Layer::L3 => "L3",
        Layer::L4 => "L4",
        Layer::Lab => "Lab",
        Layer::Unknown => "Unknown",
    }
}

fn import_kind_str(kind: &ImportKind) -> &'static str {
    match kind {
        ImportKind::Direct => "Direct",
        ImportKind::Glob => "Glob",
        ImportKind::Alias => "Alias",
        ImportKind::Named => "Named",
    }
}

fn sarif_level(level: &ViolationLevel) -> &'static str {
    match level {
        ViolationLevel::Fatal => "error",
        ViolationLevel::Error => "error",
        ViolationLevel::Warning => "warning",
        ViolationLevel::Info => "note",
    }
}

fn sarif_rules() -> Vec<serde_json::Value> {
    vec![
        sarif_rule(
            "V0",
            "UnreadableSource",
            "Unreadable source file — I/O error",
            "error",
        ),
        sarif_rule(
            "V1",
            "MissingPromptHeader",
            "Missing @prompt lineage header",
            "error",
        ),
        sarif_rule(
            "V2",
            "MissingTestFile",
            "Missing test coverage for L1 module",
            "error",
        ),
        sarif_rule(
            "V3",
            "ForbiddenImport",
            "Import violates layer dependency direction",
            "error",
        ),
        sarif_rule(
            "V4",
            "ImpureCore",
            "I/O operation detected in L1 core",
            "error",
        ),
        sarif_rule(
            "V5",
            "PromptDrift",
            "Prompt hash mismatch — implementation drifted",
            "warning",
        ),
        sarif_rule(
            "V6",
            "PromptStale",
            "Public interface changed since last prompt snapshot",
            "warning",
        ),
        sarif_rule(
            "V7",
            "OrphanPrompt",
            "Prompt without any materialization in L1–L4",
            "warning",
        ),
        sarif_rule(
            "V8",
            "AlienFile",
            "Source file outside all mapped layers",
            "error",
        ),
        sarif_rule(
            "V9",
            "PubLeak",
            "Import bypasses L1 encapsulation boundary",
            "error",
        ),
        sarif_rule(
            "V10",
            "QuarantineLeak",
            "Production code imports from lab/ quarantine",
            "error",
        ),
        sarif_rule(
            "V11",
            "DanglingContract",
            "Contract trait without implementation in L2/L3",
            "error",
        ),
        sarif_rule(
            "V12",
            "WiringLogicLeak",
            "Type declaration in L4 wiring layer",
            "warning",
        ),
        sarif_rule(
            "V13",
            "MutableStateInCore",
            "Global mutable state in L1 core",
            "error",
        ),
        sarif_rule(
            "V14",
            "ExternalTypeInContract",
            "Unauthorized external dependency in L1",
            "error",
        ),
        sarif_rule(
            "V15",
            "MultiPromptHeader",
            "Multiple @prompt headers in one file",
            "error",
        ),
        sarif_rule(
            "V16",
            "WildcardSaturation",
            "Catchall arm discards domain enum information",
            "warning",
        ),
        sarif_rule(
            "V17",
            "CompoundGuard",
            "Compound boolean logic in pattern guard",
            "warning",
        ),
        sarif_rule(
            "V18",
            "RangePatternInMatch",
            "Range pattern in domain match expression",
            "warning",
        ),
        sarif_rule(
            "V19",
            "OrPatternAlternatives",
            "Decision arm condenses multiple or-pattern alternatives",
            "note",
        ),
        sarif_rule(
            "V20",
            "DeepPatternNesting",
            "Pattern nesting depth exceeds threshold",
            "note",
        ),
        sarif_rule(
            "V21",
            "HardcodedContextualValue",
            "Hardcoded contextual scalar feeds geometric sink",
            "warning",
        ),
        sarif_rule(
            "V22",
            "ProvenanceInventory",
            "Module-level aggregate provenance ratio metric",
            "note",
        ),
        sarif_rule(
            "V23",
            "ContextErasure",
            "Required semantic context is erased before a declared sink",
            "warning",
        ),
        sarif_rule(
            "V24",
            "SemanticFieldLoss",
            "Required identity field is replaced by a neutral value",
            "warning",
        ),
        sarif_rule(
            "V25",
            "DecisionOwnership",
            "Semantic decision is recomposed outside its declared owner",
            "warning",
        ),
    ]
}

fn sarif_rule(id: &str, name: &str, description: &str, level: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "shortDescription": { "text": description },
        "defaultConfiguration": { "level": level }
    })
}

// ── Exit code logic ───────────────────────────────────────────────────────────

/// Ordena violações de forma determinística: Fatal → Error → Warning,
/// depois por path crescente, depois por linha crescente.
/// Elimina não-determinismo do pipeline paralelo rayon.
pub fn sort_violations(violations: &mut Vec<Violation<'_>>) {
    violations.sort_by(|a, b| {
        a.level
            .cmp(&b.level)
            .reverse()
            .then_with(|| a.location.path.cmp(&b.location.path))
            .then_with(|| a.location.line.cmp(&b.location.line))
            .then_with(|| a.location.column.cmp(&b.location.column))
            .then_with(|| a.rule_id.cmp(&b.rule_id))
            .then_with(|| a.message.cmp(&b.message))
    });
}

pub fn should_fail(violations: &[Violation<'_>], fail_on: &FailLevel) -> bool {
    violations.iter().any(|v| {
        if v.level == ViolationLevel::Fatal {
            return true; // Fatal always fails — cannot be suppressed by --fail-on
        }
        match fail_on {
            FailLevel::Error => v.level == ViolationLevel::Error,
            FailLevel::Warning => {
                v.level == ViolationLevel::Error || v.level == ViolationLevel::Warning
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::violation::Location;
    use std::borrow::Cow;
    use std::path::{Path, PathBuf};

    fn make_violation(rule_id: &str, level: ViolationLevel) -> Violation<'static> {
        Violation {
            rule_id: rule_id.to_string(),
            level,
            message: "test message".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new("01_core/foo.rs")),
                line: 5,
                column: 0,
            },
        }
    }

    fn make_violation_at(
        rule_id: &'static str,
        level: ViolationLevel,
        path: &'static str,
        line: usize,
    ) -> Violation<'static> {
        Violation {
            rule_id: rule_id.to_string(),
            level,
            message: "test".to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new(path)),
                line,
                column: 0,
            },
        }
    }

    /// sort_violations: Fatal antes de Error antes de Warning,
    /// desempate por path, desempate por linha.
    #[test]
    fn violations_sorted_fatal_first_then_path_then_line() {
        let mut v = vec![
            make_violation_at("V3", ViolationLevel::Warning, "b.rs", 1),
            make_violation_at("V0", ViolationLevel::Fatal, "a.rs", 5),
            make_violation_at("V2", ViolationLevel::Error, "a.rs", 2),
            make_violation_at("V1", ViolationLevel::Warning, "a.rs", 1),
        ];
        sort_violations(&mut v);
        assert_eq!(v[0].level, ViolationLevel::Fatal);
        assert_eq!(v[1].level, ViolationLevel::Error);
        assert_eq!(v[2].level, ViolationLevel::Warning);
        assert_eq!(v[2].location.line, 1); // a.rs:1 antes de b.rs:1
        assert_eq!(v[3].location.path.as_ref(), Path::new("b.rs"));
    }

    #[test]
    fn violations_have_total_deterministic_tie_breakers() {
        let make = |column, rule: &str, message: &str| Violation {
            rule_id: rule.to_string(),
            level: ViolationLevel::Warning,
            message: message.to_string(),
            location: Location {
                path: Cow::Borrowed(Path::new("same.rs")),
                line: 1,
                column,
            },
        };
        let mut violations = vec![
            make(2, "V2", "a"),
            make(1, "V2", "b"),
            make(1, "V1", "z"),
            make(1, "V1", "a"),
        ];
        sort_violations(&mut violations);
        assert_eq!(
            violations
                .iter()
                .map(|v| (v.location.column, v.rule_id.as_str(), v.message.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (1, "V1", "a"),
                (1, "V1", "z"),
                (1, "V2", "b"),
                (2, "V2", "a")
            ]
        );
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
        let checks = EnabledChecks::from_cli("v1,v2,v3,v4,v5,v6", true, false);
        assert!(!checks.v5);
        assert!(checks.v1);
        assert!(checks.v6);
    }

    fn base_cli() -> Cli {
        Cli {
            command: None,
            path: PathBuf::from("."),
            format: OutputFormat::Text,
            fail_on: FailLevel::Error,
            checks: "v1,v2,v3,v4,v5,v6,v7,v8,v9,v10,v11,v12".to_string(),
            no_drift: false,
            no_stale: false,
            quiet: false,
            config: PathBuf::from("crystalline.toml"),
            fix_hashes: false,
            update_snapshot: false,
            dry_run: false,
            emit_resolution: false,
        }
    }

    #[test]
    fn dry_run_without_fix_hashes_is_error() {
        let cli = Cli {
            dry_run: true,
            ..base_cli()
        };
        assert!(validate_args(&cli).is_err());
    }

    #[test]
    fn dry_run_with_fix_hashes_is_ok() {
        let cli = Cli {
            fix_hashes: true,
            dry_run: true,
            ..base_cli()
        };
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn dry_run_with_update_snapshot_is_ok() {
        let cli = Cli {
            update_snapshot: true,
            dry_run: true,
            ..base_cli()
        };
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn fix_hashes_alone_is_ok() {
        let cli = Cli {
            fix_hashes: true,
            ..base_cli()
        };
        assert!(validate_args(&cli).is_ok());
    }

    #[test]
    fn fix_hashes_and_update_snapshot_is_error() {
        let cli = Cli {
            fix_hashes: true,
            update_snapshot: true,
            ..base_cli()
        };
        assert!(validate_args(&cli).is_err());
    }

    #[test]
    fn enabled_checks_subset() {
        let checks = EnabledChecks::from_cli("v1,v3", false, false);
        assert!(checks.v1);
        assert!(!checks.v2);
        assert!(checks.v3);
        assert!(!checks.v4);
        assert!(!checks.v5);
        assert!(!checks.v6);
    }

    #[test]
    fn no_stale_disables_v6() {
        let checks = EnabledChecks::from_cli("v1,v2,v3,v4,v5,v6", false, true);
        assert!(!checks.v6);
        assert!(checks.v5);
    }

    #[test]
    fn checks_v11_does_not_activate_v1_or_v2() {
        let checks = EnabledChecks::from_cli("v11", false, false);
        assert!(checks.v11);
        assert!(!checks.v1);
        assert!(!checks.v2);
    }

    #[test]
    fn checks_v12_does_not_activate_v1_or_v2() {
        let checks = EnabledChecks::from_cli("v12", false, false);
        assert!(checks.v12);
        assert!(!checks.v1);
        assert!(!checks.v2);
    }

    #[test]
    fn checks_v11_and_v12_together() {
        let checks = EnabledChecks::from_cli("v11,v12", false, false);
        assert!(checks.v11);
        assert!(checks.v12);
        assert!(!checks.v1);
        assert!(!checks.v2);
    }

    #[test]
    fn checks_v15_does_not_activate_v1_or_v5() {
        let checks = EnabledChecks::from_cli("v15", false, false);
        assert!(checks.v15);
        assert!(!checks.v1);
        assert!(!checks.v5);
    }

    #[test]
    fn checks_all_activates_v15() {
        let checks = EnabledChecks::from_cli("all", false, false);
        assert!(checks.v15);
    }

    #[test]
    fn checks_with_spaces_around_tokens() {
        let checks = EnabledChecks::from_cli("v1, v3", false, false);
        assert!(checks.v1);
        assert!(checks.v3);
        assert!(!checks.v2);
    }

    // ── Critérios adicionais do prompt sarif-formatter.md ─────────────────────

    #[test]
    fn checks_all_activates_all_v1_to_v12() {
        // Dado --checks all → todos v1..v12 = true
        let checks = EnabledChecks::from_cli("all", false, false);
        assert!(checks.v1);
        assert!(checks.v2);
        assert!(checks.v3);
        assert!(checks.v4);
        assert!(checks.v5);
        assert!(checks.v6);
        assert!(checks.v7);
        assert!(checks.v8);
        assert!(checks.v9);
        assert!(checks.v10);
        assert!(checks.v11);
        assert!(checks.v12);
        assert!(checks.v16);
        assert!(checks.v17);
        assert!(checks.v18);
        assert!(checks.v19);
        assert!(checks.v20);
        assert!(checks.v21);
        assert!(checks.v22);
        assert!(checks.v23);
        assert!(checks.v24);
        assert!(checks.v25);
    }

    #[test]
    fn checks_unknown_token_silently_ignored() {
        // Dado --checks v1,v99 → v1=true, sem panic — token desconhecido ignorado
        let checks = EnabledChecks::from_cli("v1,v99", false, false);
        assert!(checks.v1);
        assert!(!checks.v2);
        // sem panic — este teste já valida isso ao completar
    }

    #[test]
    fn sarif_v7_warning_level() {
        // Dado Vec<Violation> com V7 warning
        // Quando format_sarif() for chamado
        // Então SARIF contém resultado com ruleId "V7" e level "warning"
        let v = vec![make_violation("V7", ViolationLevel::Warning)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V7");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "warning");
    }

    #[test]
    fn sarif_v8_fatal_mapped_to_error() {
        // V8 é Fatal — SARIF 2.1.0 não tem "fatal", deve ser mapeado para "error"
        let v = vec![make_violation("V8", ViolationLevel::Fatal)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V8");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "error");
    }

    #[test]
    fn sarif_v10_fatal_mapped_to_error() {
        // V10 Fatal mapeado para "error" no SARIF — idêntico ao tratamento de V0 e V8
        let v = vec![make_violation("V10", ViolationLevel::Fatal)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V10");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "error");
    }

    #[test]
    fn sarif_v11_error_level() {
        // Dado Vec<Violation> com V11 error
        let v = vec![make_violation("V11", ViolationLevel::Error)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V11");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "error");
    }

    #[test]
    fn sarif_v12_warning_level() {
        // Dado Vec<Violation> com V12 warning
        let v = vec![make_violation("V12", ViolationLevel::Warning)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V12");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "warning");
    }

    #[test]
    fn should_fail_on_fatal_regardless_of_fail_on_setting() {
        // Fatal bloqueia CI incondicionalmente — mesmo com --fail-on error
        let v = vec![make_violation("V10", ViolationLevel::Fatal)];
        assert!(should_fail(&v, &FailLevel::Error));
    }

    #[test]
    fn sarif_driver_rules_has_26_entries() {
        // SARIF driver.rules contém V0–V25.
        let out = format_sarif(&[]);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let rules = parsed["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .unwrap();
        assert_eq!(
            rules.len(),
            26,
            "expected 26 rules (V0 to V25), got {}",
            rules.len()
        );
    }

    #[test]
    fn checks_v22_activates_v22() {
        let checks = EnabledChecks::from_cli("v22", false, false);
        assert!(checks.v22);
        assert!(!checks.v1);
    }

    #[test]
    fn cli_accepts_n16_summary_format() {
        use clap::Parser;
        let cli = Cli::try_parse_from(&["crystalline-lint", "--format", "n16-summary"]).unwrap();
        assert_eq!(cli.format, OutputFormat::N16Summary);
    }

    #[test]
    fn checks_v21_activates_v21() {
        let checks = EnabledChecks::from_cli("v21", false, false);
        assert!(checks.v21);
        assert!(!checks.v1);
    }

    #[test]
    fn semantic_checks_are_isolated() {
        let v23 = EnabledChecks::from_cli("v23", false, false);
        assert!(v23.v23);
        assert!(!v23.v24 && !v23.v25);
        let v24 = EnabledChecks::from_cli("v24", false, false);
        assert!(v24.v24);
        assert!(!v24.v23 && !v24.v25);
        let v25 = EnabledChecks::from_cli("v25", false, false);
        assert!(v25.v25);
        assert!(!v25.v23 && !v25.v24);
    }
    #[test]
    fn sarif_info_mapped_to_note() {
        let v = vec![make_violation("V19", ViolationLevel::Info)];
        let out = format_sarif(&v);
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["runs"][0]["results"][0]["ruleId"], "V19");
        assert_eq!(parsed["runs"][0]["results"][0]["level"], "note");
    }

    #[test]
    fn should_not_fail_on_info_even_when_fail_on_warning() {
        let v = vec![make_violation("V19", ViolationLevel::Info)];
        assert!(!should_fail(&v, &FailLevel::Warning));
        assert!(!should_fail(&v, &FailLevel::Error));
    }
}
