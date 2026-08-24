# Assessment 0019 — partição de resultados do walker

**Estado:** RESSELADO APÓS SPEC-GAP; AGUARDA GATE CEGO
**Data:** 2026-08-24
**Passo:** P0090
**Baseline:** `40c374d572dcb0c674b807ebe498f1fb12c1b650`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/pipeline | `00_nucleo/prompts/linter-core.md` | `200c8dad9e1408ef4d930553675282777b8a862af8a1a76d7622b74359b3480e` |
| tipos/contrato | `00_nucleo/prompts/contracts/file-provider.md` | `fc4988033a6cb8d54dcef76ae16f12486dc5e3bc0edcd1be947793dcb817d817` |
| walker | `00_nucleo/prompts/file-walker.md` | `6deeec38a766c6ac16f8aa90944e75a6b6d22c91db1249f1d99fdf51c697a7c2` |
| motor/fail-fast | `00_nucleo/adr/0004-reformulação-do-motor-de-análise.md` | `33380a0bbcfb93dcdfa5ee7511d2be31bbcdfb1a4752b436726713458783a5d0` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0090 | `00_nucleo/tekt-linter-passo-0090-auditoria-particao-resultados-walker.md` | `eb0d4ae5e308798ba3a9ba2525d62c615a660a9f42422cb9f3bc41d70a920cd6` |

## Alegações a congelar

1. Vazio produz `(vec![], vec![])`.
2. Cada Ok vai exatamente uma vez ao vetor de `SourceFile`; cada Err vai exatamente uma
   vez ao vetor de `SourceError`.
3. Ordem relativa e multiplicidade são preservadas separadamente em ambos os vetores.
4. Erro não encerra nem descarta itens posteriores.
5. Todos os campos e bytes semanticamente observáveis são preservados sem normalização.
6. O iterador é consumido uma vez, até o primeiro EOF, sem `Clone`, replay ou consulta
   posterior a `next`.
7. `size_hint` inexato/hostil não altera o resultado nem autoriza pré-alocação sem limite
   que mude a semântica observável.
8. A transformação não acessa filesystem, config, ambiente, relógio, rede ou processo.

## Questões bloqueantes

O adversário A deve decidir camada causal, API pública, ownership, estabilidade de ordem
e contrato de consumo. Ausência de decisão no L0 é `SPEC-GAP`; nenhuma API privada de L4
pode ser ratificada por conveniência do gate.

## SPEC-GAP congelado e decisão causal

O adversário A confirmou que a partição é L1 e bloqueou por ausência de API/contrato de
consumo. O L0 resselado publica `collect_walker_results<I: Iterator<...>>`, movimento
sem clone, subsequências estáveis, `Vec::new` + `push`, exatamente uma observação terminal
`None`, nenhuma chamada pós-EOF e nenhuma consulta a `size_hint`. L4 apenas chama.

## Papéis

- A: adversário somente Assessment/L0 hash-pinned;
- B: verificador novo, sem produção, após saneamento;
- C: confronto somente após gate congelado;
- D: adversário final de causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge.
