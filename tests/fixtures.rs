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
        stdout.contains("\"tekt-linter\""),
        "saída SARIF inesperada para {name}:\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut ids = rule_ids_from_sarif(&stdout);
    ids.sort();
    ids
}

fn violations_with_checks(name: &str, checks: &str) -> Vec<String> {
    let dir = fixture_dir(name);
    let output = Command::new(env!("CARGO_BIN_EXE_crystalline-lint"))
        .current_dir(&dir)
        .args(["--format", "sarif", "--checks", checks, "."])
        .output()
        .expect("falha ao executar o binário crystalline-lint");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\"tekt-linter\""),
        "SARIF inválido: {stdout}"
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
fn v01_pass_clean() {
    assert_verdict("v01_pass", &[]);
}
#[test]
fn v02_pass_clean() {
    assert_verdict("v02_pass", &[]);
}
#[test]
fn v03_pass_clean() {
    assert_verdict("v03_pass", &[]);
}
#[test]
fn v04_pass_clean() {
    assert_verdict("v04_pass", &[]);
}
#[test]
fn v05_pass_clean() {
    assert_verdict("v05_pass", &[]);
}
#[test]
fn v06_pass_clean() {
    assert_verdict("v06_pass", &[]);
}
#[test]
fn v07_pass_clean() {
    assert_verdict("v07_pass", &[]);
}
#[test]
fn v08_pass_clean() {
    assert_verdict("v08_pass", &[]);
}
#[test]
fn v09_pass_clean() {
    assert_verdict("v09_pass", &[]);
}
#[test]
fn v10_pass_clean() {
    assert_verdict("v10_pass", &[]);
}
#[test]
fn v11_pass_clean() {
    assert_verdict("v11_pass", &[]);
}
#[test]
fn v12_pass_clean() {
    assert_verdict("v12_pass", &[]);
}
#[test]
fn v13_pass_clean() {
    assert_verdict("v13_pass", &[]);
}
#[test]
fn v14_pass_clean() {
    assert_verdict("v14_pass", &[]);
}
#[test]
fn v15_pass_clean() {
    assert_verdict("v15_pass", &[]);
}

// V15 negativa: "@prompt" em comentário // normal, fora do bloco de
// doc-header, não conta — só as linhas //! do topo alimentam prompt_refs.
#[test]
fn v15b_pass_prompt_mentioned_in_plain_comment() {
    assert_verdict("v15b_pass", &[]);
}

#[test]
fn semantic_fixture_all_rules_have_exact_multiplicity() {
    assert_eq!(
        violations_with_checks("v23_v25_semantic", "v23,v24,v25"),
        vec!["V23", "V23", "V24", "V25", "V25", "V25"],
    );
}

#[test]
fn semantic_fixture_selection_is_isolated() {
    assert_eq!(
        violations_with_checks("v23_v25_semantic", "v23"),
        vec!["V23", "V23"]
    );
    assert_eq!(
        violations_with_checks("v23_v25_semantic", "v24"),
        vec!["V24"]
    );
    assert_eq!(
        violations_with_checks("v23_v25_semantic", "v25"),
        vec!["V25", "V25", "V25"]
    );
}

// ── Caminhos que FALHAM — conjunto fixado de violações ──────────────────────────

#[test]
fn v01_fail_missing_header() {
    assert_verdict("v01_fail", &["V1"]);
}
#[test]
fn v02_fail_l1_without_test() {
    assert_verdict("v02_fail", &["V2"]);
}
#[test]
fn v03_fail_cross_crate_l2_to_l4() {
    assert_verdict("v03_fail", &["V3"]);
}
#[test]
fn v04_fail_io_in_core() {
    assert_verdict("v04_fail", &["V4"]);
}
#[test]
fn v04_go_fail_io_in_core() {
    assert_verdict("v04_go_fail", &["V4"]);
}
#[test]
fn v04_zig_fail_io_in_core() {
    assert_verdict("v04_zig_fail", &["V4"]);
}
#[test]
fn v04_java_fail_io_in_core() {
    assert_verdict("v04_java_fail", &["V4"]);
}
#[test]
fn v04_elixir_fail_io_in_core() {
    assert_verdict("v04_elixir_fail", &["V4"]);
}
#[test]
fn v05_fail_hash_drift() {
    assert_verdict("v05_fail", &["V5"]);
}
#[test]
fn v06_fail_interface_stale() {
    assert_verdict("v06_fail", &["V6"]);
}
#[test]
fn v07_fail_orphan_prompt() {
    assert_verdict("v07_fail", &["V7"]);
}
#[test]
fn v08_fail_alien_file() {
    assert_verdict("v08_fail", &["V8"]);
}
#[test]
fn v09_fail_cross_crate_nonport_subdir() {
    assert_verdict("v09_fail", &["V9"]);
}

