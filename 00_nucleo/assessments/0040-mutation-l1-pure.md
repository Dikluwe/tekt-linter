# Assessment 0040 — mutação dirigida do núcleo L1 puro

**Estado:** BASELINE E AUTORIDADES CONGELADOS — rodada pendente
**Data:** 2026-08-26
**Passo:** P0112
**Branch:** `codex/p0112-mutation-l1-pure`
**Baseline/merge P0111:** `e3dab595dbbe25e2690a482e0840335593ef7a5d`

## Precondição e instalação

- P0111 e a especificação P0112 foram integrados em `master` por merge não fast-forward;
- branch P0112 nasceu diretamente do mesmo commit de `master`;
- `cargo install --path . --force` substituiu o binário instalado;
- SHA-256 do binário: `ec54607d4de92edacfac2e25c0ed390b3743c58cb86eb88e969ab0021833ec32`.

## L0 hash-pinned

| Insumo | SHA-256 |
|---|---|
| lista repetida dos 38 mutantes | `16d066ef9a3cda926b2db4ec04739f1115ffd8800308bc2043b2491648213f35` |
| manifesto das duas campanhas imediatamente anteriores | `92cda0db21fe88e928455cf65cabca3918c54217729c4639323dd6398539567f` |
| `l1_allowed_external.rs` | `dbbbb809e0c66f7fcbb7a349695037be381352a4d7ed743ce14ffcebd97a9bfd` |
| `alien_file.rs` | `bd7b31b9c2547205087c7c170a53f8d86a97e465aaf93d1fe0ba3580f6212368` |
| `test_file.rs` | `6072016519b73d6df8831dc4cfe7cc8f97ee94fd3116e8eda963e338fca86d1f` |
| `quarantine_leak.rs` | `b50466a01b76c9c4eebdf92fa05685d6022843266b867cc11ac736c933a8ba97` |
| `orphan_prompt.rs` | `ecba43f507a415762d7a25e8b0bca0aff23d52d632c261bf4f6ad928e52989e4` |
| `mutable_state_core.rs` | `7ec860d6197e44a0f3dd1bd8f96d97a27f831fa3888035219ae43ff17d6a2382` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

Ferramentas: `rustc 1.92.0`, `cargo 1.92.0`, `cargo-mutants 27.1.0`.

As saídas `mutants.out` e `mutants.out.old` de P0111 foram preservadas como
`mutants.out.pre-p0112-current` e `mutants.out.pre-p0112-old`. O manifesto externo contém
17 hashes. Nenhuma campanha histórica foi removida.

## Baseline operacional

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test --all-targets` | PASS — 635 unitários e toda a suíte de integração |
| ratchet P0109 | PASS 2/2 dentro da suíte |
| auto-lint | exit 0; V19=68 e V20=17 |
| hash da saída do auto-lint | `92c51980f1574d87359a810c27c29b40c1a84b5a7119bfab2690d1277ab622c8` |
| `fix-hashes --dry-run` | `Nothing to fix` |
| hash da saída do dry-run | `bf3511c5c7cb1a9202071d8a2d60204a30f29cc11ee58ff2963a063a9f835b25` |

Não existe RED, `GATE-DRIFT` ou `SPEC-GAP` no baseline. Nenhuma produção ou teste foi
editado. Próxima transição autorizada: rodada cega dos 38 mutantes.
