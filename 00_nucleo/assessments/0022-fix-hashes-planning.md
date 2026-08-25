# Assessment 0022 — planejamento e execução de fix-hashes

**Estado:** PREFLIGHT — SPEC-GAP saneado; produção ainda não confrontada
**Data:** 2026-08-25
**Passo:** P0093
**Baseline:** `6a325dc`
**Commit do protocolo:** `fcc73fc`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| fluxo fix/update | `00_nucleo/prompts/fix-hashes.md` | `d6cc361ed70301c002717b6e80a6c166a0ba1f149084c0f3000c373ba5d1daf9` |
| apresentação/CLI | `00_nucleo/prompts/sarif-formatter.md` | `959d6e56785e6c32087fcae361300304d4a8197a2669f9df7f2b4809a4842605` |
| arquitetura | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| tipos V5 | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| writer L3 fechado | `00_nucleo/assessments/0006-prompt-io-and-hashes.md` | `df3b8bcf1f14f1989c978efe620a55a822512a8ccdbf6e5ea35d3d918d636567` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0093 | `00_nucleo/tekt-linter-passo-0093-auditoria-planejamento-fix-hashes.md` | `d08cc83575b318f6a13663fcacfd66152da97b3664657a1e1a67838e2e2e8e0b` |

## Alegações candidatas

1. Uma entrada por ocorrência V5, em ordem, preservando duplicatas e paths integrais.
2. Header ilegível não chama cálculos; header legível chama Hash A/Hash B uma vez.
3. Falhas de header, prompt hash e source hash são estados distintos e observáveis.
4. Cada entrada produz um resultado, inclusive não corrigível e dry-run.
5. Dry-run não escreve e expõe os dois hashes propostos.
6. Execução pronta chama código(Hash A) antes de prompt(Hash B), uma vez cada.
7. Falha da primeira fase impede a segunda; falha da segunda nunca vira sucesso.
8. Falha não interrompe entradas posteriores.
9. Plano/resultados e formatters não permitem alegar dupla paridade incompleta.
10. L2 não acessa I/O nem importa L3.

## SPEC-GAPs congelados

### G1 — plano incompleto e estados inválidos

O `FixEntry` L0 não contém prompt path nem Hash B, embora a dupla paridade os exija. Sua
combinação de string vazia e `Option`s permite estados contraditórios. Falha de Hash A e
Hash B não possuem variantes nominais.

### G2 — resultado ausente

`FixResult`, dry-run e fases de erro não são publicados. B2/B3 não podem inventar forma,
cardinalidade ou apresentação.

### G3 — transação entre duas escritas

P0074 prova atomicidade de cada writer isolado, não da operação composta. O L0 não decide
o que ocorre se o header recebe Hash A e a metadata do prompt rejeita Hash B. Rollback
exigiria bytes anteriores não presentes no port atual.

### G4 — ordem, duplicatas e continuidade

O filtro V5 é explícito, mas uma entrada/resultado por ocorrência, ordem, duplicatas,
chamada única e continuação após erro não estão normatizados.

### G5 — apresentação consumida

O L0 mostra exemplos, mas não distingue nominalmente dry-run, sucesso integral, falha na
fase código, falha na fase prompt e não corrigível. Também não exige mostrar Hash B no
dry-run.

## Decisão recomendada ao adversário A

- enums públicos comparáveis para plano e resultado;
- `Ready` contém source path, prompt path, old hash, Hash A e Hash B;
- falhas `HeaderUnreadable`, `PromptHashUnavailable` e `SourceHashUnavailable` distintas;
- uma entrada/resultado por V5, preservando ordem e duplicatas;
- dry-run distinto e sem escrita;
- execução em duas fases; falha da segunda vira `PartialWrite` explícito, nunca sucesso;
- sem rollback inventado: o port atual não fornece captura/restauração transacional;
- formatter torna fase, razão e hashes relevantes observáveis;
- L2 decide estados; L3 executa primitivas; L4 injeta/reanalisa.

## Parecer A e saneamento

O adversário A validou os oito hashes iniciais e confirmou G1–G5 como `SPEC-GAP`.
Rollback foi rejeitado como expectativa não implementável pelo port atual; a decisão
fail-closed é `PartialWrite` explícito.

O L0 passou a publicar `FixUnavailable`, `FixEntry` e `FixResult` como enums comparáveis,
as quatro combinações de indisponibilidade de hashes, cardinalidade total, dry-run sem
efeitos, ordem das duas escritas, falhas por fase, continuação e apresentação não
enganosa. B1/B2/B3 podem começar após resselamento; produção permanece proibida.

## Papéis

- A: somente Assessment/L0 hash-pinned;
- B1/B2/B3: verificadores e arquivos distintos após saneamento;
- C: produção somente após três gates congelados;
- D: causalidade, transação, consumidor real, arquitetura e regressão.

Resultados: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge/push.
