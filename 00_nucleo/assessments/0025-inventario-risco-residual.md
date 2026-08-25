# Assessment 0025 — inventário de risco residual

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-25
**Passo:** P0096
**Baseline:** `75c076951b2a873b74bfbe163fef34c4ca5f2800`

## Insumos iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| protocolo P0096 | `00_nucleo/tekt-linter-passo-0096-inventario-risco-residual.md` | `cf5b4aa0d70c2033caea975a7b17439f0a93be9749ee3a3cbf3c1104c400c7ab` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| fechamento inicial | `00_nucleo/assessments/0013-fechamento-pre-merge.md` | `3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9` |
| último Assessment | `00_nucleo/assessments/0024-refinement-snapshot-loader.md` | `6e32d9d8b798928c438bca3959b5de35bbd899a4dfed646860e82ffd4f51bb3c` |
| fechamento P0094 | `00_nucleo/relatorio-p0094-auditoria-n16-summary.md` | `5631381eabae71e090f663d5ce00093a2f524bddcd9eb4bd744afd08699da4b6` |
| fechamento P0095 | `00_nucleo/relatorio-p0095-auditoria-loader-snapshots-refinamento.md` | `33c7ae9cf9608daf1f99e289c1af8eb9a2be78e80515e7acf8d5709033106b01` |
| método de insumo cego | `00_nucleo/relatorio-p0081-fechamento-insumo-normativo-cego.md` | `25177c76c5a3005daccac54dea4c085b04a7a58aed268b50fbbfc8763273c33d` |

## Pergunta de auditoria

Quais seams comportamentais ainda não possuem fechamento segregado suficiente após os
Assessments 0001–0024, qual é o risco reproduzível de cada uma e qual única seam, se
alguma, pode ser proposta para P0097 sem fingir baixo risco?

## Invariantes

1. Nenhum arquivo fora de `00_nucleo` pode mudar.
2. Produção e testes são evidência somente leitura; teste existente não vira gate por
   nome ou por estar verde.
3. A unidade é seam comportamental, não arquivo nem contagem de linhas.
4. Cobertura exige autoridade, gate independente e consumidor confrontado.
5. Arquitetura segue L1 tipos/regras, L2 apresentação, L3 I/O/parsers e L4 coordenação.
6. Divergência de hash, omissão relevante ou cobertura sem evidência é `RED`.
7. Autoridade ausente ou contraditória é `SPEC-GAP`.
8. Gate histórico/acoplado tratado como independente é `GATE-DEFECT`.

## Protocolo ativo

- A lê este Assessment, P0096 e fechamentos/Assessments; não lê produção e não recomenda
  P0097.
- B1 lê este Assessment, produção e testes; não lê pareceres de A nem classifica produção
  como autoridade.
- B2 lê este Assessment, prompts/ADRs e índice de passos; não lê produção nem B1.
- C somente começa após congelamento textual e hash dos três pareceres.
- D confronta o inventário consolidado, sem editar.

Fechamento somente `READY WITH RESIDUAL AUDIT` ou `BLOCKED`. P0096 não autoriza produção,
gate executável, correção funcional, merge ou push.

## RED D1 e resselamento

D bloqueou a primeira reconciliação: o protocolo nomeava `04_forge`, inexistente, em vez
de `04_wiring`; A omitiu o fechamento da materialização/refinement no Assessment 0013; C
classificou S3 como `UNAUDITED` e a recomendou sem causa válida de reabertura; B1 afirmou
ausência de sync embora `write_atomic` execute `file.sync_all()`.

O protocolo foi corrigido para `04_wiring`. A deve reconciliar S3 com o Assessment 0013;
B1 deve registrar precisamente que há sync do arquivo, mas não fsync do diretório nem
preservação explícita de modo; C anterior está invalidado e deve ser refeito. Nenhuma
produção mudou.

## RED D2

D bloqueou a segunda reconciliação porque `SourceConstant`/`HasConstants` possui dois
consumidores diretos, V21 e V22, enquanto S5a identificava e pontuava apenas V21. O recorte
deve incluir regressão V22, proibir ambos os classificadores como oráculo e pontuar a
extração em memória como pura, com dois consumidores.

B2 identificou `SPEC-GAP` na associação compartilhada de citações: V21 aceita somente
`ref:/spec:/rationale:`, enquanto o histórico V22 reconhece evidência mais ampla e a IR
possui um único `citation`. O candidato só permanece elegível se excluir associação,
janela e semântica de citações, limitando-se à extração estrutural de `SourceConstant`.

## Fechamento D3

Após dois bloqueios adversariais e resselamentos, D confirmou hashes, universo
`04_wiring`, cobertura histórica, busca reversa, pontuação e delta exclusivamente
documental. A matriz final recomenda S5a — extração estrutural Rust de `SourceConstant`
compartilhada por V21/V22 — com risco 10, médio. O recorte exclui citações, frescura,
configuração global, wiring, apresentação, exit e os dois classificadores como oráculo.

Parecer: `READY WITH RESIDUAL AUDIT`. Residual obrigatório: associação/janela de citações
e agregação semântica V22 permanecem fora do P0097 até saneamento L0. Nenhum arquivo de
produção ou teste foi alterado.
