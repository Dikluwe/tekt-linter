### Estrutura do Projeto

```text
.
├── 00_nucleo
│  ├── 0051-verificar_v3_cross_crate.md
│  ├── 0052-linter_classificacao_ciente_deps.md
│  ├── 0054-corpo_de_fixtures.md
│  ├── 0055-fechar_extracao_v6_v12_v2.md
│  ├── 0056-reconciliar_sobreviventes.md
│  ├── 0057-mutacao_caminho_veredito.md
│  ├── 0058-oraculo_diferencial.md
│  ├── 0059-consertar_cego_1_3.md
│  ├── 0060-consertar_cego_2.md
│  ├── 0061-excluir_teste_gravidade.md
│  ├── 0062-versao_linter.md
│  ├── adr
│  │  ├── 0001-tree-sitter-intermediate-repr.md
│  │  ├── 0002-typed-extensions-for-parsed-file.md
│  │  ├── 0003-code-to-prompt-feedback-direction.md
│  │  ├── 0004-reformulação-do-motor-de-análise.md
│  │  ├── 0005-location-owned-paths-e-cargo.toml-como-artefato-gerido.md
│  │  ├── 0006-fechamento-topológico-e-proteção-de-encapsulamento.md
│  │  ├── 0007-fechamento-comportamental-lab-contratos-fiacao.md
│  │  ├── 0008-estrategia-de-distribuicao.md
│  │  ├── '0009-isolamento- de-parsers-por-linguagem.md'
│  │  ├── 0010-exclusao-ficheiros-individuais.md
│  │  ├── 0011-mutable-state-in-core.md
│  │  ├── 0012-external-type-in-contract.md
│  │  ├── 0013-import-vs-module-decl.md
│  │  ├── 0014-v11-configurable-level.md
│  │  └── ADR-0015-detecção-de-blanket-impls-para-V11.md
│  └── prompts
│     ├── architecture-standards.md
│     ├── cargo.md
│     ├── consertar-cego-1-3.md
│     ├── consertar-cego-2.md
│     ├── contracts
│     │  ├── file-provider.md
│     │  ├── language-parser.md
│     │  ├── parse-error.md
│     │  ├── prompt-provider.md
│     │  ├── prompt-reader.md
│     │  ├── prompt-snapshot-reader.md
│     │  └── rule-traits.md
│     ├── corpo-de-fixtures.md
│     ├── crate-registry.md
│     ├── excluir-teste-gravidade.md
│     ├── fechar-extracao-v6-v12-v2.md
│     ├── file-walker.md
│     ├── fix-hashes.md
│     ├── linter-core.md
│     ├── mutacao-caminho-veredito.md
│     ├── oraculo-diferencial.md
│     ├── parsers
│     │  ├── _template.md
│     │  ├── c.md
│     │  ├── cpp.md
│     │  ├── python.md
│     │  ├── rust.md
│     │  ├── typescript.md
│     │  └── zig.md
│     ├── project-index.md
│     ├── prompt-walker.md
│     ├── readme_prompt.md
│     ├── reconciliar-sobreviventes.md
│     ├── rules
│     │  ├── alien-file.md
│     │  ├── dangling-contract.md
│     │  ├── external-type-in-contract.md
│     │  ├── forbidden-import.md
│     │  ├── impure-core.md
│     │  ├── mutable-state-core.md
│     │  ├── orphan-prompt.md
│     │  ├── prompt-drift.md
│     │  ├── prompt-header.md
│     │  ├── prompt-stale.md
│     │  ├── pub-leak.md
│     │  ├── quarantine-leak.md
│     │  ├── test-file.md
│     │  └── wiring-logic-leak.md
│     ├── sarif-formatter.md
│     ├── versao-linter.md
│     └── violation-types.md
├── 01_core
│  ├── contracts
│  │  ├── file_provider.rs
│  │  ├── language_parser.rs
│  │  ├── mod.rs
│  │  ├── parse_error.rs
│  │  ├── prompt_provider.rs
│  │  ├── prompt_reader.rs
│  │  └── prompt_snapshot_reader.rs
│  ├── entities
│  │  ├── l1_allowed_external.rs
│  │  ├── layer.rs
│  │  ├── mod.rs
│  │  ├── parsed_file.rs
│  │  ├── project_index.rs
│  │  ├── rule_traits.rs
│  │  └── violation.rs
│  └── rules
│     ├── alien_file.rs
│     ├── dangling_contract.rs
│     ├── external_type_in_contract.rs
│     ├── forbidden_import.rs
│     ├── impure_core.rs
│     ├── mod.rs
│     ├── mutable_state_core.rs
│     ├── orphan_prompt.rs
│     ├── prompt_drift.rs
│     ├── prompt_header.rs
│     ├── prompt_stale.rs
│     ├── pub_leak.rs
│     ├── quarantine_leak.rs
│     ├── test_file.rs
│     └── wiring_logic_leak.rs
├── 02_shell
│  ├── cli.rs
│  ├── fix_hashes.rs
│  ├── mod.rs
│  └── update_snapshot.rs
├── 03_infra
│  ├── c_parser.rs
│  ├── config.rs
│  ├── cpp_parser.rs
│  ├── crate_registry.rs
│  ├── hash_writer.rs
│  ├── mod.rs
│  ├── prompt_reader.rs
│  ├── prompt_snapshot_reader.rs
│  ├── prompt_walker.rs
│  ├── py_parser.rs
│  ├── rs_parser.rs
│  ├── snapshot_writer.rs
│  ├── ts_parser.rs
│  ├── walker.rs
│  └── zig_parser.rs
├── 04_wiring
│  └── main.rs
├── Cargo.lock
├── Cargo.toml
├── CHANGELOG.md
├── CLAUDE.md
├── crystalline.toml
├── estrutura.md
├── lib.rs
├── oraculo
│  ├── biteproof
│  │  ├── a
│  │  │  ├── Cargo.toml
│  │  │  └── src
│  │  │     └── lib.rs
│  │  ├── b
│  │  │  ├── Cargo.toml
│  │  │  └── src
│  │  │     └── lib.rs
│  │  ├── Cargo.lock
│  │  └── Cargo.toml
│  ├── biteproof_pathref
│  │  ├── a
│  │  │  ├── Cargo.toml
│  │  │  └── src
│  │  │     └── lib.rs
│  │  ├── b
│  │  │  ├── Cargo.toml
│  │  │  └── src
│  │  │     └── lib.rs
│  │  └── Cargo.toml
│  ├── Cargo.toml
│  └── src
│     └── main.rs
├── README.md
├── tests
│  ├── fixtures
│  │  ├── _single_crate.toml
│  │  ├── v01_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v01_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v01b_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v02_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v02_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v02b_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v02c_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v02d_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v03_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── cli
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── main.rs
│  │  │  ├── crystalline.toml
│  │  │  └── wiring
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── cli
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── main.rs
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  └── crystalline.toml
│  │  ├── v03c_fail_alias
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  ├── shell
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  └── wiremod
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03d_fail_rename
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── a
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── b
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── Cargo.toml
│  │  │  └── crystalline.toml
│  │  ├── v03e_fail_pathref_expr
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  ├── shell
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  └── wiremod
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03f_fail_pathref_type
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  ├── shell
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  └── wiremod
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03g_fail_pathref_attr
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  ├── shell
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  └── wiremod
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03h_pass_pathref_local
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  └── shell
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v03i_pass_pathref_std
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crystalline.toml
│  │  │  └── shell
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v04_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v04_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v04b_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v05_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v05_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v05b_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v06_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v06_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v06b_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v06c_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v06d_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v07_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     ├── core.md
│  │  │  │     └── orphan.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v07_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v08_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── crystalline.toml
│  │  │  └── weird
│  │  │     └── alien.rs
│  │  ├── v08_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v09_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── shell
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v09_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── shell
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── v09b_fail_intra
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 02_shell
│  │  │  │  └── s.rs
│  │  │  └── crystalline.toml
│  │  ├── v10_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 03_infra
│  │  │  │  └── infra.rs
│  │  │  └── crystalline.toml
│  │  ├── v10_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 03_infra
│  │  │  │  └── infra.rs
│  │  │  └── crystalline.toml
│  │  ├── v11_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── contracts
│  │  │  │     └── contract.rs
│  │  │  └── crystalline.toml
│  │  ├── v11_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── contracts
│  │  │  │     └── contract.rs
│  │  │  ├── 02_shell
│  │  │  │  └── imp.rs
│  │  │  └── crystalline.toml
│  │  ├── v12_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 04_wiring
│  │  │  │  └── wire.rs
│  │  │  └── crystalline.toml
│  │  ├── v12_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 04_wiring
│  │  │  │  └── wire.rs
│  │  │  └── crystalline.toml
│  │  ├── v12b_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 04_wiring
│  │  │  │  └── wire.rs
│  │  │  └── crystalline.toml
│  │  ├── v12b_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 04_wiring
│  │  │  │  └── wire.rs
│  │  │  └── crystalline.toml
│  │  ├── v12c_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 04_wiring
│  │  │  │  └── wire.rs
│  │  │  └── crystalline.toml
│  │  ├── v13_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v13_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── v14_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crates
│  │  │  │  ├── corehelper
│  │  │  │  │  ├── Cargo.toml
│  │  │  │  │  └── src
│  │  │  │  │     └── lib.rs
│  │  │  │  └── corelib
│  │  │  │     ├── Cargo.toml
│  │  │  │     └── src
│  │  │  │        └── lib.rs
│  │  │  └── crystalline.toml
│  │  ├── v14_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── crates
│  │  │  │  ├── corehelper
│  │  │  │  │  ├── Cargo.toml
│  │  │  │  │  └── src
│  │  │  │  │     └── lib.rs
│  │  │  │  └── corelib
│  │  │  │     ├── Cargo.toml
│  │  │  │     └── src
│  │  │  │        └── lib.rs
│  │  │  └── crystalline.toml
│  │  ├── v14b_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 01_core
│  │  │  │  └── core.rs
│  │  │  └── crystalline.toml
│  │  ├── vexcl_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  ├── crystalline.toml
│  │  │  └── junk
│  │  │     └── bad.rs
│  │  ├── vl0_pass
│  │  │  ├── 00_nucleo
│  │  │  │  ├── note.rs
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  └── crystalline.toml
│  │  ├── vmod_l4_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── 02_shell
│  │  │  │  └── s.rs
│  │  │  └── crystalline.toml
│  │  ├── vnest_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  ├── 01_core
│  │  │  │  └── a
│  │  │  │     └── b
│  │  │  │        └── c
│  │  │  │           └── deep.rs
│  │  │  └── crystalline.toml
│  │  ├── vtest_default_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── infra
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── vtest_on_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── infra
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── vtest_pathref_default_pass
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── infra
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  ├── vtest_pathref_on_fail
│  │  │  ├── 00_nucleo
│  │  │  │  └── prompts
│  │  │  │     └── core.md
│  │  │  ├── Cargo.toml
│  │  │  ├── corelib
│  │  │  │  ├── Cargo.toml
│  │  │  │  └── src
│  │  │  │     └── lib.rs
│  │  │  ├── crystalline.toml
│  │  │  └── infra
│  │  │     ├── Cargo.toml
│  │  │     └── src
│  │  │        └── lib.rs
│  │  └── vtest_prod_fail
│  │     ├── 00_nucleo
│  │     │  └── prompts
│  │     │     └── core.md
│  │     ├── Cargo.toml
│  │     ├── corelib
│  │     │  ├── Cargo.toml
│  │     │  └── src
│  │     │     └── lib.rs
│  │     ├── crystalline.toml
│  │     └── infra
│  │        ├── Cargo.toml
│  │        └── src
│  │           └── lib.rs
│  └── fixtures.rs
├── tmp
│  └── ast_check.rs
└── USAGE.md
```
