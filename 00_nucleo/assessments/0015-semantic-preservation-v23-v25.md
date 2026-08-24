# Assessment 0015 — preservação semântica V23–V25

**Estado:** CONGELADO PARA TRIAGEM SEGREGADA
**Data:** 2026-08-24
**Alvos:** `context_erasure`, `semantic_field_loss`, `decision_ownership`
**Baseline:** `ce15824d57aa1f906b05a215ffa688258ef80153`

## Insumos L0 autorizados

| Regra | Caminho | SHA-256 |
|---|---|---|
| V23 | `00_nucleo/prompts/rules/context-erasure.md` | `09545244abfb7209cbd2d987322098365d5dd24e0b0569a0ca49b1f808ab9e3a` |
| V24 | `00_nucleo/prompts/rules/semantic-field-loss.md` | `f4b7593c8990a4817ce1c767d5f67c21856a908ce6220acb7774cc93edb84cab` |
| V25 | `00_nucleo/prompts/rules/decision-ownership.md` | `addf36ee1ede26f974f782e3a5c180344b99dd99af482a05c58c21f722681341` |

## Natureza e fronteira

Assessment retroativo (`assessed`, não `sealed`) dos classificadores L1 puros. Este lote
não audita carregamento TOML, extração pelos parsers, dispatcher, seleção CLI ou SARIF;
esses consumidores formam um lote posterior. Durante a triagem, produção não pode ser
alterada antes que qualquer RED/SPEC-GAP seja congelado.

O verificador recebe somente este assessment e os três L0 acima, presos pelos hashes
exatos. Não pode ler L1–L4, testes existentes, histórico ou relatórios antes de congelar
o gate. Insumo insuficiente é `SPEC-GAP`, nunca expectativa adivinhada.

## Alegações congeladas

1. **Escopo comum:** V23–V25 são silenciosas para observações que não pertencem à sua
   categoria e preservam ordem, multiplicidade e location das observações elegíveis.
2. **V23:** emite exatamente um `Warning` por resolução contextual com neutro que alcança
   sumidouro, exceto fonte `absolute-only`; e por projeção apagadora cujo resultado
   alcança sumidouro do mesmo contrato. Operações fora do contrato não diagnosticam.
3. **V24:** sob `normalization = preserve`, emite exatamente um `Warning` quando o slot
   obrigatório usa forma neutra sem depender da origem; dependência da origem e
   `drop-to-default` isentam. Ausência/opacidade não vira acusação.
4. **V25:** emite exatamente um `Warning` para cada `duplicate-owner`, `proxy-reentry` ou
   `canonicalizer-reentry`; consumidor que chama owner, identidade diferente e operação
   anterior ao marco não diagnosticam.
5. **Identidade/evidência:** cada diagnóstico preserva id do contrato/decisão, modalidade
   ou operação decisiva, source/destination/owner/consumer aplicável, path, linha e coluna.
6. **Isolamento:** variar campos irrelevantes não altera saída; V23/V24/V25 não se ativam
   mutuamente e não inferem significado de nomes, literais ou similaridade textual.
7. **Determinismo:** permutação equivalente, particionamento e duplicatas idempotentes de
   fatos de conjunto não alteram o significado; multiplicidade normativa de ocorrências
   distintas não é perdida.
8. **Totalidade:** coleção vazia, strings vazias/Unicode, spans extremos e valores opacos
   não causam panic nem falso positivo.

## Gate mínimo

- controles positivo/negativo de cada cenário enumerado nos três L0;
- produto categoria × regra para provar isolamento;
- V23: neutro/contexto/absolute-only, projeção/sumidouro e contrato divergente;
- V24: preserve/drop-to-default, dependência/neutralidade, origem ausente/opaca;
- V25: três modalidades, identidades iguais/diferentes, owner/consumer e antes/depois;
- ordem/cardinalidade com múltiplas observações e locations distintas;
- mutação sistemática de campos irrelevantes com comparação integral da saída;
- permutação/duplicação somente onde o contrato declara semântica de conjunto;
- Unicode, vazio e ausência de panic.

Cada resultado é `PASS`, `RED`, `SPEC-GAP` ou `GATE-DEFECT`. O gate não pode importar
helpers, constantes ou classificadores da produção como oráculo compartilhado.

## Fechamento

Antes de merge: achados congelados, saneamento normativo anterior à produção, gate
independente verde, adversário final `NÃO REABRIR`, suíte global, hashes dry-run,
auto-lint, rustfmt scoped, `git diff --check`, relatório P0086 e estado final
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Este branch não executa merge.
