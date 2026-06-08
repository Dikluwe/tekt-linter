//! Harness de fixtures bite-proof (prompt 0054 — corpo de fixtures).
//!
//! Roda o binário `crystalline-lint` sobre cada workspace de fixture em
//! `tests/fixtures/vNN_{pass,fail}/` e afirma o **conjunto exato** de IDs de
//! violação (com multiplicidade), não apenas sucesso/fracasso. Esses testes são
//! o oráculo que a mutação (`cargo-mutants`) usa para provar completude contra o
//! motor de regras: um mutante que sobrevive é um ramo que nenhuma fixture morde.
//!
//! O linter é exercido como caixa-preta — o harness não liga contra a lib, só
//! invoca o binário com `current_dir` na fixture (replicando `crystalline-lint .`
//! com o `crystalline.toml` e o `00_nucleo/` da própria fixture).

use std::path::PathBuf;
use std::process::Command;

/// Caminho da fixture relativo ao crate.
fn fixture_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Extrai os `ruleId` do SARIF sem dependências de parsing: varre as linhas
/// procurando `"ruleId": "VNN"`. O formato é `to_string_pretty`, uma por linha.
fn rule_ids_from_sarif(sarif: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in sarif.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("\"ruleId\":") {
            // rest = ` "V3",`
            if let Some(start) = rest.find('"') {
                let after = &rest[start + 1..];
                if let Some(end) = after.find('"') {
                    ids.push(after[..end].to_string());
                }
            }
        }
    }
    ids
}

/// Roda o linter na fixture e devolve os IDs de violação, ordenados (multiset).
fn violations(name: &str) -> Vec<String> {
    let dir = fixture_dir(name);
    assert!(dir.is_dir(), "fixture inexistente: {}", dir.display());

    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(&dir)
        .args(["--format", "sarif", "."])
        .output()
        .expect("falha ao executar o binário crystalline-lint");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // O SARIF deve estar bem-formado: presença do driver é uma sanidade barata.
    assert!(
        stdout.contains("\"crystalline-lint\""),
        "saída SARIF inesperada para {name}:\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = rule_ids_from_sarif(&stdout);
    ids.sort();
    ids
}

/// Afirma o conjunto exato de violações (IDs + contagem) de uma fixture.
fn assert_verdict(name: &str, expected: &[&str]) {
    let got = violations(name);
    let mut want: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
    want.sort();
    assert_eq!(
        got, want,
        "veredito divergente para fixture '{name}': esperado {want:?}, obtido {got:?}"
    );
}

// ── Caminhos que PASSAM — nenhuma violação ─────────────────────────────────────

#[test]
fn v01_pass_clean() { assert_verdict("v01_pass", &[]); }
#[test]
fn v02_pass_clean() { assert_verdict("v02_pass", &[]); }
#[test]
fn v03_pass_clean() { assert_verdict("v03_pass", &[]); }
#[test]
fn v04_pass_clean() { assert_verdict("v04_pass", &[]); }
#[test]
fn v05_pass_clean() { assert_verdict("v05_pass", &[]); }
#[test]
fn v06_pass_clean() { assert_verdict("v06_pass", &[]); }
#[test]
fn v07_pass_clean() { assert_verdict("v07_pass", &[]); }
#[test]
fn v08_pass_clean() { assert_verdict("v08_pass", &[]); }
#[test]
fn v09_pass_clean() { assert_verdict("v09_pass", &[]); }
#[test]
fn v10_pass_clean() { assert_verdict("v10_pass", &[]); }
#[test]
fn v11_pass_clean() { assert_verdict("v11_pass", &[]); }
#[test]
fn v12_pass_clean() { assert_verdict("v12_pass", &[]); }
#[test]
fn v13_pass_clean() { assert_verdict("v13_pass", &[]); }
#[test]
fn v14_pass_clean() { assert_verdict("v14_pass", &[]); }

// ── Caminhos que FALHAM — conjunto fixado de violações ──────────────────────────

#[test]
fn v01_fail_missing_header() { assert_verdict("v01_fail", &["V1"]); }
#[test]
fn v02_fail_l1_without_test() { assert_verdict("v02_fail", &["V2"]); }
#[test]
fn v03_fail_cross_crate_l2_to_l4() { assert_verdict("v03_fail", &["V3"]); }
#[test]
fn v04_fail_io_in_core() { assert_verdict("v04_fail", &["V4"]); }
#[test]
fn v05_fail_hash_drift() { assert_verdict("v05_fail", &["V5"]); }
#[test]
fn v06_fail_interface_stale() { assert_verdict("v06_fail", &["V6"]); }
#[test]
fn v07_fail_orphan_prompt() { assert_verdict("v07_fail", &["V7"]); }
#[test]
fn v08_fail_alien_file() { assert_verdict("v08_fail", &["V8"]); }
#[test]
fn v09_fail_cross_crate_nonport_subdir() { assert_verdict("v09_fail", &["V9"]); }

// V9 intra-crate: `use crate::internal;` (2 segmentos, módulo L1 não-porta). O
// ramo `crate::`/`super::` de `resolve_subdir` resolve o subdir mesmo com 2
// segmentos, ao contrário do ramo cross-crate (que exige ≥3). Sem este caso, um
// mutante que funde os dois ramos sobrevive — a fixture cross-crate não o morde.
#[test]
fn v09_fail_intra_crate_nonport_subdir() { assert_verdict("v09b_fail_intra", &["V9"]); }

// V10 co-ocorre com V3: qualquer import de produção→lab é proibido pela direção
// (V3) e pela quarentena (V10) ao mesmo tempo — não há camada-origem que dispare
// um sem o outro. O veredito honesto da fixture é o par.
#[test]
fn v10_fail_quarantine_leak() { assert_verdict("v10_fail", &["V10", "V3"]); }

#[test]
fn v11_fail_dangling_contract() { assert_verdict("v11_fail", &["V11"]); }
#[test]
fn v12_fail_enum_in_wiring() { assert_verdict("v12_fail", &["V12"]); }
#[test]
fn v13_fail_mutable_static_in_core() { assert_verdict("v13_fail", &["V13"]); }

// V14: prova a distinção do 0052 — `use serde::…` (externo real) FALHA, enquanto
// `use corehelper::…` (first-party L1, mesma camada) no MESMO ficheiro NÃO falha.
// Por isso o veredito é exatamente um V14, não dois.
#[test]
fn v14_fail_external_dep_in_core() { assert_verdict("v14_fail", &["V14"]); }
