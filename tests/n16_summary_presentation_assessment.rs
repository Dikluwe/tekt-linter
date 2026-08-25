//! Blind B2 gate for the presentation half of the P0094 N16 summary contract.
//!
//! The L0 publishes the pure L2 formatter, but no nominal public wiring-validation
//! API. CLI selection/exit-status observation therefore remains a SPEC-GAP for this
//! gate instead of being tested through an invented function.

use crystalline_lint::shell::n16_summary::{format_n16_summary, N16ModuleStats, N16Stats};

fn stats(rows: &[(&str, usize, usize, usize)]) -> N16Stats {
    rows.iter()
        .map(|(name, alpha, beta, gamma)| {
            (
                (*name).to_owned(),
                N16ModuleStats {
                    alpha: *alpha,
                    beta: *beta,
                    gamma: *gamma,
                },
            )
        })
        .collect()
}

fn row_position(output: &str, module: &str) -> usize {
    output
        .lines()
        .position(|line| line.contains(module))
        .unwrap_or_else(|| panic!("missing row for {module:?} in:\n{output}"))
}

#[test]
fn orders_by_gamma_descending_then_module_name_by_utf8_bytes_only() {
    let input = stats(&[
        ("z-low-percent/", 18, 0, 2),
        ("é/", 0, 0, 1),
        ("z/", 98, 0, 1),
        ("a/", 0, 0, 1),
        ("gamma-first/", 97, 0, 3),
    ]);

    let output = format_n16_summary(&input, 0);

    assert!(row_position(&output, "gamma-first/") < row_position(&output, "z-low-percent/"));
    assert!(row_position(&output, "z-low-percent/") < row_position(&output, "a/"));
    assert!(row_position(&output, "a/") < row_position(&output, "z/"));
    assert!(row_position(&output, "z/") < row_position(&output, "é/"));
}

#[test]
fn renders_normative_columns_totals_zero_gamma_and_half_up_tenths() {
    let input = stats(&[
        ("half-up/", 15, 0, 1), // 6.25% -> 6.3%
        ("zero-gamma/", 1, 2, 0),
        ("two-thirds/", 0, 1, 2),
    ]);

    let output = format_n16_summary(&input, 0);

    assert!(output.contains("Módulo"));
    assert!(output.contains("Total"));
    assert!(output.contains("α"));
    assert!(output.contains("β"));
    assert!(output.contains("γ"));
    assert!(output.contains("% γ"));
    assert!(output
        .lines()
        .any(|line| line.contains("half-up/") && line.contains("6.3%")));
    assert!(output
        .lines()
        .any(|line| line.contains("zero-gamma/") && line.contains("0.0%")));
    assert!(output
        .lines()
        .any(|line| line.contains("two-thirds/") && line.contains("66.7%")));
    assert!(output.lines().any(|line| {
        line.contains("Total")
            && line.contains("22")
            && line.contains("16")
            && line.contains("3")
            && line.contains("13.6%")
    }));
}

#[test]
fn empty_report_has_headers_and_total_dash_but_no_small_sample_warning() {
    let output = format_n16_summary(&N16Stats::new(), 5);

    assert!(output.contains("Módulo"));
    assert!(output
        .lines()
        .any(|line| line.contains("Total") && line.contains('—')));
    assert!(!output.contains("amostra pequena"));
    assert!(!output.contains("~pp"));
}

#[test]
fn warns_every_small_module_including_zero_gamma_and_rounds_pp_half_up() {
    let input = stats(&[
        ("gamma-zero/", 8, 0, 0),
        ("gamma-one/", 7, 0, 1),
        ("at-threshold/", 9, 0, 0),
    ]);

    let output = format_n16_summary(&input, 9);
    let warnings: Vec<_> = output
        .lines()
        .filter(|line| line.contains("amostra pequena"))
        .collect();

    assert_eq!(warnings.len(), 2, "unexpected warnings:\n{output}");
    assert!(warnings
        .iter()
        .all(|line| line.contains("n=8") && line.contains("~13pp")));
    assert!(!warnings.iter().any(|line| line.contains("n=9")));
}

#[test]
fn thresholds_zero_and_one_never_warn_for_emitted_modules() {
    let input = stats(&[("one/", 1, 0, 0)]);

    for threshold in [0, 1] {
        let output = format_n16_summary(&input, threshold);
        assert!(
            !output.contains("amostra pequena"),
            "threshold={threshold}:\n{output}"
        );
        assert!(!output.contains("~pp"), "threshold={threshold}:\n{output}");
    }
}

#[test]
fn hostile_names_and_extreme_but_non_overflowing_counts_remain_deterministic() {
    let huge = usize::MAX / 8;
    let input = stats(&[
        ("N16[γ]-decoy/", huge, 0, 0),
        ("percent%—emoji-🧪/", 0, huge, 0),
        ("Ω/", 0, 0, 1),
    ]);

    let first = format_n16_summary(&input, usize::MAX);
    let second = format_n16_summary(&input, usize::MAX);

    assert_eq!(first, second);
    assert!(first.contains("N16[γ]-decoy/"));
    assert!(first.contains("percent%—emoji-🧪/"));
    assert!(first.contains("Ω/"));
    assert!(!first.contains("NaN"));
    assert!(!first.contains("inf"));
}
