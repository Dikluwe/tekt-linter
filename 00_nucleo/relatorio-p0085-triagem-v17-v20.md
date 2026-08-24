# Relatório P0085 — triagem segregada V17–V20

## Superfície congelada

- base: `cea1e7013b8c6e6f61dbcef69a4d52a392e628de`;
- branch: `codex/audit-low-risk-v17-v20`;
- L0 final: `00_nucleo/prompts/rules/wildcard-saturation.md`;
- SHA-256 L0: `5941adf0c444a65e101224dacfdb1fea0cbafebf46a5a9ac6be5bed25063cc08`;
- produção funcional: `range_pattern.rs` e `deep_pattern_nesting.rs`;
- linhagem resselada: família V16–V20;
- gate: `tests/decision_metrics_v17_v20_assessment.rs`;
- SHA-256 gate: `40472d68e557cea37819898298f5b578da7f7bbb6b672bc0271c2d46ab830849`.

## Cadeia segregada

| Fase | Produtor | Resultado | Commit |
|---|---|---|---|
| contrato | orquestrador | alegações e L0 hash-pinned | `b51d301` |
| gate inicial | `verifier/v17-v20/0014` | SPEC-GAP de API | `b169d16` |
| adversário inicial | `adversary/v17-v20/0014` | SPEC-GAP V18/V20 | `b169d16` |
| saneamento normativo | orquestrador | gaps decidíveis em L0 | `3cd77dd` |
| reteste cego | `verifier/v17-v20/0014-retest` | 4 PASS / 2 RED | `2990819` |
| correção | orquestrador | V18 substring; V20 catch-all | `fbf234a` |
| ampliação do gate | verificador cego | fecha GATE-DEFECTs | `4588997`, `50f1af4` |
| adversário final | `adversary/v17-v20/0014-final` | NÃO REABRIR | HEAD pré-fechamento |

Identidades registram segregação operacional, não alegação de sandbox formal.

## Matriz final

| Unidade | Gate | Resultado |
|---|---|---|
| linguagem/vazio | todas as nove variantes não-Rust × V17–V20 | PASS |
| V17 | tabela-verdade, evidência, ordem e isolamento | PASS |
| V18 | componente/stem, substring, case, separadores, evidência | PASS |
| V19 | limites 0/1/2/máximo, contagem, ordem e isolamento | PASS |
| V20 | limites, tuple scrutinee, tabela/catch-all/quase-tabela | PASS |
| transversal | 2 expressões × 2 braços; layer e campos irrelevantes | PASS |
| regressão V16 | testes direcionados | PASS 5/5 |

## Validação

- gate independente: 10/10;
- `cargo test --workspace --quiet`: PASS, 628 unitários, 83 fixtures e integrações;
- `cargo run --quiet -- . --fix-hashes --dry-run`: `Nothing to fix`;
- auto-lint V16–V20: execução íntegra; somente métricas V19/V20 esperadas;
- `rustfmt --check` nos arquivos funcionalmente alterados e no gate: PASS;
- `git diff --check` contra a base: PASS;
- worktree limpo após o commit de fechamento: critério a confirmar no próprio commit.

O `cargo fmt --all -- --check` continua falhando em arquivos legados fora do delta. Essa
dívida global já era residual explícito do assessment 0013; não foi mascarada nem
formatada neste lote.

## Veredito

`READY WITH RESIDUAL AUDIT`.

V17–V20 estão fechadas dentro das alegações do assessment 0014. Não resta RED,
SPEC-GAP ou GATE-DEFECT do lote. Componentes posteriores ainda não auditados e o rustfmt
global legado permanecem fora do escopo. Nenhum merge, instalação ou release ocorreu.