// V9 intra-crate: `use crate::internal;` (2 segmentos, módulo L1 não-porta). O
// ramo `crate::`/`super::` de `resolve_subdir` resolve o subdir mesmo com 2
// segmentos, ao contrário do ramo cross-crate (que exige ≥3). Sem este caso, um
// mutante que funde os dois ramos sobrevive — a fixture cross-crate não o morde.
#[test]
fn v09_fail_intra_crate_nonport_subdir() {
    assert_verdict("v09b_fail_intra", &["V9"]);
}

// V10 co-ocorre com V3: qualquer import de produção→lab é proibido pela direção
// (V3) e pela quarentena (V10) ao mesmo tempo — não há camada-origem que dispare
// um sem o outro. O veredito honesto da fixture é o par.
#[test]
fn v10_fail_quarantine_leak() {
    assert_verdict("v10_fail", &["V10", "V3"]);
}

#[test]
fn v11_fail_dangling_contract() {
    assert_verdict("v11_fail", &["V11"]);
}
#[test]
fn v12_fail_enum_in_wiring() {
    assert_verdict("v12_fail", &["V12"]);
}
#[test]
fn v13_fail_mutable_static_in_core() {
    assert_verdict("v13_fail", &["V13"]);
}

// V15: duas linhas `//! @prompt` no doc-header — a regra de linhagem é
// um ficheiro, um prompt; --fix-hashes é indefinido com multi-@prompt,
// por isso o lint bloqueia em vez de corrigir ambiguamente.
#[test]
fn v15_fail_multi_prompt_header() {
    assert_verdict("v15_fail", &["V15"]);
}

// ── Fechamento da extração (0055) — V6/V12/V2/V4 ──────────────────────────────
// Mata os sobreviventes de extração que o corpo 0054 deixou adiados. A fixture V6
// do 0054 só variava `reexports`; estas exercitam `functions` e `types`, e a
// extração de campos/variantes/métodos/genéricos.

// V6 pass com interface RICA idêntica ao snapshot: função tipada, struct com
// campos, enum com variantes, trait com método, struct genérico. Qualquer mutação
// num extrator muda a interface lida → quebra a igualdade → V6 espúrio → falha aqui.
#[test]
fn v06b_pass_rich_interface_identical() {
    assert_verdict("v06b_pass", &[]);
}

// V6 fail com delta SÓ em functions (snapshot omite uma função do código).
#[test]
fn v06c_fail_functions_delta() {
    assert_verdict("v06c_fail", &["V6"]);
}

// V6 fail com delta SÓ em types (snapshot tem um struct com um campo a menos).
#[test]
fn v06d_fail_types_delta() {
    assert_verdict("v06d_fail", &["V6"]);
}

// V12 com genéricos no L4: enum genérico falha; struct adaptador genérico passa.
#[test]
fn v12b_fail_generic_enum_in_wiring() {
    assert_verdict("v12b_fail", &["V12"]);
}
#[test]
fn v12b_pass_generic_struct_in_wiring() {
    assert_verdict("v12b_pass", &[]);
}

// V2 bordas de cobertura: impl SÓ com const (sem fn) é declaração-só → isento;
// `#[cfg(feature=...)]` que NÃO é teste não conta como cobertura → ainda dispara.
#[test]
fn v02b_pass_impl_without_function_exempt() {
    assert_verdict("v02b_pass", &[]);
}
#[test]
fn v02c_fail_cfg_not_test_still_uncovered() {
    assert_verdict("v02c_fail", &["V2"]);
}

// V2: "cfg(test)" como TEXTO num comentário (não atributo) não conta como cobertura
// → ainda dispara. Prova que `check_cfg_test` chaveia no nó-atributo, não na mera
// presença do texto — mata os mutantes que afrouxam o casamento de `kind`.
#[test]
fn v02d_fail_cfg_test_only_in_comment() {
    assert_verdict("v02d_fail", &["V2"]);
}

