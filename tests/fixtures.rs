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

// ── Fechamento da extração (0055) — V6/V12/V2/V4 ──────────────────────────────
// Mata os sobreviventes de extração que o corpo 0054 deixou adiados. A fixture V6
// do 0054 só variava `reexports`; estas exercitam `functions` e `types`, e a
// extração de campos/variantes/métodos/genéricos.

// V6 pass com interface RICA idêntica ao snapshot: função tipada, struct com
// campos, enum com variantes, trait com método, struct genérico. Qualquer mutação
// num extrator muda a interface lida → quebra a igualdade → V6 espúrio → falha aqui.
#[test]
fn v06b_pass_rich_interface_identical() { assert_verdict("v06b_pass", &[]); }

// V6 fail com delta SÓ em functions (snapshot omite uma função do código).
#[test]
fn v06c_fail_functions_delta() { assert_verdict("v06c_fail", &["V6"]); }

// V6 fail com delta SÓ em types (snapshot tem um struct com um campo a menos).
#[test]
fn v06d_fail_types_delta() { assert_verdict("v06d_fail", &["V6"]); }

// V12 com genéricos no L4: enum genérico falha; struct adaptador genérico passa.
#[test]
fn v12b_fail_generic_enum_in_wiring() { assert_verdict("v12b_fail", &["V12"]); }
#[test]
fn v12b_pass_generic_struct_in_wiring() { assert_verdict("v12b_pass", &[]); }

// V2 bordas de cobertura: impl SÓ com const (sem fn) é declaração-só → isento;
// `#[cfg(feature=...)]` que NÃO é teste não conta como cobertura → ainda dispara.
#[test]
fn v02b_pass_impl_without_function_exempt() { assert_verdict("v02b_pass", &[]); }
#[test]
fn v02c_fail_cfg_not_test_still_uncovered() { assert_verdict("v02c_fail", &["V2"]); }

// V2: "cfg(test)" como TEXTO num comentário (não atributo) não conta como cobertura
// → ainda dispara. Prova que `check_cfg_test` chaveia no nó-atributo, não na mera
// presença do texto — mata os mutantes que afrouxam o casamento de `kind`.
#[test]
fn v02d_fail_cfg_test_only_in_comment() { assert_verdict("v02d_fail", &["V2"]); }

// V4 via macro com prefixo proibido (`std::fs::x!()`) — exercita o ramo
// `macro_invocation` da extração de tokens, que o par V4 do 0054 (call) não tocava.
#[test]
fn v04b_fail_forbidden_macro_in_core() { assert_verdict("v04b_fail", &["V4"]); }

// ── Caminho do veredito (0057) — config / walker / prompt-IO ──────────────────
// Põe sob oráculo os botões de config e bordas de walker que produzem o veredito
// mas que nenhuma fixture variava. Cada uma mata um mutante veredito-mudante fora
// das regras (em config.rs / walker.rs / prompt_reader.rs).

// V12 config: com allow_adapter_structs=false, struct no L4 passa a disparar V12
// (par do default true em v12_pass). Mata o mutante do botão de config.
#[test]
fn v12c_fail_struct_when_adapter_disallowed() { assert_verdict("v12c_fail", &["V12"]); }

// [excluded]: violação real (ficheiro sem header) num dir excluído → 0. Mata os
// mutantes que quebram a exclusão do walker (o ficheiro apareceria como V1/V8).
#[test]
fn vexcl_pass_violation_in_excluded_dir() { assert_verdict("vexcl_pass", &[]); }

// Recursão do walker: V1 num subdir profundamente aninhado tem de ser achada.
// Mata o mutante que para a recursão / aborta a descida em subdiretórios.
#[test]
fn vnest_fail_violation_in_nested_subdir() { assert_verdict("vnest_fail", &["V1"]); }

// V14 config: thiserror é externo PERMITIDO em L1 ([l1_allowed_external]) → sem
// V14. Mata o mutante que zera `l1_allowed_for_language` (faria thiserror falhar).
#[test]
fn v14b_pass_allowed_external_in_core() { assert_verdict("v14b_pass", &[]); }

// V5 leitura de hash: header com @prompt-hash CORRETO (igual ao hash do prompt) →
// sem V5. Mata os mutantes de `read_hash` (qualquer hash errado → drift espúrio).
#[test]
fn v05b_pass_correct_hash_no_drift() { assert_verdict("v05b_pass", &[]); }

// V1 existência de prompt: header aponta para um prompt INEXISTENTE → V1. Mata os
// mutantes de `exists` que sempre retornam true (esconderiam o prompt ausente).
#[test]
fn v01b_fail_header_points_to_missing_prompt() { assert_verdict("v01b_fail", &["V1"]); }

// V3 via [module_layers]: módulo mapeado a L4; L2 importando `crate::wiremod` →
// V3. Mata o mutante que apaga o arm L4 de `layer_for_module` (viraria Unknown).
#[test]
fn vmod_l4_fail_module_mapped_to_l4() { assert_verdict("vmod_l4_fail", &["V3"]); }

// Walker L0: um `.rs` em 00_nucleo (camada L0) com header → sem violação. Mata o
// mutante que apaga o arm L0 de `resolve_file_layer` (o ficheiro viraria alienígena/V8).
#[test]
fn vl0_pass_rust_file_in_l0() { assert_verdict("vl0_pass", &[]); }

// V14: prova a distinção do 0052 — `use serde::…` (externo real) FALHA, enquanto
// `use corehelper::…` (first-party L1, mesma camada) no MESMO ficheiro NÃO falha.
// Por isso o veredito é exatamente um V14, não dois.
#[test]
fn v14_fail_external_dep_in_core() { assert_verdict("v14_fail", &["V14"]); }
