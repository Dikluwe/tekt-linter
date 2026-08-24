# Relatório P0087 — triagem segregada V16

## Superfície

- base: `2b7e19f775c66f8da5fc0fb14dc1464ec55b0bbf`;
- branch: `codex/audit-wildcard-saturation-v16`;
- L0 final: SHA-256 `19f79428f1e7c9740ae7f2466f03bc82c22a5632a2388e5b2c587a3fa2588609`;
- gate: `tests/wildcard_saturation_v16_assessment.rs`;
- hash gate: `e4657dd46d03339afe861460cf2a4cfaf3a043b56a1223cef38d497fa032e02f`.

## Cadeia segregada

| Fase | Resultado | Commit |
|---|---|---|
| contrato | alegações hash-pinned | `e8f1f3e` |
| adversário inicial | 12 SPEC-GAPs | `f011d8c` |
| saneamento L0 | API/paths/determinismo executáveis | `2602ec0` |
| gate cego | 5 PASS / 3 RED | `d63dde3` |
| correção | três REDs fechados | `df3f750` |
| adversário final | NÃO REABRIR funcional | HEAD pré-fechamento |

## Matriz final

| Unidade | Resultado |
|---|---|
| languages, vazio e sete scrutinees | PASS |
| enum candidato por braços distintos | PASS |
| catch-all, reincorporação e barreiras | PASS |
| nove BodyForm e severidades | PASS |
| evidência, ordem e multiplicidade | PASS |
| exceções, paths exatos e stale sintático | PASS |
| determinismo sob HashMap | PASS |
| Unicode, limites e campos irrelevantes | PASS |
| isolamento/regressão V17–V20 | PASS 10/10 |

## Arquitetura e validação

- L0 foi atualizado antes da correção L1;
- headers da família foram resselados oficialmente;
- V16 continua puro e depende somente de entities/std;
- suíte global: PASS, 628 unitários, 83 fixtures e integrações;
- gate V16: 8/8;
- hashes dry-run: `Nothing to fix`;
- auto-lint V16: zero violações;
- rustfmt scoped e `git diff --check`: PASS.

## Residual

Extração `DecisionExpr` no parser Rust e carregamento/configuração/wiring de
`wildcard_exceptions` não foram auditados neste lote.

## Veredito

`READY WITH RESIDUAL AUDIT`. V16 L1 fechado; integração residual. Nenhum merge ocorreu.