// V4 via macro com prefixo proibido (`std::fs::x!()`) — exercita o ramo
// `macro_invocation` da extração de tokens, que o par V4 do 0054 (call) não tocava.
#[test]
fn v04b_fail_forbidden_macro_in_core() {
    assert_verdict("v04b_fail", &["V4"]);
}

// ── Caminho do veredito (0057) — config / walker / prompt-IO ──────────────────
// Põe sob oráculo os botões de config e bordas de walker que produzem o veredito
// mas que nenhuma fixture variava. Cada uma mata um mutante veredito-mudante fora
// das regras (em config.rs / walker.rs / prompt_reader.rs).

// V12 config: com allow_adapter_structs=false, struct no L4 passa a disparar V12
// (par do default true em v12_pass). Mata o mutante do botão de config.
#[test]
fn v12c_fail_struct_when_adapter_disallowed() {
    assert_verdict("v12c_fail", &["V12"]);
}

// [excluded]: violação real (ficheiro sem header) num dir excluído → 0. Mata os
// mutantes que quebram a exclusão do walker (o ficheiro apareceria como V1/V8).
#[test]
fn vexcl_pass_violation_in_excluded_dir() {
    assert_verdict("vexcl_pass", &[]);
}

// Recursão do walker: V1 num subdir profundamente aninhado tem de ser achada.
// Mata o mutante que para a recursão / aborta a descida em subdiretórios.
#[test]
fn vnest_fail_violation_in_nested_subdir() {
    assert_verdict("vnest_fail", &["V1"]);
}

// V14 config: thiserror é externo PERMITIDO em L1 ([l1_allowed_external]) → sem
// V14. Mata o mutante que zera `l1_allowed_for_language` (faria thiserror falhar).
#[test]
fn v14b_pass_allowed_external_in_core() {
    assert_verdict("v14b_pass", &[]);
}

// V5 leitura de hash: header com @prompt-hash CORRETO (igual ao hash do prompt) →
// sem V5. Mata os mutantes de `read_hash` (qualquer hash errado → drift espúrio).
#[test]
fn v05b_pass_correct_hash_no_drift() {
    assert_verdict("v05b_pass", &[]);
}

// V1 existência de prompt: header aponta para um prompt INEXISTENTE → V1. Mata os
// mutantes de `exists` que sempre retornam true (esconderiam o prompt ausente).
#[test]
fn v01b_fail_header_points_to_missing_prompt() {
    assert_verdict("v01b_fail", &["V1"]);
}

// V3 via [module_layers]: módulo mapeado a L4; L2 importando `crate::wiremod` →
// V3. Mata o mutante que apaga o arm L4 de `layer_for_module` (viraria Unknown).
#[test]
fn vmod_l4_fail_module_mapped_to_l4() {
    assert_verdict("vmod_l4_fail", &["V3"]);
}

// Walker L0: um `.rs` em 00_nucleo (camada L0) com header → sem violação. Mata o
// mutante que apaga o arm L0 de `resolve_file_layer` (o ficheiro viraria alienígena/V8).
#[test]
fn vl0_pass_rust_file_in_l0() {
    assert_verdict("vl0_pass", &[]);
}

// ── Conserto do resolvedor (0059) — cegos #1 (alias) e #3 (dep renomeada) ─────
// Import cross-crate em direção PROIBIDA (L2→L4) por um nome de superfície que o
// resolvedor cego não enxergava. O V3 só aparece DEPOIS do conserto — antes, o
// import virava LocalItem (#1) ou Unknown (#3), invisível ao V3.

// #1 alias no `use`: `use wiremod as w;` (wiremod é L4) → V3 (sufixo ` as w` removido).
#[test]
fn v03c_fail_alias_use_resolves_crate() {
    assert_verdict("v03c_fail_alias", &["V3"]);
}

// #3 dep renomeada: `alias = { package = "b" }`, `use alias::…` (b é L4) → V3
// (chave resolvida ao pacote real pelo mapa por-membro do crate_registry).
#[test]
fn v03d_fail_renamed_dep_resolves_crate() {
    assert_verdict("v03d_fail_rename", &["V3"]);
}

// ── Conserto do cego #2 (0060) — referência cross-crate por caminho fora do `use` ─
// `wiremod::ITEM` (L4) num ficheiro L2, SEM `use`. A aresta L2→L4 (e o V3) só aparece
// DEPOIS do conserto — antes, a extração só visitava `use`/`extern crate`.
// Positivas: cada posição estruturada/atributo morde exatamente um V3.

