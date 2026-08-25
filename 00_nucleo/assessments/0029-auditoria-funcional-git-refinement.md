# Assessment 0029 — auditoria funcional Git de `refine-revisions`

**Estado:** L0 SANEADO — aguardando re-preflight A; produção e gates proibidos
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

## RED/SPEC-GAP A1 e saneamento

A1 validou os quatorze pins e encontrou três itens de preflight `PASS`, oito
`SPEC-GAP` e um `RED` normativo; nas alegações, sete `PASS`, cinco `SPEC-GAP` e um
`RED`. A adenda permitia alternates locais apesar da autocontenção fechada por P0072, e
o L0 não publicava transcript, ambiente, framing, contabilidade, lifecycle, taxonomia ou
seam de executável suficientes para um gate hostil independente.

O saneamento alterou somente `refinement-validator.md` e ADR-0019. Foram congelados:

- autocontenção sem alternates/object stores externos, `.git` indireto, bare ou linked;
- seam pública L3 `load_revision_with_git`, que troca apenas o path absoluto do
  executável e atravessa o adapter real;
- tipos públicos `GitRevisionContent`, `GitPathContent`, `GitUnknownReason` e
  `GitRevisionError`;
- gramáticas de ref/path, ambiente limpo, prefixo `-c`, três argvs e framing byte-level;
- budgets inclusivos, contabilidade e descarte integral;
- timeout por operação, grupo/job, kill, drenagem, reap e falha de contenção;
- matriz fechada entre erro L3, `Missing` e razões `Unknown`.

Novas identidades a validar por A2:

| Unidade | SHA-256 saneado |
|---|---|
| `00_nucleo/prompts/refinement-validator.md` | `86e5d4e35f0abbb8099ff0a37da25d62253cb2352b0af57582917ea508676391` |
| `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `cdd1acfe688aabd0c2bb0b7061a55c80dc47f1d7745c8a5c2e7f7f560115485f` |
| parecer A1 | `afb696a894c87b3f90875fca7388c4512d7c53b9912402077164c3aad07a3a1f` |

O drift V5 decorrente do novo prompt é esperado e não será reparado antes dos gates:
atualizar headers de produção nesta fase contaminaria o RED causal. A2 deve ler somente
o Assessment, A1 e os dois L0 saneados; B1/B2 continuam proibidos até `PASS` de A2.

## A2 e gates congelados

A2 revalidou os dois L0 saneados e fechou o preflight em 12/12 `PASS` e as alegações em
13/13 `PASS`. O RED de alternates foi resolvido e a seam pública foi considerada
executável. Parecer A2: SHA-256
`d636f6411c8f8a73bf60ff0e63a91885e26a8f4335eb1c8db8ddfb383087e195`.

| Papel | Artefato | SHA-256 | RED inicial |
|---|---|---|---|
| B1 | `tests/git_refinement_protocol_assessment.rs` | `29b7bc053c88e2bdf6102319e9677917c94e464669f2f5bef5f9f8cc6883fcf9` | compilação: API L3 e tipos ausentes |
| B1 | `tests/fixtures/git_refinement_protocol/hostile_git.sh` | `d8a698c71b8e801b00b1fbe493c42b184193f6fef814622e639a52720dac6b0e` | fixture congelada |
| B2 | `tests/git_refinement_timeout_assessment.rs` | `96d230525992acf779374f57ff15eb7ce4734df3444ee97e2ee7b198866750a3` | compilação: API L3 e tipos ausentes |
| B2 | `tests/fixtures/git_refinement_timeout/hostile-git.sh` | `5f3265bdad976531da3b463732ca51801a381f570a01de3e7b9c7f9ad2b4c5b9` | fixture congelada |

B1 cobre sete testes de protocolo, identidade, ambiente, tipos, framing e budgets. B2
cobre quatro testes de status, saída parcial, timeout/reap e descendente. Antes do
congelamento foi corrigido um `GATE-DEFECT`: a fixture B2 identificava o subcomando pelo
último argv e não alcançaria o cenário parcial; agora percorre argv nominalmente.

O RED causal comum está preservado: `load_revision_with_git`, `GitRevisionContent`,
`GitPathContent`, `GitUnknownReason` e `GitRevisionError` ainda não existem na API
publicada. Produção está liberada somente após o commit destes hashes.
