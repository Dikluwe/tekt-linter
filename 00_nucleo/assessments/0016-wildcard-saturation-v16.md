# Assessment 0016 — WildcardSaturation V16

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-24
**Alvo:** `01_core/rules/wildcard_saturation.rs`
**Baseline:** `2b7e19f775c66f8da5fc0fb14dc1464ec55b0bbf`
**L0 autorizado:** `00_nucleo/prompts/rules/wildcard-saturation.md`
**SHA-256 L0:** `19f79428f1e7c9740ae7f2466f03bc82c22a5632a2388e5b2c587a3fa2588609`

## Escopo

Assessment retroativo do classificador L1 V16. O parser que produz `DecisionExpr`, a
leitura TOML das exceções, CLI e wiring ficam fora deste lote. V17–V20 estão fechadas no
assessment 0014 e entram somente como regressão.

O verificador lê somente este assessment e o L0 hash-pinned. O L0 já publica a API
black-box de `HasDecisionArms`, IR, violações e módulo. Produção, testes, histórico e
relatórios permanecem proibidos até o gate ser congelado.

## Alegações congeladas

1. V16 é silenciosa fora de Rust e em coleção vazia.
2. Scrutinee `MethodCall`, `Index` ou `Literal` isenta a expressão inteira; demais formas
   continuam elegíveis.
3. Enum candidato exige pelo menos dois braços distintos com um prefixo qualificado
   comum. Prefixos duplicados no mesmo braço ou prefixos incompatíveis não bastam.
4. Somente catch-all não reincorporado é elegível; `bound_ident_used_in_body`,
   `ErrorBarrier` e `MessageProducer` isentam.
5. Corpos elegíveis mapeiam exatamente: EnumPath/LiteralOther Warning deny-class;
   LiteralNeutral Warning; Call Info; EmptyBlock/Continue Warning; Other Warning.
6. Diagnóstico principal nunca é silenciado por exceção/citação; preserva termo nativo,
   pattern/body snippets e location do braço.
7. Exceção com justificativa vazia ou `ok` gera warning adicional; exceção cujo path:line
   exato não corresponde a catch-all sintático gera warning obsoleto. Exceção de outro
   arquivo não interfere; matching e ordem seguem o algoritmo completo do L0.
8. Ordem de expressões/braços é preservada. A ordem de entrada do HashMap de exceções não
   pode tornar a saída pública não determinística; ocorrências distintas não são perdidas.
9. Campos irrelevantes, Unicode, paths relativos/absolutos, linha/coluna extremas e
   coleção vazia não causam panic nem alteram decisões.
10. V16 não emite V17–V20 e o gate não usa helper/classificador da produção como oráculo.

## Gate mínimo

- matriz de languages e sete `ScrutineeForm`;
- candidato: mesmo prefixo em braços distintos, prefixos divergentes, duplicata num braço;
- produto catch-all × reincorporação × barreira e todos os `BodyForm`;
- evidência, severidade, termo/snippets e ordem/cardinalidade;
- exceção válida, vazia, `ok`, obsoleta, outro arquivo, paths parecidos e múltiplas
  exceções sob ordens de inserção diferentes;
- mutação sistemática de campos irrelevantes e limites/Unicode;
- regressão: somente rule_id V16.

Resultados: `PASS`, `RED`, `SPEC-GAP` ou `GATE-DEFECT`. Achados são congelados antes de
correção. Fechamento exige adversário `NÃO REABRIR`, suíte, hashes, auto-lint, rustfmt
scoped, diff-check e relatório P0087. Nenhum merge ocorre neste branch.

## Resultado e fechamento

A rodada inicial bloqueou por SPEC-GAP de API, exceções, `ScrutineeForm::Other`, path e
determinismo. Os achados foram congelados em `f011d8c`; o L0 executável foi saneado em
`2602ec0` antes de qualquer correção funcional.

O gate cego `d63dde3` produziu 5 PASS / 3 RED:

1. prefixo duplicado no mesmo braço criava falso enum candidato;
2. catch-all sintático isento era marcado como exceção obsoleta;
3. warnings obsoletos herdavam a ordem de inserção do HashMap.

As três correções foram materializadas em `df3f750`, com headers da família V16–V20
resselados. O gate final, SHA-256
`e4657dd46d03339afe861460cf2a4cfaf3a043b56a1223cef38d497fa032e02f`, passou 8/8.

O adversário final confirmou os três REDs fechados, gate completo, regressão V17–V20
10/10, pureza L1 e causalidade L0→L1. Nenhum RED/SPEC-GAP/GATE-DEFECT permanece no
classificador puro. Parser DecisionExpr e carregamento/wiring de exceções permanecem
residuais explícitos para lote de integração.

Veredito: `READY WITH RESIDUAL AUDIT`. Nenhum merge, instalação ou release foi executado.