// A — expressão: `wiremod::go();` num corpo de função.
#[test]
fn v03e_fail_pathref_expr() {
    assert_verdict("v03e_fail_pathref_expr", &["V3"]);
}

// A — tipo: `-> wiremod::Thing` em posição de tipo.
#[test]
fn v03f_fail_pathref_type() {
    assert_verdict("v03f_fail_pathref_type", &["V3"]);
}

// B — atributo: `#[arg(default_value_t = wiremod::N)]` (varredura de token_tree).
#[test]
fn v03g_fail_pathref_attr() {
    assert_verdict("v03g_fail_pathref_attr", &["V3"]);
}

// Negativas (guarda contra falso-positivo): caminho local e stdlib NÃO criam aresta.
// Local é bite-proof: o módulo interno `wire` está mapeado L4, logo se a guarda
// `crate::` caísse, `crate::wire::Thing` resolveria L2→L4 e um V3 espúrio nasceria.
#[test]
fn v03h_pass_pathref_local() {
    assert_verdict("v03h_pass_pathref_local", &[]);
}

// Std é robustez: stdlib é isento a jusante por construção, então a sua referência
// inline tem de continuar a dar 0 violação.
#[test]
fn v03i_pass_pathref_std() {
    assert_verdict("v03i_pass_pathref_std", &[]);
}

// V14: prova a distinção do 0052 — `use serde::…` (externo real) FALHA, enquanto
// `use corehelper::…` (first-party L1, mesma camada) no MESMO ficheiro NÃO falha.
// Por isso o veredito é exatamente um V14, não dois.
#[test]
fn v14_fail_external_dep_in_core() {
    assert_verdict("v14_fail", &["V14"]);
}

// ── Excluir #[cfg(test)] da gravidade (0061) ──────────────────────────────────
// A MESMA aresta proibida L1→L3 dentro de `#[cfg(test)]`: o default (excluir teste)
// dá [], a opção `check_test_imports = true` reabre o gate → [V3]. O par
// default-pass/on-fail é o bite-proof dos dois lados. Cobertas as DUAS vias de
// coleta: `use` (collect_imports) e path-ref (collect_path_refs, do 0060).

// Via `use` em `#[cfg(test)] mod` — default exclui.
#[test]
fn vtest_default_pass_use() {
    assert_verdict("vtest_default_pass", &[]);
}

// Mesma fonte, `check_test_imports = true` — o gate abre.
#[test]
fn vtest_on_fail_use() {
    assert_verdict("vtest_on_fail", &["V3"]);
}

// Via path-ref (fora do `use`) dentro de `#[cfg(test)] fn` — default exclui.
#[test]
fn vtest_pathref_default_pass() {
    assert_verdict("vtest_pathref_default_pass", &[]);
}

// Mesma fonte, opção ligada — path-ref test-origin volta a morder.
#[test]
fn vtest_pathref_on_fail() {
    assert_verdict("vtest_pathref_on_fail", &["V3"]);
}

// Não-regressão de produção: a MESMA aresta L1→L3 FORA de teste segue [V3] (a
// mudança só tira arestas test-origin — produção nunca é test-origin).
#[test]
fn vtest_prod_fail() {
    assert_verdict("vtest_prod_fail", &["V3"]);
}

#[path = "fixtures/ghost_variant.rs"]
mod ghost_variant;

#[path = "fixtures/error_message_arm.rs"]
mod error_message_arm;

// ── Não-regressão V16–V20 em TypeScript e Python (ADR-0016) ───────────────────

#[test]
fn non_regression_v16_to_v20_on_typescript() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::HasDecisionArms;
    use crystalline_lint::rules::wildcard_saturation;
    use std::collections::HashMap;
    use std::path::Path;

    struct MockTsFile {
        path: &'static Path,
    }
    impl HasDecisionArms<'static> for MockTsFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn decision_exprs(
            &self,
        ) -> &[crystalline_lint::entities::rule_traits::DecisionExpr<'static>] {
            &[]
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::TypeScript
        }
    }

    let file = MockTsFile {
        path: Path::new("01_core/index.ts"),
    };
    let exceptions = HashMap::new();
    let viols = wildcard_saturation::check(&file, &exceptions);
    assert!(
        viols.is_empty(),
        "TypeScript file must produce zero V16 violations"
    );
}

