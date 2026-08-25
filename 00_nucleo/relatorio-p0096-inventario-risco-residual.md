# Relatório P0096 — inventário segregado de risco residual

**Data:** 2026-08-25
**Branch:** `codex/audit-residual-risk-inventory`
**Baseline:** `75c076951b2a873b74bfbe163fef34c4ca5f2800`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Resultado

O inventário confirmou que não restam seams claramente simples. Das seis fronteiras
amplas encontradas, duas têm risco crítico, três alto e a materialização do selo já estava
fechada. O único recorte elegível para um próximo lote tem risco médio.

| Seam | Cobertura | Risco | Tratamento |
|---|---|---:|---|
| extractor/escritor de snapshot | PARTIAL | alto (15) | sanear L0 |
| refinamento Git/subprocesso | INDETERMINATE | crítico (17) | resolver contradição L0 |
| manifesto/recibo/selo | CLOSED_WITH_RESIDUAL | médio (9) | não reabrir |
| pipeline principal | PARTIAL | crítico (19) | decompor por comando |
| nove parsers concretos | PARTIAL | alto (15) | separar característica/linguagem |
| precedência CLI ampliada | PARTIAL | alto (15) | recortar caso exato |

## Segregação e correções adversariais

- A reconciliou Assessments 0001–0024 sem ler produção.
- B1 inventariou produtores/consumidores sem usar produção como autoridade.
- B2 mapeou L0 e SPEC-GAPs sem ler implementação.
- C calculou as sete dimensões somente após congelamento dos três artefatos.
- D confrontou hashes, produção, testes, consumidores e arquitetura.

D1 bloqueou `04_forge` no protocolo, a omissão do fechamento histórico do selo e uma
alegação incorreta de ausência de sync. D2 bloqueou a omissão de V22 como consumidor de
`SourceConstant`. B2 então encontrou SPEC-GAP na semântica compartilhada de citações. Cada
achado foi preservado, corrigido e resselado antes da reconciliação seguinte.

## Recomendação P0097

Auditar somente a extração estrutural Rust de `SourceConstant`, da fonte ao IR
compartilhado por V21 e V22:

- literal e localização/span;
- direção de multiplicação/divisão e campos profundos;
- `context_var` e `geometric_sink`;
- origem em teste e data-table.

Pontuação: `1/0/3/2/0/2/2 = 10`, risco médio. Gates devem ser fonte→IR, independentes e
não podem usar V21 ou V22 como oráculo. Regressões dos dois consumidores são obrigatórias.

Ficam expressamente fora: associação/janela/semântica de citações, agregação V22,
frescura filesystem, configuração global, wiring, apresentação e exit. Necessidade de
qualquer decisão nesses temas é `SPEC-GAP` e interrompe P0097.

## Validação

- hashes A/B1/B2/C e protocolo: PASS;
- amostras de CLOSED, CLOSED_WITH_RESIDUAL, PARTIAL e INDETERMINATE: confrontadas;
- apenas `00_nucleo` mudou desde o baseline;
- workspace: 630 unitários e todas as integrações/fixtures PASS;
- V5/V6/V7/V12: nenhuma violação;
- reparador V5 dry-run: `Nothing to fix`;
- `git diff --check`: PASS.

Nenhum código, gate executável, fixture ou configuração foi alterado. Nenhum push foi
realizado.
