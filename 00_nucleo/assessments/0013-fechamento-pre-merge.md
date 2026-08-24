# Assessment 0013 — fechamento pré-merge

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-24
**Passo:** P0084
**Base:** `75a56656a2e8cd0df4d0678eab9e78291ec34506`
**HEAD congelado:** `1b7e18f`

## Superfície

No HEAD congelado `1b7e18f`, o branch continha 207 arquivos alterados, 14.771 inserções e
568 remoções. A superfície inclui o protocolo de
materialização segregada, refinement seals, adapters de entrada, apresentação e regras
V1–V15 auditadas em lotes.

## Matriz de rastreabilidade

| Unidade | Produção/L0 | Gate final | Resultado | Fechamento |
|---|---|---|---|---|
| Materialização/refinement | contrato, entidade, infra e wiring do selo | `segregated_materialization_cli` 16/16 | PASS | `177e3b8` |
| 0001 Git refinement | `git_refinement` | 6/6 | PASS P0072 | `0bfda5f` |
| 0002 entidades/índice | `project_index` | 4/4 | PASS P0072 | `0bfda5f` |
| 0003 texto/SARIF | CLI e `path_encoding` | 4/4 | PASS P0072 | `0bfda5f` |
| 0004 registry/inventário | `crate_registry`, V22 | 7/7 | PASS P0072 | `0bfda5f` |
| 0005 config/walker | `config`, `walker` | 6/6 | PASS P0073 | `61cf043` |
| 0006 prompt I/O/hash | readers, walker, snapshot, writer | 6/6 | PASS P0074 | `69e923b` |
| 0007 V5/V6/V7 | proveniência pura | 6/6 | PASS P0075 | `265fe7d` |
| 0008 V2/V8/V10/V11 | classificadores mecânicos | 6/6 | PASS P0076 | `fec683f` |
| 0009 V1/V15 | classificadores de header | 6/6 | PASS P0077 | `30fd85e` |
| 0010 V3/V9 | fronteiras de import | 6/6 | PASS P0079 | `d0ee056` |
| 0011 V12/V13 | declarações/estado | 6/6 + gate nominal 5/5 | PASS P0080/P0081 | `905490a`, `db02fb8` |
| 0012 V4/V14 | pureza/externos | 7/7 e 9/9 | PASS P0083 | `6e6d12d` |

Alterações de headers sem mudança funcional são cobertas pelo reparo oficial de hashes e
agrupadas como linhagem, não como implementação nova de regra.

## Auditoria adversarial

O primeiro veredito foi `BLOCKED` por três causas de fechamento:

1. assessments 0001–0006 ainda declaravam estados congelados;
2. assessment 0013 e relatório P0084 ainda não existiam;
3. `git diff --check` encontrou trailing whitespace nos relatórios P0081/P0083.

Os estados foram reconciliados somente contra relatórios e gates existentes, os dois
artefatos finais foram produzidos e o whitespace removido. O adversário não demonstrou
RED funcional atual nem produção funcional sem gate correspondente.

## Validação

- `cargo test --workspace --quiet`: PASS, incluindo 628 unitários, 83 fixtures e todos os
  gates de integração;
- hashes em modo seco: `Nothing to fix`;
- auto-lint V1/V5/V7: zero violações;
- `git diff --check` contra a base: PASS após saneamento documental;
- todos os arquivos Rust novos do delta: `rustfmt --check` PASS;
- smoke Typst das regras tocadas: exit 0;
- passagem Typst sem V5/V6: exit 0, somente warnings/info históricos;
- hashes Typst em modo seco: 415 candidatos, nenhum aplicado;
- fingerprint do worktree Typst antes/depois:
  `4c565340c757b00a24a5af3145618210d0a8f2631adc94a248e046c6f76adc67`.

O check de formatação sobre todo arquivo legado tocado não é verde: a migração de hashes
inclui parsers e módulos que já divergem do rustfmt atual. Formatar todo esse conjunto
agora ampliaria materialmente o delta. Os arquivos novos estão verdes; a formatação
legada fica como residual explícito para outro branch.

## Veredito

`READY WITH RESIDUAL AUDIT`.

Não resta RED/SPEC-GAP conhecido pertencente às alegações efetivamente fechadas pelo
delta. Permanecem fora dele a auditoria das regras posteriores e a formatação global do
legado. O merge pode ser decidido em ação separada; este assessment não o executa.
