# Adversário final — Assessment 0018 / P0089

**Identidade:** D / adversário final
**Baseline:** `cc1924b52f2079034c7814241ade88e0ca8f7583`
**Confronto:** `cc1924b..ec43f05`
**Data:** 2026-08-24
**Veredito:** `READY WITH RESIDUAL AUDIT`

## Escopo e cadeia causal

Os oito insumos autorizados do Assessment 0018 foram recalculados com `sha256sum`; os
oito SHA-256 coincidem exatamente com a tabela resselada. O gate em
`tests/infrastructure_error_projection_assessment.rs` também coincide byte a byte com
a versão congelada no commit `b3c6d1c` (SHA-256
`7cf459d6251533df903019795b036ff43345731f14d9b9eb636e6773b5bbb7db`). Portanto não há
mutação posterior do oráculo.

A decisão causal é coerente: `prompts/rules/infrastructure-error.md` coloca a política
em L1; `01_core/rules/infrastructure_error.rs` materializa somente transformação pura;
`04_wiring/main.rs` importa e encaminha os erros aos projetores. Não restaram corpos,
IDs, severidades, mensagens, posições ou ownership duplicados em L4. Busca global
encontrou as mensagens normativas apenas no módulo L1 e no gate.

## Matriz adversarial

| Alegação | Resultado | Evidência |
|---|---|---|
| `Unreadable` → uma V0 `Fatal`, path owned, `0:0`, razão preservada | `PASS` | implementação e gate congelado concordam, inclusive razão vazia/hostil |
| `SyntaxError` → uma PARSE `Error`, posição e mensagem preservadas | `PASS` | posição não zero, Unicode, controles e mensagem vazia cobertos |
| `UnsupportedLanguage` → PARSE `Warning`, path owned, `0:0` | `PASS` | linguagem interpolada por `Debug`, exatamente como L0 |
| `EmptySource` → PARSE `Warning`, path owned, `0:0` | `PASS` | texto normativo e cardinalidade preservados |
| Derives de `SourceError` | `PASS` | `Debug, Clone, PartialEq, Eq`, conforme L0 e necessário para teste de imutabilidade |
| Pureza L1 | `PASS` | módulo importa apenas `Cow`, contratos e entidades L1; sem fs/config/env/time/network/process |
| L4 zero política/duplicação | `PASS` | conversores locais removidos; chamadas somente nas linhas de encaminhamento do pipeline |
| Gate independente | `PASS` | gate congelado antes da correção, usa API pública e enums, sem parser/walker/CLI |
| Hashes oficiais | `PASS` | `cargo run --quiet -- . --fix-hashes --dry-run` informou `Nothing to fix` |
| 13 arquivos de header | `PASS` | diff individual mostra somente `@prompt-hash`; nenhum delta funcional escondido |
| Regressão 0001–0017 | `PASS` | `cargo test --workspace --quiet`: 628 unitários e todos os binários de integração, incluindo 83 fixtures, passaram |
| Formatação funcional do lote | `PASS` | `rustfmt --check` nos quatro arquivos funcionais/gate tocados passou |
| Higiene do delta | `PASS` | `git diff --check cc1924b..ec43f05` passou sem saída após `ec43f05` |

## Reabertura e fechamento do RED

O primeiro confronto encontrou quatro linhas documentais com whitespace final:

- `00_nucleo/assessments/0018-v0-parse-projections.md`: linhas 3, 4 e 5;
- `00_nucleo/prompts/rules/infrastructure-error.md`: linha 5.

O commit `ec43f05` removeu somente esses espaços, resselou no Assessment o novo SHA-256
do L0 (`bb2f8f5669ca264b205d03240a06d53c576b57847450388f44663f4f89cab119`) e atualizou
somente o `@prompt-hash` causal no módulo L1. O confronto reaberto confirmou:

- `git diff --check cc1924b..ec43f05`: PASS;
- oito hashes L0: coincidência exata com o Assessment resselado;
- gate: ainda byte a byte idêntico ao congelado em `b3c6d1c`;
- gate isolado: 7/7 PASS;
- `cargo run --quiet -- . --fix-hashes --dry-run`: `Nothing to fix`;
- nenhuma alteração de corpo funcional entre `f1f8486` e `ec43f05`.

O RED está fechado.

## GATE-DEFECT e SPEC-GAP

- `GATE-DEFECT`: nenhum aberto. O gate permaneceu byte a byte congelado e observa a
  API pública sem compartilhar implementação.
- `SPEC-GAP`: nenhum aberto no comportamento auditado. Camada, API, quatro mensagens,
  severidades, ownership e semântica de `0:0` estão decididos nos L0 hash-pinned.

## Conclusão

O delta satisfaz a arquitetura Tekt, as quatro projeções, a segregação do gate e todas
as validações de fechamento. Não há `RED`, `GATE-DEFECT` ou `SPEC-GAP` aberto no escopo.
O P0089 está `READY WITH RESIDUAL AUDIT`; o residual é apenas a auditoria futura dos
componentes fora do lote, sem bloquear seu fechamento.
