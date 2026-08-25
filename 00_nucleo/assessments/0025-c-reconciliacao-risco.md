# Assessment 0025/C — reconciliação e ranking de risco

**Papel:** C, somente artefatos A/B1/B2 congelados
**Resultado:** PASS
**Produção/testes lidos:** não

## Insumos

| Artefato | SHA-256 |
|---|---|
| A — cobertura histórica | `8eb464e16011c088529d5fc4ff5f649e29c4730434dabe1a10fbb6159125d8dc` |
| B1 — inventário estrutural | `2ed0f8f0ebe94d6050ce708e520ed92382fa518e0f080d13c12d1baa1347b598` |
| B2 — inventário normativo | `c53a381bce30c0b37f44b8c7bd530161e906f150be23bd88c355fe8b5e370c20` |

## Matriz consolidada

Pontuação: camadas/efeitos/entrada/consumidores/L0/gates/regressão.

| Seam | Cobertura | Pontuação | Total | Risco | Confiança | Tratamento |
|---|---|---|---:|---|---|---|
| S1 — extractor/escritor de snapshot | PARTIAL | 2/2/3/2/2/2/2 | 15 | alto | média | sanear L0 antes da seam completa |
| S2 — refinamento Git/subprocesso | INDETERMINATE | 2/3/3/2/3/2/2 | 17 | crítico | alta no risco | sanear contradição L0 |
| S3 — manifesto, recibo e selo | UNAUDITED | 1/2/2/1/0/2/2 | 10 | médio | alta | candidato delimitável |
| S4 — pipeline principal | PARTIAL | 3/3/3/3/2/2/3 | 19 | crítico | alta | decompor por comando |
| S5 — fidelidade de nove parsers | PARTIAL | 2/1/3/2/2/2/3 | 15 | alto | média | separar linguagem/característica |
| S6 — preflight/precedência CLI ampliada | PARTIAL | 1/3/2/2/2/2/3 | 15 | alto | média | recortar um subcomando/caso |

S1 atravessa L3/L4, escreve snapshot e recebe TOML/path/fonte/query; schema, limites,
erros e atomicidade permanecem parcialmente abertos. S2 executa processo/threads/timeout
com revisões hostis e possui autoridade contraditória. S3 é fronteira L3→L4 com um
consumidor, escrita/rename, L0 forte e testes ainda não independentes. S4 coordena L1–L4,
filesystem, Rayon, output, exit e reruns. S5 fabrica o IR consumido por todas as regras,
mas tem L0 desigual por linguagem. S6 amplia apresentação já fechada para precedência e
exit globais, exigindo recorte.

## Discordâncias preservadas

1. Git genérico está fechado, mas Git revisional tem vigência contraditória: S2 permanece
   `INDETERMINATE`.
2. S3 tem testes substanciais e L0 forte, mas nenhum fechamento dedicado: permanece
   `UNAUDITED`.
3. Roteamento MultiParser fechado não prova fidelidade dos produtores concretos S5.
4. Apresentação L2 fechada não cobre a ampliação de precedência CLI de S6.
5. Loader P0095 fechado não cobre extractor/escritor S1 nem publicação S3.

## Ranking

1. S3 — manifesto, recibo e publicação de selo: médio, delimitável;
2. S6 estreita — um único caso de precedência: alto, depende de novo recorte;
3. S1 — extractor de snapshot: alto, bloqueado por saneamento SG-B2-03.

## Recomendação P0097

Recomendar **S3 — manifesto, recibo e publicação de selo**, restrita ao contrato entre
`refinement_seal` L3 e o bloco Seal L4, incluindo publicação observável e exit do
consumidor direto.

S3 está `UNAUDITED`, possui fronteira e consumidor identificáveis, pré-condições L0
enumeráveis, permite gates cegos separados, admite pontos de parada entre contrato,
materialização e consumidor e tem o menor risco elegível: **10, médio**. A recomendação
não a descreve como simples ou de baixo risco e não autoriza P0097.
