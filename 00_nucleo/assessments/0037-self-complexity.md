# Assessment 0037 — complexidade histórica do próprio linter

**Estado:** BASELINE E CLASSIFICAÇÃO CONGELADOS — saneamento pendente  
**Data:** 2026-08-25  
**Passo:** P0109  
**Branch:** `codex/p0109-self-complexity`  
**Baseline de produção:** `507fc519236363b12c0379250fddb4ebda18b50e`  
**Commit do passo:** `a6639cc1b23896232f3f1365bec6f90d3927b2b4`

## L0 hash-pinned

| Insumo | SHA-256 |
|---|---|
| saída integral do auto-lint | `0cae776bab3c3ec11a13d37fc59a7232a8fed1c9dd623b5c95e5ae38084d5cd0` |
| manifesto 0037 | `bd03d01ad2905de088f101c3f66f987ebf369abd300a705dbfe2a16658a1d27e` |
| ADR-0016 | `abdf38e2c75b7f3a113a50db61913320d08f510e5c7f377efde020af1c19ffd2` |
| prompt V16 | `2414fcef861fc426d0fc25555eb00369b5be670ddab911b49723acaae5b450de` |
| prompt V17 | `d5ef806723eea38137c8c71ace80057cd7c8e79aa7d4ef7696fa3b72b9ea1a98` |
| prompt V19 | `91c409539ea603c2e4ae1aa4932e6bedddb209991652b4a345dbbe7e3b159620` |
| prompt V20 | `b4a4acabc362920561bec12f95ddcb99a0fcf68c5d53759b9ff3e6ab1d5060e5` |

Baseline validado com worktree limpo, `cargo fmt --check`, 630 testes unitários e toda a
suíte de integração/doc-tests. `fix-hashes --dry-run` respondeu `Nothing to fix`.

## Superfície fechada

O manifesto contém exatamente 90 ocorrências em 23 arquivos: V16=3, V17=2, V19=68 e
V20=17. V18 e V21–V25 estão em zero. V2 e V3 continuam visíveis na execução completa,
mas não pertencem ao P0109.

## Classificação B

- cinco ocorrências acionáveis: três V16 e duas V17, inicialmente `REFACTOR`;
- 68 V19: `ACCEPT-EQUIVALENT`, pois as alternativas observadas entram no mesmo braço e
  compartilham resultado, evidência e efeito;
- 17 V20: `ACCEPT-BOUNDARY`, pois os padrões tornam explícito o estado fechado consumido
  por parsers, Git, shell ou snapshots sem converter erro em ausência.

A classificação V19/V20 não afirma que padrões condensados sejam universalmente bons.
Afirma somente que separar estes braços, sem diferença de decisão, duplicaria lógica e
criaria pontos de evolução divergente. O ratchet deverá identificar as evidências por
regra, path e forma estrutural normalizada, sem depender de linha.

## Hipóteses adversariais C

| ID | Hipótese | Gate necessário |
|---|---|---|
| R1 | explicitar V16 altera a cobertura de `CitationKind` | tabela completa das três variantes e suíte V21 |
| R2 | decompor V17 altera short-circuit ou número de acessos ao filesystem | equivalência booleana e testes Git existentes |
| R3 | aceitação V19 esconde decisões distintas | mesmo corpo, resultado, evidência e efeitos para todas as alternativas |
| R4 | aceitação V20 transforma erro/UNKNOWN em ausência | inspeção das classes e testes de fronteira existentes |
| R5 | ratchet baseado em linha fica frágil após rustfmt | mutação de whitespace/linha não altera identidade canônica |
| R6 | resselo toca owner não alterado | dry-run congelado antes do write |

Nenhuma correção funcional começa antes deste registro. RED, `RULE-RED` e SPEC-GAP são
bloqueantes e devem ser congelados antes de qualquer mudança de contrato.

## Próxima transição

Sanear D1 em commit isolado, reexecutar os testes direcionados e então construir o ratchet
sobre o baseline residual efetivamente observado, não sobre esta contagem presumida.
