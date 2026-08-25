# Relatório P0097 — auditoria da extração Rust de `SourceConstant`

**Data:** 2026-08-25
**Branch:** `codex/audit-rust-source-constant-extraction`
**Baseline:** `3a5ffbec3968230f8fda29dff329c476fa73be39`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Resultado

O lote auditou a transformação fonte Rust → `ParsedFile.constants` sem usar V21 ou V22
como oráculo. O preflight encontrou `SPEC-GAP` em nove das dez áreas candidatas; por isso,
o escopo executável foi reduzido à projeção numérica dentro de `function_item`:

- `FunctionNumberLiteral` para literais positivos e `NegativeLiteral` para negativos;
- snippet byte-exato, incluindo sinal e sufixo;
- linha e coluna 1-based, com coluna medida em bytes UTF-8;
- ordem lexical, multiplicidade preservada e ausência de deduplicação;
- exclusão de patterns, ranges, macros e numerais fora de função.

Origem de teste, return type, scaling, context var, geometric sink, data-table, citações e
todos os outros kinds ficaram opacos. Essa redução evitou inventar semântica ausente do L0
e preservou a arquitetura Tekt: L3 extrai fatos, L1 mantém o IR e decide regras, e L4
apenas coordena.

## Evidência causal

Os gates segregados foram congelados antes da correção da produção. B1 iniciou 0/3 e B2
1/3: a implementação publicava colunas zero-based e projetava numerais de contextos
excluídos; o erro sintático sem IR parcial já estava conforme. Após a correção mínima,
ambos terminaram 3/3.

| Gate | SHA-256 final | Resultado |
|---|---|---|
| `tests/rust_source_constant_identity_assessment.rs` | `cc596d876bcfbcacbf3688bd9a1aa1b875928bcb53f77abe1544af5100fd3dcb` | 3/3 PASS |
| `tests/rust_source_constant_context_assessment.rs` | `dc50ebb0b3913a108c09a0b5e2dc81d6918ac622d7ab8a4159340a1c818237dd` | 3/3 PASS |

Dois defeitos do próprio gate foram classificados e fechados: o harness provisório B2
usava `Box::leak`; e a primeira projeção exigia coleção inteira vazia, removendo kinds
históricos não numéricos. O harness passou a manter seus dados vivos lexicalmente e os
gates finais filtram somente os dois kinds numéricos autorizados.

## Mudança de produção

`03_infra/rs_parser.rs` recebeu apenas a correção causal:

- detecção de ancestrais para limitar a projeção numérica à função e excluir pattern e
  macro;
- coluna `+1` somente para os dois kinds numéricos deste lote, preservando a convenção
  histórica dos kinds opacos;
- retorno explícito para `range_expression`;
- preservação de `ItemDefinition`, strings, `FormatString`, `MatchPattern` e citações.

Nenhuma regra V21/V22, tipo L1, configuração, wiring, apresentação ou exit status foi
alterado para acomodar os gates.

## Validação

- B1: 3/3; B2: 3/3;
- V21: 9/9; V22 e teste dirigido do parser Rust: PASS;
- suíte completa: 630 testes unitários e todas as integrações/fixtures: PASS;
- V5/V6/V7/V12: nenhuma violação;
- reparador V5 dry-run: `Nothing to fix`;
- hashes L0 e identidades dos gates: PASS;
- `git diff --check`: PASS;
- adversário final: `READY WITH RESIDUAL AUDIT`.

## Auditoria residual

Variantes adicionais da gramática de macros, em particular tokens negativos dentro de
macros, não foram enumeradas na matriz explícita. Associação de citações e os campos
estruturais removidos no preflight continuam deliberadamente fora do lote. O RED
intermediário está preservado no histórico Git e no Assessment 0026, não como artefato
executável separado.

Nenhum merge ou push foi realizado.
