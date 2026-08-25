# Assessment 0025/C — reconciliação refeita após RED D1

**Papel:** C, somente artefatos A/B1/B2 resselados
**Resultado:** PASS
**Produção/testes lidos:** não

## Insumos

| Artefato | SHA-256 |
|---|---|
| A — cobertura histórica | `910b47385147c9851d8a62b284fcec6dc2a77fc064adca2c26cc8aecd2fb8ccd` |
| B1 — inventário estrutural | `865e19d772ced34aceee2d6322d2ba1d68bd388e633ae819aecfea4f8953d995` |
| B2 — inventário normativo | `22247bac303a4ee999543cd0d96d9e705e6fb445bed6c50d1c723aa1de427701` |

A reconciliação anterior está integralmente descartada.

## Matriz consolidada

Pontuação: camadas/efeitos/entrada/consumidores/L0/gates/regressão.

| Seam | Cobertura | Pontuação | Total | Risco | Confiança | Tratamento |
|---|---|---|---:|---|---|---|
| S1 — extractor/escritor de snapshot | PARTIAL | 2/2/3/2/2/2/2 | 15 | alto | média | sanear SG-B2-03 |
| S2 — refinamento Git/subprocesso | INDETERMINATE | 2/3/3/2/3/2/2 | 17 | crítico | alta no risco | sanear SG-B2-04 |
| S3 — manifesto, recibo e selo | CLOSED_WITH_RESIDUAL | 1/2/2/1/1/0/2 | 9 | médio | alta | não reabrir |
| S4 — pipeline principal | PARTIAL | 3/3/3/3/2/2/3 | 19 | crítico | alta | decompor por comando |
| S5 — nove parsers concretos | PARTIAL | 2/1/3/2/2/2/3 | 15 | alto | média | separar característica/linguagem |
| S6 — preflight/precedência CLI ampliada | PARTIAL | 1/3/2/2/2/2/3 | 15 | alto | média | recortar caso exato |

S1 atravessa L3/L4, escreve e recebe TOML/path/fonte/query. S2 executa Git, threads,
timeout e kill sob autoridade contraditória. S3 possui fechamento histórico com gate
independente 16/16 e consumidor confrontado. S4 coordena L1–L4 e política global. S5
fabrica IR consumido por todas as regras, com L0 desigual. S6 amplia apresentação fechada
para precedência e exit globais.

## Desdobramento autorizado de S5

| Seam estreita | Cobertura | Pontuação | Total | Risco | Confiança |
|---|---|---|---:|---|---|
| S5a — extração estrutural Rust `SourceConstant` | PARTIAL | 1/0/3/2/0/2/2 | 10 | médio | alta |
| S5b — `DecisionExpr`/wiring V16 | PARTIAL | 2/1/3/2/2/2/2 | 14 | alto | média/baixa |

S5a limita-se a fonte Rust→IR estrutural `SourceConstant`→V21/V22: literal,
span/localização, direção de multiplicação/divisão, campos profundos, `context_var`,
`geometric_sink` e origem em teste/tabela. A extração em memória é pura; os dois
consumidores diretos elevam `consumidores` a 2. Exclui completamente associação, janela e
semântica de citações, agregação V22, ambos os classificadores como oráculos, frescura,
configuração global, wiring, apresentação e exit. S5b cruza mais consumidores e pode
revelar SPEC-GAP no mapeamento AST→IR.

## Discordâncias preservadas

1. S3 tem resíduos operacionais, mas A0013 fecha entidade, infraestrutura e wiring com
   gate 16/16; não existe causa de reabertura.
2. Git genérico fechado não resolve a vigência contraditória do Git de refinamento S2.
3. Roteamento MultiParser fechado não prova fidelidade/associação dos parsers concretos.
4. Apresentação L2 fechada não cobre a ampliação de precedência global S6.
5. Loader P0095 fechado não cobre extractor/escritor S1.
6. S3 sincroniza o arquivo com `file.sync_all()`; resíduos precisos são fsync do diretório
   e preservação explícita de modo.

## Ranking e recomendação

1. S5a — extração estrutural Rust `SourceConstant`: 10, médio, confiança alta;
2. S5b — extração/wiring V16: 14, alto, confiança média/baixa;
3. S6 por comando/caso: 15, alto, ainda sem recorte exato.

Recomendar **S5a — extração estrutural Rust de `SourceConstant` compartilhada por V21/V22**
para P0097. Está `PARTIAL`, possui fronteira fonte→IR→dois consumidores delimitada,
pré-condições normativas enumeradas, admite gate cego sem usar V21 ou V22 como oráculo,
inclui regressões explícitas dos dois consumidores, exclui citações e integração externa
e tem o menor risco elegível:
**10, médio**. P0097 deve parar em `SPEC-GAP` se exigir decisão fora da lista fechada.
