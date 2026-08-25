# Assessment 0030 — fechamento dos REDs Git de F05

**Estado:** PROTOCOLO CONGELADO — gates e produção proibidos
**Data:** 2026-08-25
**Passo:** P0101
**Baseline funcional:** `ba6f3a1c6cf0142ff44075fce6cd903a5f3d1dcf`
**Envelope operacional:** `d7da297` acrescenta somente P0101 ao baseline

## Insumos hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| protocolo P0101 | `00_nucleo/tekt-linter-passo-0101-fechamento-reds-git-refinement.md` | `e6d80e6440f997750ee39afa43dd70ab37d176078c072b8d639f2a4dc3b5afd2` |
| Assessment P0100 | `00_nucleo/assessments/0029-auditoria-funcional-git-refinement.md` | `9d643bc5a8c887d7ab328879fcd989558ecf5b40de40e9f847280c80a4d7cf15` |
| relatório P0100 | `00_nucleo/relatorio-p0100-auditoria-funcional-git-refinement.md` | `a69e6a72caff573f3666d61ab39477c3c72fdaef2500e18155beea603792f33f` |
| contrato Git | `00_nucleo/prompts/refinement-validator.md` | `9ab972915e8f21e6c0fc323686d507fb2cb4b590de6d987b454e05642f167818` |
| ADR B2 | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `cdd1acfe688aabd0c2bb0b7061a55c80dc47f1d7745c8a5c2e7f7f560115485f` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| produção parcial | `03_infra/git_refinement.rs` | `db50933e6976913a2ce0c1acb9883faf2efbab3e41135437440640027c51ef6b` |
| gate protocolo | `tests/git_refinement_protocol_assessment.rs` | `89bdaa09f3a1e3dff7cf30be71630f1cfafbe4b0d314d74e3faa607858c41eb0` |
| gate lifecycle | `tests/git_refinement_timeout_assessment.rs` | `076106ff4c868165634661d720b2f6e9b71851126d5f4022e1b231d9ec69c442` |
| gate histórico | `tests/git_refinement_assessment.rs` | `9609ebdb84d21fb79cddd744392d9fb8692513c809bf651c52eefa1c8b75c434` |
| CLI histórica | `tests/refinement_cli.rs` | `641dbd3088710efc77d9e209a613236b59b819d565af7a2ee8fd80346d386408` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

## REDs congelados

| ID | Divergência | Gate previsto |
|---|---|---|
| R1 | rota produtiva usa adapter histórico paralelo | B3 |
| R2 | Windows sem Job Object/descendentes | B4 runtime Windows |
| R3 | oversized com pipe aberto vira timeout | B1 |
| R4 | loose object/pack acessível pode ser symlink externo | B2 |
| R5 | líder sai, descendente segura pipes; deadline/reap não cobre tudo | B1 + B4 |

## Infraestrutura disponível

O host atual possui apenas targets Rust `x86_64-unknown-linux-gnu` e
`x86_64-unknown-linux-musl`; `wine`/`wine64` não estão disponíveis. Portanto B4 runtime
Windows não pode produzir `PASS` nesta execução. Instalação, download ou emulação não
foram autorizados. R2 permanece bloqueante mesmo que uma implementação `cfg(windows)` e
cross-check auxiliar sejam materializados.

## Segregação

- A pode ler somente produção/wiring necessários para mapear R1–R5; não edita.
- B1–B4 começam somente após A congelado.
- B1/B2/B3 possuem arquivos e fixtures próprios; B4 deve ser gate Windows real.
- produção só reabre após RED dos gates adicionais.
- D confronta R1–R5 separadamente e não promove cross-compile a runtime.

P0101 pode fechar melhorias Linux como evidência parcial, mas F05 só fecha com R1–R5
`PASS`. Resultado final: `READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Sem merge/push.