#[test]
fn non_regression_v16_to_v20_on_python() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::HasDecisionArms;
    use crystalline_lint::rules::wildcard_saturation;
    use std::collections::HashMap;
    use std::path::Path;

    struct MockPyFile {
        path: &'static Path,
    }
    impl HasDecisionArms<'static> for MockPyFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn decision_exprs(
            &self,
        ) -> &[crystalline_lint::entities::rule_traits::DecisionExpr<'static>] {
            &[]
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Python
        }
    }

    let file = MockPyFile {
        path: Path::new("01_core/main.py"),
    };
    let exceptions = HashMap::new();
    let viols = wildcard_saturation::check(&file, &exceptions);
    assert!(
        viols.is_empty(),
        "Python file must produce zero V16 violations"
    );
}

// ── V21 & V22 Fixtures & Critérios de Aceitação (Passo 0066) ───────────────────

#[test]
fn v21_hardcoded_contextual_value_triggers_warning() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{ConstantKind, HasConstants, SourceConstant};
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("layout/src/raw.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "0.9",
            line: 10,
            column: 8,
            citation: None,
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: true,
            context_var: Some("size".to_string()),
            geometric_sink: Some("gap".to_string()),
            is_in_data_table: false,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert_eq!(viols.len(), 1, "escalar contextual fixo dispara V21");
    assert_eq!(viols[0].rule_id, "V21");
    assert_eq!(
        viols[0].level,
        crystalline_lint::entities::violation::ViolationLevel::Warning
    );
    assert!(viols[0].message.contains("0.9"));
    assert!(viols[0].message.contains("size"));
    assert!(viols[0].message.contains("gap"));
}

#[test]
fn v21_isolated_literal_outside_scaling_does_not_trigger() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{ConstantKind, HasConstants, SourceConstant};
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("layout/src/frame.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "12.5",
            line: 20,
            column: 8,
            citation: None,
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: false,
            context_var: None,
            geometric_sink: Some("gap".to_string()),
            is_in_data_table: false,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert!(viols.is_empty(), "literal isolado nao dispara V21");
}

#[test]
fn v21_data_table_is_exempt() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{ConstantKind, HasConstants, SourceConstant};
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("layout/src/table.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "0.9",
            line: 30,
            column: 8,
            citation: None,
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: true,
            context_var: Some("size".to_string()),
            geometric_sink: Some("gap".to_string()),
            is_in_data_table: true,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert!(viols.is_empty(), "tabela de dados é isenta de V21");
}

#[test]
fn v21_format_syntax_module_is_exempt() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{ConstantKind, HasConstants, SourceConstant};
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("export/pdf/stream.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "0.9",
            line: 10,
            column: 8,
            citation: None,
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: true,
            context_var: Some("width".to_string()),
            geometric_sink: Some("pos".to_string()),
            is_in_data_table: false,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert!(
        viols.is_empty(),
        "modulo de sintaxe de formato é isento de V21"
    );
}

#[test]
fn v21_valid_ref_suppresses() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{
        Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
    };
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;

    let mut temp = NamedTempFile::new().unwrap();
    writeln!(temp, "// spec line 1").unwrap();
    let temp_path = temp.path().to_str().unwrap().to_string();

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("layout/src/font.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "0.9",
            line: 10,
            column: 8,
            citation: Some(Citation {
                kind: CitationKind::Ref {
                    path: Box::leak(temp_path.into_boxed_str()),
                    line: 1,
                },
                raw: "// ref: ...",
                line: 9,
            }),
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: true,
            context_var: Some("size".to_string()),
            geometric_sink: Some("gap".to_string()),
            is_in_data_table: false,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert!(viols.is_empty(), "// ref: válido apaga o aviso");
}

#[test]
fn v21_stale_ref_triggers_stale_citation_warning() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{
        Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
    };
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let file = MockFile {
        path: Path::new("layout/src/font.rs"),
        constants: vec![SourceConstant {
            kind: ConstantKind::FunctionNumberLiteral,
            snippet: "0.9",
            line: 25,
            column: 4,
            citation: Some(Citation {
                kind: CitationKind::Ref {
                    path: "non_existent_file.md",
                    line: 10,
                },
                raw: "// ref: non_existent_file.md:10",
                line: 24,
            }),
            is_test_origin: false,
            function_return_type: None,
            is_in_binary_scaling: true,
            context_var: Some("size".to_string()),
            geometric_sink: Some("gap".to_string()),
            is_in_data_table: false,
        }],
    };

    let viols = check(&file, &V21RuleConfig::default(), None);
    assert_eq!(viols.len(), 1, "ref: obsoleto dispara StaleCitation");
    assert_eq!(
        viols[0].level,
        crystalline_lint::entities::violation::ViolationLevel::Warning
    );
    assert!(viols[0].message.contains("Citação obsoleta"));
}

