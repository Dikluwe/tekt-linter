# Assessment 0041 — campanha paralela de mutação L1

**Estado:** CAMPANHA FECHADA — READY FOR ACCUMULATED INTEGRATION
**Data:** 2026-08-26
**Passo:** P0113
**Branch acumulador:** `codex/p0112-mutation-l1-pure`
**Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## L0 comum

| Insumo | SHA-256 |
|---|---|
| manifesto das 16 fontes e 16 prompts owner | `2000c01ee1068daa74dd7c7b49c217ede023e52a84fafaf02e14d31769d2e99b` |
| lista S1, 32 mutantes | `fc11fde2de1888dffbcc34aed0f0e5c77061184e21136bc4db331d422fb17d31` |
| lista S2, 15 mutantes | `c2f26f19ec04cb601b7b767ca5c6420ece76fcc6376e62fee55b02233281c40d` |
| lista S3, 30 mutantes | `06bd4f529e22c14b22d8597395e36a4f2ffff2ed404a795435367f3b60a2672f` |
| lista S4, 35 mutantes | `12683903693d10a66432545f60954cf0a04b3b0ab279404b0b63590074f68cf7` |
| lista S5, 32 mutantes | `7fdb07f0b0d23d5e67aef4add5a5222fc599fc9476d8a155f864eb30afabe3ce` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |
| binário instalado | `ec54607d4de92edacfac2e25c0ed390b3743c58cb86eb88e969ab0021833ec32` |
| manifesto da campanha P0112 preservada | `e4d752ba789a45751279b005bdc620d3483c0b209a5d7ead8d397ea8816f6495` |

Ferramentas: `rustc 1.92.0`, `cargo 1.92.0`, `cargo-mutants 27.1.0`.
As listas repetidas contêm exatamente 32/15/30/35/32 linhas, total 144.

## Baseline comum

| Gate | Resultado |
|---|---|
| worktree | limpo antes do Assessment |
| `cargo fmt --check` | PASS |
| `cargo test --all-targets` | PASS — hash da saída `86d08683905b0360c5dd44fad6d04b59e0b2220b515271b8be3122cb02c55ae7` |
| ratchet P0109 | PASS 2/2 dentro da suíte |
| auto-lint | exit 0; V19=68/V20=17; hash `92c51980f1574d87359a810c27c29b40c1a84b5a7119bfab2690d1277ab622c8` |
| `fix-hashes --dry-run` | `Nothing to fix`; hash `bf3511c5c7cb1a9202071d8a2d60204a30f29cc11ee58ff2963a063a9f835b25` |

`mutants.out` de P0112 foi movido sem remoção para
`mutants.out.pre-p0113-current`. Nenhuma produção foi editada. A única alteração funcional
da campanha é um teste normativo que fecha a identidade de cada item em imports agrupados.
Não existe `PRODUCTION-RED`, `SPEC-GAP` ou `ARCH-RED` residual.

## Quadro de shards

| Shard | Estado | Resultado | Assessment próprio |
|---|---|---|---|
| S1 | FECHADO | 29 CAUGHT, 3 UNVIABLE, 0 MISSED | `0041-s1-*` |
| S2 | FECHADO | 11 CAUGHT, 4 UNVIABLE, 0 MISSED | `0041-s2-*` |
| S3 | FECHADO | 28 CAUGHT, 2 MISSED iniciais; 1 TEST-GAP fechado e 1 FLAKY-GATE reproduzido como CAUGHT | `0041-s3-*` |
| S4 | FECHADO | 34 CAUGHT, 1 UNVIABLE, 0 MISSED | `0041-s4-*` |
| S5 | FECHADO | 28 CAUGHT, 4 UNVIABLE, 0 MISSED | `0041-s5-*` |

## Consolidação

O passe inicial executou 144 mutantes: 130 `CAUGHT`, 12 `UNVIABLE/TOOL-LIMIT` e 2
`MISSED`, ambos no S3. A reprodução serial segregou os dois sobreviventes:

- a troca `+ -> *` em `external_type_in_contract::imported_items` confirmou um
  `TEST-GAP`; `root_grouped_import_preserves_each_authorized_item_identity` foi adicionado
  e matou o cluster reproduzido 2/2;
- a troca `- -> +` não sobreviveu à reprodução serial e ficou classificada como
  `FLAKY-GATE/TOOL-LIMIT`, sem evidência de lacuna semântica.

Assim, não há mutante acionável sobrevivente. Os 12 inviáveis foram rejeitados pelo
compilador, predominantemente por tentativas da ferramenta de construir
`Violation::default()` quando `Violation` não implementa `Default`; não justificam mudar a
API de produção.

## Defeitos de gate observados e saneados

1. A primeira onda paralela mostrou colisão entre fixtures no `/tmp` global. Isolar
   worktree, target e output não basta: execuções paralelas também precisam de `TMPDIR`
   próprio. S1 foi repetido integralmente após a segregação.
2. S2 precisou recriar quatro diretórios vazios de fixture que Git não transporta entre
   worktrees. Isso é fragilidade do fixture, não resultado de mutação.
3. S4 reteve cache do último mutante no target segregado. Apenas esse target foi limpo e
   todos os gates foram repetidos a partir da fonte restaurada.

Esses eventos são `GATE-DEFECT` fechados. Nenhum foi reinterpretado como sucesso do
linter.

## Veredito

P0113 amplia a sanitização sem alterar o comportamento de produção. A campanha permanece
fora de `master`, de acordo com a decisão de acumular uma massa maior de testes de mutação
antes de pagar o custo de integração. O binário instalado continua semanticamente válido;
o delta atual é exclusivamente documental e de teste.

## Gate acumulado pós-composição

| Gate | Resultado |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --all-targets` | PASS; 635 testes unitários e todas as integrações |
| ratchet P0109 | PASS 2/2 dentro da suíte |
| auto-lint | exit 0; saída idêntica ao baseline, SHA-256 `92c51980f1574d87359a810c27c29b40c1a84b5a7119bfab2690d1277ab622c8` |
| `fix-hashes --dry-run` | `Nothing to fix`; SHA-256 `bf3511c5c7cb1a9202071d8a2d60204a30f29cc11ee58ff2963a063a9f835b25` |
| `git diff --check` | PASS |
