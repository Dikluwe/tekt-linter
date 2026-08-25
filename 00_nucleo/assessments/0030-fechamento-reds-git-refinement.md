# Assessment 0030 — fechamento dos REDs Git de F05

**Estado:** FECHADO `BLOCKED` — F05 aberto; merge proibido
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

## Materialização B1–B3

| Gate | SHA-256 | Resultado inicial | Classificação |
|---|---|---|---|
| B1 `tests/git_refinement_stream_lifecycle_assessment.rs` | `c946032e31c083d051705f4bfe3ff66c8d03d5894822a99cb47ab8fe7af615f0` | 1/4 PASS | R3 e R5 `RED` |
| fixture B1 `tests/fixtures/git_refinement_stream_lifecycle/fake-git.sh` | `01c05c918eaf376405e511f624f5f4837ec087f80899cff3d87020a2351fc430` | hostil reproduzível; cleanup confirmou ausência de processo remanescente | evidência |
| B2 `tests/git_refinement_object_containment_assessment.rs` | `8d612adac31fc168b2904b4ef32c82f34573a6e753780edf9e9a8a35e4a33925` | 2/7 PASS | R4 `RED` em cinco escapes |
| B3 `tests/git_refinement_productive_route_assessment.rs` | `5d79bc4ac2a96f3e88252b8df654bfd6f9afe7f04bc63835adb00724b8af7cf9` | 2/3 PASS | R1 `RED` |
| B4 `tests/git_refinement_windows_job_assessment.rs` | `7c1541991e8b303767c3d5c0e1b8c1f89599cb4e0cd97775713bc8feed59fc35` | 0 testes neste host Linux | R2/R5 `NOT RUN / BLOCKED` |

B1 prova que header oversized com pipe aberto termina em `Timeout`, que líder encerrado
com descendente segurando pipes ultrapassa o watchdog externo e que descendente sem pipes
permite publicação sem prova de contenção. O controle de cap de transcript já falha cedo.

B2 prova publicação indevida de bytes por symlink em loose object, fanout, `.pack`, `.idx`
e troca regular→symlink após preflight. Os dois controles internos regulares passam.

B3 entra pelo comando publicado. O transcript mostra seis chamadas pelo adapter histórico,
com argv diferente da seam L3. Framing inválido interrompe antes do comparador, e bytes
iguais preservam equivalência semântica com a rota pública `snapshot + refine`.

Nenhuma expectativa foi adaptada à produção. B1–B3 não usam código de exit como oráculo;
não há `SPEC-GAP` nem `GATE-DEFECT` identificado nesta materialização.

B4 é um gate `cfg(windows)` real e compila a fixture Rust hostil no próprio runtime. Ele
confronta associação antes do código hostil, `KILL_ON_JOB_CLOSE`, timeout com descendente
sem pipes, líder encerrado, watchdog e contagem de handles. Neste host, `cargo test`
descobriu zero testes: isso é `NOT RUN / BLOCKED`, jamais `PASS`. A API pública também não
oferece fault injection para falha direta de criação/atribuição do Job; esse caminho deve
ser confrontado no adversário Windows e não é promovido a `SPEC-GAP`.

## Correções C e confronto D

| Unidade | Commit | Evidência final |
|---|---|---|
| stream/lifecycle | `21aa141` | B1 4/4; R3 `PASS` Unix |
| object database | `febafc3` | B2 7/7, porém TOCTOU não fechado |
| rota produtiva | `0b1e4a9` | B3 3/3; R1 `PASS` |
| produção L3 | — | SHA-256 `42bab723efa948b3025a70154d2087493d7104fa9186ba02fc5347e6a4614d65` |
| wiring L4 | — | SHA-256 `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` |

| RED | Veredito D | Causa restante |
|---|---|---|
| R1 | `PASS` | comando publicado usa a seam L3 única e resolve cada ref uma vez |
| R2 | `RED / BLOCKED` | produção não implementa Job Object; B4 executou zero testes no Linux |
| R3 | `PASS` no Unix | oversize é incremental e retorna budget antes do deadline |
| R4 | `RED` | scan pathname antes/depois não elimina troca durante o subprocesso |
| R5 | `RED` Unix + `BLOCKED` Windows | descendente pode escapar do process group; joins ainda podem ficar fora da contenção efetiva |

### GATE-DEFECTs finais

- B2 deixa o symlink instalado no cenário concorrente. Assim prova detecção pós-fato,
  mas não prova ausência de consumo externo quando o atacante restaura a entrada antes
  do pós-check.
- B1 mantém o descendente no mesmo process group. Não confronta `setsid`/migração de
  grupo segurando stdout/stderr, que pode tornar `group_is_alive` falso antes dos joins.

Não há `SPEC-GAP`: os comportamentos exigidos já constam do L0. A estratégia de varrer
todo `.git/objects` também não se torna regra arquitetural; é correção parcial e não
autoriza declarar R4 fechado.

## Regressão final

- gates P0101: B1 4/4, B2 7/7, B3 3/3; B4 0/0 (`NOT RUN`);
- P0100: 7/7 + 4/4; Git histórico 6/6; CLI 10/10;
- suíte `cargo test --workspace`: `PASS`;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- reparador V5 dry-run: `Nothing to fix`;
- `rustfmt --check` dirigido e `git diff --check`: `PASS`.

Resultado: **F05 `BLOCKED`**. R1 e R3 são melhorias reais preserváveis, mas R2/R4/R5
impedem fechamento e integração deste branch.