#[test]
fn v22_provenance_inventory_aggregates_module_ratio() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{
        Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
    };
    use crystalline_lint::rules::provenance_inventory;
    use crystalline_lint::rules::unsourced_constant::V21RuleConfig;
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let files = vec![MockFile {
        path: Path::new("layout/src/flow.rs"),
        constants: vec![
            SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "10.5",
                line: 10,
                column: 0,
                citation: Some(Citation {
                    kind: CitationKind::Rationale("teste"),
                    raw: "// rationale: teste",
                    line: 9,
                }),
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: None,
                is_in_data_table: false,
            },
            SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "20.5",
                line: 12,
                column: 0,
                citation: None,
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: None,
                is_in_data_table: false,
            },
        ],
    }];

    let viols = provenance_inventory::check_inventory(&files, &V21RuleConfig::default());
    assert_eq!(
        viols.len(),
        1,
        "V22 emite exatamente 1 linha agregada por módulo"
    );
    assert_eq!(viols[0].rule_id, "V22");
    assert_eq!(
        viols[0].level,
        crystalline_lint::entities::violation::ViolationLevel::Info
    );
    assert!(viols[0].message.contains("1/2"));
    assert!(viols[0].message.contains("50.0%"));
}

#[test]
fn non_regression_v21_and_v22_on_typescript() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::HasConstants;
    use crystalline_lint::rules::provenance_inventory;
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockTsFile {
        path: &'static Path,
    }
    impl HasConstants<'static> for MockTsFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[crystalline_lint::entities::rule_traits::SourceConstant<'static>] {
            &[]
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::TypeScript
        }
    }

    let file = MockTsFile {
        path: Path::new("01_core/index.ts"),
    };
    let viols21 = check(&file, &V21RuleConfig::default(), None);
    assert!(
        viols21.is_empty(),
        "TypeScript file must produce zero V21 violations"
    );
    let viols22 = provenance_inventory::check_inventory(&[file], &V21RuleConfig::default());
    assert!(
        viols22.is_empty(),
        "TypeScript file must produce zero V22 violations"
    );
}

#[test]
fn non_regression_v21_and_v22_on_python() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::HasConstants;
    use crystalline_lint::rules::provenance_inventory;
    use crystalline_lint::rules::unsourced_constant::{check, V21RuleConfig};
    use std::path::Path;

    struct MockPyFile {
        path: &'static Path,
    }
    impl HasConstants<'static> for MockPyFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[crystalline_lint::entities::rule_traits::SourceConstant<'static>] {
            &[]
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Python
        }
    }

    let file = MockPyFile {
        path: Path::new("01_core/main.py"),
    };
    let viols21 = check(&file, &V21RuleConfig::default(), None);
    assert!(
        viols21.is_empty(),
        "Python file must produce zero V21 violations"
    );
    let viols22 = provenance_inventory::check_inventory(&[file], &V21RuleConfig::default());
    assert!(
        viols22.is_empty(),
        "Python file must produce zero V22 violations"
    );
}

