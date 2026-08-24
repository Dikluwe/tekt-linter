# Relatório P0086 — triagem segregada V23–V25

## Superfície

- base: `ce15824d57aa1f906b05a215ffa688258ef80153`;
- branch: `codex/audit-semantic-preservation-v23-v25`;
- classificadores: V23, V24 e V25 em L1;
- IR compartilhada: `SemanticObservationKind`, `SemanticObservation` e
  `HasSemanticObservations`;
- gate: `tests/semantic_preservation_v23_v25_assessment.rs`;
- hash gate: `9d7bbda9cd97f164785e7e8f1dea406a4d9190148396452afea36839029dd1e6`.

## Hashes normativos finais

| L0 | SHA-256 |
|---|---|
| V23 context-erasure | `a1352aaa397b1e849da5a6d9db006eace0aea127643bda53f8bfb7844e2ec65c` |
| V24 semantic-field-loss | `ffcb08aa01c6f5fafaab8ba40830929670399e94dae2a8edc7df4bb957ade518` |
| V25 decision-ownership | `e26a83fb44c923f9f07fdcf64495cd72340c0032b70cbeb17a511493066fc355` |
| IR rule-traits | `aeced5c851ac21a6214c1c4ca2cdd12e011926af9ae64898b95fcda0690ac4df` |

## Cadeia causal e segregada

| Fase | Resultado | Commit |
|---|---|---|
| contrato inicial | assessment e três L0 congelados | `4a6867c` |
| adversário/gate inicial | SPEC-GAP e fail-closed | `cf4ee3c` |
| fronteira normativa | decisão L3 separada de classificação L1 | `de1b1a6` |
| L0 causal da IR | taxonomia vinculada antes do código | `a6a23f8` |
| materialização L1 | quarta modalidade V25 e hashes | `c4069c4` |
| gate cego | 5/5 | `552c1af` |
| evidência integral | GATE-DEFECT fechado | `a811b3c` |
| adversário final | NÃO REABRIR | HEAD pré-fechamento |

## Arquitetura Tekt

- todo código tocado em L1 possui header causal e hash válido;
- L0 causal precede a materialização L1 no histórico;
- `rule_traits.rs` importa somente entidades/std;
- regras dependem de entities, nunca de infra, shell, wiring ou lab;
- L1 não realiza I/O e não reinterpreta contratos/AST;
- testes acompanham a materialização e o gate é externo/black-box.

## Matriz final

| Unidade | Gate | Resultado |
|---|---|---|
| V23 | 2 kinds, todos os níveis, evidência e ordem | PASS |
| V24 | 1 kind, todos os níveis, evidência e ordem | PASS |
| V25 | 4 kinds/modalidades, níveis, evidência e ordem | PASS |
| isolamento | 7 kinds × 3 regras; language irrelevante | PASS |
| totalidade | vazio, Unicode, strings vazias e spans extremos | PASS |
| arquitetura | causalidade, pureza e direção de dependência | PASS |

## Validação

- gate independente: 5/5;
- suíte global: PASS, 628 unitários, 83 fixtures e integrações;
- hashes dry-run: `Nothing to fix`;
- auto-lint V23–V25: zero violações no próprio repositório;
- rustfmt scoped dos quatro L1 e do gate: PASS;
- `git diff --check`: PASS;
- adversário final: `NÃO REABRIR`.

## Residual explícito

Este relatório não audita o extrator Rust, configuração semântica, agregação global,
dispatcher, CLI ou SARIF. Neutralidade, fluxo, dependência, ausência/opacidade, owners,
composição e `resolved_after` permanecem como alegações obrigatórias do lote L3 seguinte.

## Veredito

`READY WITH RESIDUAL AUDIT`. Classificadores L1 V23–V25 fechados; integração L3 pendente.
Nenhum merge, instalação ou release ocorreu.
