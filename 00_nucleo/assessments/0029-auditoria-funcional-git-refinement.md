# Assessment 0029 — auditoria funcional Git de `refine-revisions`

**Estado:** PREFLIGHT CONGELADO — produção e gates proibidos
**Data:** 2026-08-25
**Passo:** P0100 / F05
**Baseline:** `8c28cc01ea7cdb47aa9e8e582597085304a7ece4`

## Insumos L0 hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| protocolo P0100 | `00_nucleo/tekt-linter-passo-0100-auditoria-funcional-git-refinement.md` | `e85740fb3057b030c0a328e32dbd70e1ed3b36bc7bdde2c618dda4169337da06` |
| contrato de refinamento | `00_nucleo/prompts/refinement-validator.md` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` |
| ADR B2 | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `088e5806c948d60c2f5b1ea2c04c4b181672c037c31f53c0b125ddf594a497d6` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment F04 | `00_nucleo/assessments/0028-vigencia-refine-revisions.md` | `46b1bcec486c8e909fe1bc66a36e9e0a9b7c91d992ead6f8fd20f53fd73b4ba2` |
| decisão F04 | `00_nucleo/assessments/0028-c-decisao-saneamento.md` | `1fa048e01935717806ef48ae6ea74cda62cc2f26b6807ee2d806f8176adf9f06` |
| fechamento P0099 | `00_nucleo/relatorio-p0099-saneamento-vigencia-refine-revisions.md` | `15d0d757414358f94f073b7084c1b0d057c834986343148c0dad56fb8f854588` |
| Assessment Git histórico | `00_nucleo/assessments/0001-git-refinement.md` | `5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17` |
| fechamento P0072 | `00_nucleo/relatorio-p0072-saneamento-deterministico-segregado.md` | `d43d1dd6e9d356b0f3dcd652a57c02cf7af0d24ecf24fb8c6d419d7b2a393fb7` |
| gate Git real histórico | `tests/git_refinement_assessment.rs` | `9609ebdb84d21fb79cddd744392d9fb8692513c809bf651c52eefa1c8b75c434` |
| inventário estrutural | `00_nucleo/assessments/0027-b1-delta-estrutural.md` | `fac7a67068e6f63a969f3725710026afed3f828275859e8f49cafb6a1ec914e2` |
| inventário normativo | `00_nucleo/assessments/0027-b2-autoridade-promessas.md` | `3f4fee9273c72ca0202e9f2ab95e551f7f53e55c7862493ba34660a148f94e3e` |

## Fronteira congelada

P0100 observa a fonte Git B2 em L3: refs/paths/respostas hostis entram; OIDs imutáveis,
bytes de blobs regulares ou falha tipada saem para o extrator compartilhado. Git real é
somente regressão. O oráculo principal deve controlar processo, argv, ambiente e framing.

Exits/precedência F09, composição F08, schema/writer F01–F03, comparador L1, backend e
efeitos fora do envelope B2 são opacos e proibidos.

## Preflight A

A deve classificar os doze itens de preflight e as treze alegações de P0100. Deve decidir
se existe contrato suficiente para injeção de processo hostil sem ler produção. Se não
existir, retorna `SPEC-GAP` e propõe apenas seam L1/L3 mínima; B1/B2 permanecem proibidos
até saneamento e resselamento.

## Segregação

- A lê somente este Assessment e os quatorze insumos; não lê produção.
- B1/B2 só começam após A e qualquer saneamento L0 congelado.
- B1 e B2 possuem arquivos e fixtures independentes.
- produção só abre após hashes dos gates e RED inicial registrados.
- D fecha contenção, causalidade, arquitetura e regressões.

Classificações: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Sem merge ou push.
