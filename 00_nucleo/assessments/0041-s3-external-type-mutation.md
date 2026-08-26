# Assessment 0041-S3 — mutação de tipos externos em contratos

> **Estado:** SHARD FOUND TEST GAPS — CLOSED
> **Passo:** P0113, shard S3
> **Branch/worktree:** `codex/p0113-s3` / `/tmp/p0113-s3`
> **Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## Escopo L0 hash-pinned

- alvo único: `01_core/rules/external_type_in_contract.rs`;
- SHA-256 da fonte: `cef2e1e0873afa21d2c37ae8f23dbfa076086273dfb21ccbae5d5f79b95c0c43`;
- owner: `00_nucleo/prompts/rules/external-type-in-contract.md`;
- SHA-256 do owner: `1129acfbea325c00986273b54afd6c9c272e29803131b249bb094d8e69d10ea3`;
- SHA-256 de `Cargo.lock`: `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb`;
- `cargo-mutants 27.1.0`;
- lista: `/tmp/p0113-s3-list.txt`, 30 linhas, SHA-256
  `06bd4f529e22c14b22d8597395e36a4f2ffff2ed404a795435367f3b60a2672f`;
- binário instalado: `/home/dikluwe/.cargo/bin/crystalline-lint`, SHA-256
  `ec54607d4de92edacfac2e25c0ed390b3743c58cb86eb88e969ab0021833ec32`.

O mapa de autoridade foi materializado em `0041-s3-authority-map.tsv` antes da campanha.
Prompt owner e arquitetura Tekt prevalecem sobre testes históricos; o shard não autoriza
movimento de política para fora de L1.

## Baseline dirigido

- worktree inicialmente limpo: PASS;
- enumeração exata/hash: PASS 30/30;
- `cargo test --test normative_v14_blind_assessment`: PASS 9/9;
- `cargo fmt --check`: PASS;
- `git diff --check`: PASS.

## Campanha e classificação

Comando integral:

```text
cargo mutants -j 2 --no-shuffle --no-times --output /tmp/p0113-output-s3 \
  --file 01_core/rules/external_type_in_contract.rs
```

Resultado cego: **30 testados = 28 CAUGHT + 2 MISSED**, sem timeout ou inviável. A
execução integral está em `/tmp/p0113-output-s3/mutants.out.old`:

| artefato | SHA-256 |
|---|---|
| `mutants.json` | `00f16f86d1447f85391af0666355e9cf3d5a82ae4b207ad826707049b128ecb3` |
| `outcomes.json` | `c4d740006509b411f1601c56e20455fcdd1a01f2a98888802e7f0142422c1b23` |
| `caught.txt` | `e0dc6a084e46c039c31b362e58fd9fc6bd8010bafd793e0f0ae44a865b009d2b` |
| `missed.txt` | `b449b16427ec0f8a7cc092e871a1a3556682506169d7ecac13994cce5489bfa1` |
| `timeout.txt` / `unviable.txt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Os dois `MISSED` foram reproduzidos com `--iterate -j1`. A troca `+ -> *` em
`imported_items` permaneceu `MISSED`; a troca `- -> +` foi `CAUGHT`, portanto a morte
paralela divergente é registrada como `FLAKY-GATE` e classificada operacionalmente como
`TOOL-LIMIT`, sem mudança semântica no produto. A reprodução serial tem hashes
`mutants.json=2154beca...`, `outcomes.json=80580ef3...`, `caught.txt=397b3f8d...` e
`missed.txt=9d4d8bab...`.

O sobrevivente estável é `TEST-GAP`, não `PRODUCTION-RED`: num grupo de raiz totalmente
autorizado, a mutação incorpora `{` à identidade do primeiro item e gera falso V14. O
teste histórico continha também um item proibido e, assim, esperava violação em ambos os
programas. Foi adicionado o gate público positivo
`root_grouped_import_preserves_each_authorized_item_identity`, sem alteração da produção.

## Reteste e fechamento

A repetição serial dirigida às duas mutações da posição `127:42` terminou **2 CAUGHT / 0
MISSED**. Artefatos em `/tmp/p0113-output-s3-final/mutants.out`:

- `mutants.json`: `d1dc390b8c1014e8793a96ac05999acf15ad12aebb0494d3785d4e5958b33bfe`;
- `outcomes.json`: `2bba134669266a0d1db328a831da1076e1703ed8a549e6940da93bc5449ac98b`;
- `caught.txt`: `20814143c01ed7ab7eb1163839388b0620617a1cdf5fd40eb264a9a39587592d`;
- listas vazias: SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Classificação integral dos não mortos: 1 `TEST-GAP` saneado e 1 `TOOL-LIMIT` por
`FLAKY-GATE`; 0 `PRODUCTION-RED`, `SPEC-GAP` ou `ARCH-RED`. A arquitetura Tekt permanece
inalterada: política em L1, gate público pela função `check`, sem seam ou exposição de
helper privado.

Gates dirigidos finais:

- `cargo fmt --check`: PASS;
- `cargo test --test normative_v14_blind_assessment`: PASS 10/10;
- `cargo test external_type_in_contract`: PASS 15/15 unitários, 0 falhas;
- `crystalline-lint --fix-hashes --dry-run .`: PASS, `Nothing to fix`;
- `git diff --check`: PASS.

O fechamento não altera código de produção. O commit do shard fica para a composição,
pois a autorização de escrita no metadata Git não foi concedida durante esta execução.
