# Assessment 0039 — piloto dirigido de mutação

**Estado:** BASELINE CONGELADO — rodada pendente
**Data:** 2026-08-25
**Passo:** P0111
**Branch:** `codex/p0111-mutation-pilot`
**Baseline:** `b2a2826e540a556081476918f98cb85c5dfe21be`
**Commit do passo:** `2874bda`

## L0 hash-pinned

| Insumo | SHA-256 |
|---|---|
| lista repetida dos 192 mutantes | `f7cb4b6708c576eb40aa513064bc2ec02a8d916382ed4a43acad77db1fb987d8` |
| manifesto de hashes das campanhas históricas | `b965185bc6b075ef6bf1d8db2ec8f74072050d96ffc38162c9cbf1353c98f4cb` |
| `refinement_seal.rs` | `01bce52ec024c3ae52473b2e3339bd4f99d93beafc040d602d8e6bdd59a7e015` |
| `forbidden_import.rs` | `a6f19cd55547de8bfb3961c6741b25a1cbdb3d8d13f7d7904e9ce0574d30eff3` |
| `prompt_drift.rs` | `31fe43b1c7dafc877bb7016b3b3711c3a37dd92ff8dd8044009640cf0a93c7a6` |
| `fix_hashes.rs` | `d83fe7cf0a4f0d93dd080b6e39130791aea35743fd71f04a0baffdcfb320b6d4` |
| `refinement_snapshot.rs` | `e482661b43557c3ca979e5170e151b2cf218013eb7b160a1a89e06d3e14d02e` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

Ferramentas: `cargo-mutants 27.1.0`; `cargo-llvm-cov 0.8.7`.

## Preservação histórica

As campanhas preexistentes ocupavam 37 MiB (`mutants.out`) e 2 MiB (`mutants.out.old`).
Foram movidas byte a byte, sem remoção, para:

- `mutants.out.pre-p0111-current`;
- `mutants.out.pre-p0111-old`.

O manifesto externo contém SHA-256 de 16 arquivos. A campanha P0111 não reutiliza nem
rotaciona esses nomes.

## Baseline operacional

- lista repetida: 192/192 e mesmo SHA-256;
- worktree Git: limpo antes do Assessment;
- auto-lint instalado e do workspace: exit 0, somente V19=68/V20=17;
- hashes: `Nothing to fix`;
- suíte, fmt e ratchet: verdes no fechamento P0110.

Nenhuma produção ou teste foi editado. Próxima transição: congelar o mapa de cobertura e
executar a rodada cega.
