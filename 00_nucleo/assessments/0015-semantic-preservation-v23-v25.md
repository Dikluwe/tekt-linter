# Assessment 0015 — preservação semântica V23–V25

**Estado:** CONGELADO PARA TRIAGEM SEGREGADA
**Data:** 2026-08-24
**Alvos:** `context_erasure`, `semantic_field_loss`, `decision_ownership`
**Baseline:** `ce15824d57aa1f906b05a215ffa688258ef80153`

## Insumos L0 autorizados

| Regra | Caminho | SHA-256 |
|---|---|---|
| V23 | `00_nucleo/prompts/rules/context-erasure.md` | `a1352aaa397b1e849da5a6d9db006eace0aea127643bda53f8bfb7844e2ec65c` |
| V24 | `00_nucleo/prompts/rules/semantic-field-loss.md` | `ffcb08aa01c6f5fafaab8ba40830929670399e94dae2a8edc7df4bb957ade518` |
| V25 | `00_nucleo/prompts/rules/decision-ownership.md` | `c6ebd9250b865f655c96b5bbdd125dc4b9b84101eb6d46be34247a605199f136` |
| IR compartilhada | `00_nucleo/prompts/contracts/rule-traits.md` | `01f72010e0f5ee43ca7933cd0e8e5aba982853d25787b09c5a62c6370b81b05d` |

## Natureza e fronteira

Assessment retroativo (`assessed`, não `sealed`) dos classificadores L1 puros. Este lote
não audita carregamento TOML, extração pelos parsers, dispatcher, seleção CLI ou SARIF;
esses consumidores formam um lote posterior. Durante a triagem, produção não pode ser
alterada antes que qualquer RED/SPEC-GAP seja congelado.

O verificador recebe somente este assessment e os três L0 acima, presos pelos hashes
exatos. Não pode ler L1–L4, testes existentes, histórico ou relatórios antes de congelar
o gate. Insumo insuficiente é `SPEC-GAP`, nunca expectativa adivinhada.

## Alegações congeladas

1. **Escopo comum:** V23–V25 recebem ocorrências já classificadas por L3 e são silenciosas
   para kinds de outras regras. Nenhum classificador reinterpreta contratos, nomes,
   language, fluxo, neutralidade, dependência, owner ou temporalidade.
2. **V23:** mapeia exatamente `ContextNeutralArgument` e
   `ContextErasingProjection`, uma violação por ocorrência, em ordem.
3. **V24:** mapeia exatamente `NeutralProjectionDestination`, uma violação por
   ocorrência, em ordem.
4. **V25:** mapeia exatamente `DuplicateDecisionOwner`, `DecisionProxyReentry`,
   `CanonicalizerReentry` e `DirectDecisionReimplementation`, com as quatro modalidades
   textuais congeladas no L0, uma violação por ocorrência, em ordem.
5. **Evidência:** cada diagnóstico preserva `contract_id` e `detail` verbatim, path do
   arquivo e linha/coluna da observação; V25 preserva também a modalidade. `rule_id` é
   V23/V24/V25 e o nível é exatamente o parâmetro recebido.
6. **Isolamento:** variar language e campos de observações ignoradas não altera as
   violações elegíveis; os três classificadores não se ativam mutuamente.
7. **Ordem/multiplicidade:** L1 não ordena nem deduplica. Permutar observações permuta a
   saída correspondente; duplicar ocorrência elegível duplica diagnóstico.
8. **Totalidade:** coleção vazia, strings vazias/Unicode, linha/coluna zero ou máximas e
   todos os níveis públicos não causam panic; ocorrências elegíveis continuam observáveis.

## Gate mínimo

- matriz completa dos sete kinds × três regras;
- todos os níveis públicos, provando preservação exata do parâmetro;
- quatro modalidades V25 e campos obrigatórios da mensagem;
- ordem/cardinalidade com múltiplas observações e locations distintas;
- duplicação e permutação como sequência de ocorrências, sem deduplicação;
- mutação de language e de observações ignoradas com comparação integral;
- contract/detail Unicode/vazios, spans extremos, vazio e ausência de panic.

Cada resultado é `PASS`, `RED`, `SPEC-GAP` ou `GATE-DEFECT`. O gate não pode importar
helpers, constantes ou classificadores da produção como oráculo compartilhado.

Neutralidade, fluxo, dependência, ausência/opacidade, identidade contratual, owners
efetivos, duplicatas de entrada, composição e `resolved_after` são alegações reservadas
ao lote L3/integração. O gate P0086 não pode apresentá-las como auditadas.

## Fechamento

Antes de merge: achados congelados, saneamento normativo anterior à produção, gate
independente verde, adversário final `NÃO REABRIR`, suíte global, hashes dry-run,
auto-lint, rustfmt scoped, `git diff --check`, relatório P0086 e estado final
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Este branch não executa merge.