#[test]
fn v22_recognizes_real_world_citations_in_inventory() {
    use crystalline_lint::entities::layer::{Language, Layer};
    use crystalline_lint::entities::rule_traits::{
        Citation, CitationKind, ConstantKind, HasConstants, SourceConstant,
    };
    use crystalline_lint::rules::provenance_inventory;
    use crystalline_lint::rules::unsourced_constant::V21RuleConfig;
    use std::path::Path;

    struct MockFile {
        path: &'static Path,
        constants: Vec<SourceConstant<'static>>,
    }
    impl HasConstants<'static> for MockFile {
        fn layer(&self) -> &Layer {
            &Layer::L1
        }
        fn constants(&self) -> &[SourceConstant<'static>] {
            &self.constants
        }
        fn path(&self) -> &'static Path {
            self.path
        }
        fn language(&self) -> &Language {
            &Language::Rust
        }
    }

    let files = vec![MockFile {
        path: Path::new("layout/src/equation.rs"),
        constants: vec![
            // P813 — container.rs:342
            SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.9",
                line: 119,
                column: 8,
                citation: Some(Citation {
                    kind: CitationKind::Ref {
                        path: "lab/typst-original/src/layout/container.rs",
                        line: 342,
                    },
                    raw: "// P813 — layout — lab/typst-original/src/layout/container.rs:342",
                    line: 117,
                }),
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: true,
                context_var: Some("size".to_string()),
                geometric_sink: Some("gap".to_string()),
                is_in_data_table: false,
            },
            // vanilla resolve.rs:1173
            SourceConstant {
                kind: ConstantKind::FunctionNumberLiteral,
                snippet: "0.85",
                line: 125,
                column: 8,
                citation: Some(Citation {
                    kind: CitationKind::Ref {
                        path: "resolve.rs",
                        line: 1173,
                    },
                    raw: "// vanilla resolve.rs:1173",
                    line: 124,
                }),
                is_test_origin: false,
                function_return_type: None,
                is_in_binary_scaling: false,
                context_var: None,
                geometric_sink: None,
                is_in_data_table: false,
            },
        ],
    }];

    let viols = provenance_inventory::check_inventory(&files, &V21RuleConfig::default());
    assert_eq!(viols.len(), 1);
    assert_eq!(viols[0].rule_id, "V22");
    assert!(viols[0].message.contains("2/2"));
    assert!(viols[0].message.contains("100.0%"));
}

#[test]
fn n16_summary_emits_small_sample_warning_for_synthetic_fixture() {
    use crystalline_lint::contracts::file_provider::SourceFile;
    use crystalline_lint::entities::layer::Language;
    use crystalline_lint::shell::n16_summary::{collect_n16_stats, format_n16_summary};
    use std::collections::HashMap;

    let fixture_content =
        std::fs::read_to_string("tests/fixtures/n16_summary_small_sample.rs").unwrap();
    let source_files = vec![SourceFile {
        path: PathBuf::from("01_core/src/compiler/synthetic/sample.rs"),
        content: fixture_content,
        language: Language::Rust,
        layer: crystalline_lint::entities::layer::Layer::L1,
        has_adjacent_test: true,
    }];

    let stats = collect_n16_stats(&source_files, &HashMap::new());
    assert_eq!(stats.len(), 1);
    let sample_stat = stats.get("synthetic/").expect("expected synthetic/ module");
    assert_eq!(sample_stat.total(), 3);
    assert_eq!(sample_stat.alpha, 1);
    assert_eq!(sample_stat.beta, 1);
    assert_eq!(sample_stat.gamma, 1);

    let summary = format_n16_summary(&stats, 5);
    assert!(summary.contains("| `synthetic/` | 3 | 1 | 1 | 1 | 33.3% |"));
    assert!(summary.contains("⚠ amostra pequena em `synthetic/` (n=3) — percentual pouco confiável, 1 caso muda o resultado em ~33pp"));
}

#[test]
fn n16_summary_custom_min_sample_size_threshold() {
    use crystalline_lint::contracts::file_provider::SourceFile;
    use crystalline_lint::entities::layer::Language;
    use crystalline_lint::shell::n16_summary::{collect_n16_stats, format_n16_summary};
    use std::collections::HashMap;

    let fixture_content =
        std::fs::read_to_string("tests/fixtures/n16_summary_small_sample.rs").unwrap();
    let source_files = vec![SourceFile {
        path: PathBuf::from("01_core/src/compiler/synthetic/sample.rs"),
        content: fixture_content,
        language: Language::Rust,
        layer: crystalline_lint::entities::layer::Layer::L1,
        has_adjacent_test: true,
    }];

    let stats = collect_n16_stats(&source_files, &HashMap::new());
    // Com min_sample_size = 2, n=3 NÃO deve emitir aviso
    let summary_low_threshold = format_n16_summary(&stats, 2);
    assert!(!summary_low_threshold.contains("⚠ amostra pequena"));

    // Com min_sample_size = 10, n=3 DEVE emitir aviso
    let summary_high_threshold = format_n16_summary(&stats, 10);
    assert!(summary_high_threshold.contains("⚠ amostra pequena em `synthetic/` (n=3)"));
}
