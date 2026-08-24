# Assessment 0015 — preservação semântica V23–V25

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-24
**Alvos:** `context_erasure`, `semantic_field_loss`, `decision_ownership`
**Baseline:** `ce15824d57aa1f906b05a215ffa688258ef80153`

## Insumos L0 autorizados

| Regra | Caminho | SHA-256 |
|---|---|---|
| V23 | `00_nucleo/prompts/rules/context-erasure.md` | `a1352aaa397b1e849da5a6d9db006eace0aea127643bda53f8bfb7844e2ec65c` |
| V24 | `00_nucleo/prompts/rules/semantic-field-loss.md` | `ffcb08aa01c6f5fafaab8ba40830929670399e94dae2a8edc7df4bb957ade518` |
| V25 | `00_nucleo/prompts/rules/decision-ownership.md` | `e26a83fb44c923f9f07fdcf64495cd72340c0032b70cbeb17a511493066fc355` |
| IR compartilhada | `00_nucleo/prompts/contracts/rule-traits.md` | `aeced5c851ac21a6214c1c4ca2cdd12e011926af9ae64898b95fcda0690ac4df` |

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

## Resultado e fechamento

A rodada inicial bloqueou por doze SPEC-GAPs: API ausente e mistura entre decisão L3 e
classificação L1. Os achados e o gate fail-closed foram congelados em `cf4ee3c`.

O saneamento `de1b1a6` separou explicitamente a extração semântica upstream do mapeamento
puro de ocorrências. Após revisão de arquitetura, o L0 causal de `rule_traits.rs` também
foi atualizado e hash-pinned em `a6a23f8`, antes da materialização L1 `c4069c4`. Essa
materialização adicionou `DirectDecisionReimplementation` como quarta modalidade V25 e
resselou os quatro headers pela ferramenta oficial.

O gate cego passou 5/5. O adversário final encontrou um GATE-DEFECT de evidência; o gate
foi ampliado, sem mudança de produção, em `a811b3c`. Seu SHA-256 final é
`9d7bbda9cd97f164785e7e8f1dea406a4d9190148396452afea36839029dd1e6`.

O adversário declarou `NÃO REABRIR` para os classificadores L1 puros: nenhum RED,
SPEC-GAP ou GATE-DEFECT residual. A causalidade L0→L1, a direção rules→entities e a
pureza de L1 foram preservadas. Neutralidade, fluxo, dependência, owners e temporalidade
continuam explicitamente fora do escopo, para auditoria L3/integração.

Veredito: `READY WITH RESIDUAL AUDIT`. Nenhum merge, instalação ou release foi executado.
