# Passo operacional 0101 — fechamento dos REDs funcionais Git

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch:** continuar `codex/audit-git-refinement-functional`
> **Baseline:** `ba6f3a1c6cf0142ff44075fce6cd903a5f3d1dcf`
> **Predecessor:** P0100 `BLOCKED`
> **Lote do backlog:** retomada de F05; nenhum novo lote

## Objetivo

Fechar os cinco REDs materiais que impediram P0100/F05:

1. integrar o caminho produtivo de `refine-revisions` à mesma seam
   `load_revision_with_git` confrontada pelos gates;
2. abortar incrementalmente blob declarado acima do budget, mesmo que stdout permaneça
   aberto;
3. manter deadline e contenção até EOF/reap quando o líder termina e descendente conserva
   pipes;
4. impedir symlink em loose objects e packs efetivamente acessíveis;
5. implementar e provar contenção de descendentes por Job Object no Windows.

P0101 começa por gates adicionais congelados. A produção do commit `bc6cdb8` é evidência
somente leitura até esses gates registrarem RED. O passo não reescreve os onze gates
verdes anteriores e não reduz o contrato para acomodar a implementação parcial.

## Condição de execução

Este passo continua no branch bloqueado; não integra P0100 em `master` antes do
fechamento. O baseline deve ser exatamente o commit documental `ba6f3a1`. Qualquer delta
posterior precisa ser inventariado e classificado antes dos gates.

Se Windows não puder ser executado ou emulado com evidência proporcional, o resultado
correto é `BLOCKED` com F05 ainda aberto. Compilação condicionada por `cfg(windows)`,
inspeção textual ou teste Unix não provam Job Object.

## Insumos hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| Assessment P0100 | `00_nucleo/assessments/0029-auditoria-funcional-git-refinement.md` | `9d643bc5a8c887d7ab328879fcd989558ecf5b40de40e9f847280c80a4d7cf15` |
| relatório P0100 | `00_nucleo/relatorio-p0100-auditoria-funcional-git-refinement.md` | `a69e6a72caff573f3666d61ab39477c3c72fdaef2500e18155beea603792f33f` |
| contrato Git saneado | `00_nucleo/prompts/refinement-validator.md` | `9ab972915e8f21e6c0fc323686d507fb2cb4b590de6d987b454e05642f167818` |
| ADR B2 saneado | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `cdd1acfe688aabd0c2bb0b7061a55c80dc47f1d7745c8a5c2e7f7f560115485f` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| produção parcial | `03_infra/git_refinement.rs` | `db50933e6976913a2ce0c1acb9883faf2efbab3e41135437440640027c51ef6b` |
| gate B1 final | `tests/git_refinement_protocol_assessment.rs` | `89bdaa09f3a1e3dff7cf30be71630f1cfafbe4b0d314d74e3faa607858c41eb0` |
| gate B2 final | `tests/git_refinement_timeout_assessment.rs` | `076106ff4c868165634661d720b2f6e9b71851126d5f4022e1b231d9ec69c442` |
| gate Git histórico | `tests/git_refinement_assessment.rs` | `9609ebdb84d21fb79cddd744392d9fb8692513c809bf651c52eefa1c8b75c434` |
| CLI histórica | `tests/refinement_cli.rs` | `641dbd3088710efc77d9e209a613236b59b819d565af7a2ee8fd80346d386408` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

O Assessment 0030 deve recalcular todos os hashes. Os gates P0100 permanecem congelados;
alterá-los exige `GATE-DEFECT` novo, causal e adversarialmente aceito.

## Fronteiras arquiteturais

- L1 permanece puro e não conhece Git, processos, handles ou Job Objects.
- L2/F09 continua dona de apresentação, códigos e precedência.
- L3 possui subprocesso, transcript incremental, filesystem/object database e projeção
  `GitRevisionContent`.
- L4 pode receber somente a ligação mínima para fazer o comando vigente consumir a seam
  única; política, exit e fluxo dos demais comandos não mudam.

Duplicar parser, budgets, ambiente ou taxonomia entre adapter antigo e novo é `RED`,
mesmo que ambos passem fixtures separadas.

## Protocolo segregado

### A — inventário causal dos cinco REDs

A lê P0100, L0, produção parcial e wiring somente para localizar cada fluxo. Produz
`00_nucleo/assessments/0030-a-mapa-causal-reds-git.md` com:

- símbolo/linha produtor e consumidor de cada RED;
- estado e efeito observável;
- seam mínima autorizada para correção;
- dependências entre REDs;
- pontos em que gate novo precisa atravessar L3 ou L4.

A não edita produção nem propõe expectativa diferente do L0.

### B1 — overflow incremental e lifecycle pós-líder

B1 cria exclusivamente `tests/git_refinement_stream_lifecycle_assessment.rs` e fixtures
próprias. Deve cobrir:

1. `cat-file` escreve header `blob 4194305`, mantém stdout aberto e não envia payload;
   adapter retorna `BudgetExhausted` antes de 10 segundos, encerra/reap o grupo e não
   publica bytes;
2. líder escreve framing parcial, cria descendente que mantém stdout/stderr abertos e
   termina; adapter conserva deadline, mata/reap a contenção e retorna falha tipada sem
   bloquear joins;
3. descendente fecha/redireciona pipes antes do timeout; o resultado ainda não pode ser
   publicado até a contenção provar encerramento dos membros conhecidos pelo mecanismo;
4. cap de transcript excedido com pipe aberto termina como `InvalidFraming`, não timeout.

Watchdog externo máximo de 15 segundos; PIDs pertencem somente à fixture. O gate não
mede exit CLI.

### B2 — autocontenção por objeto acessível

B2 cria exclusivamente `tests/git_refinement_object_containment_assessment.rs`, usando
repositórios temporários próprios. Deve confrontar ao menos:

- loose object solicitado substituído por symlink externo;
- arquivo `.pack` ou `.idx` acessível substituído por symlink externo;
- diretório fanout de loose object como symlink;
- pack/info interno regular como controle positivo;
- troca concorrente regular→symlink entre preflight e leitura, que deve falhar fechada.

O gate não exige varrer todo object database por princípio; exige provar que nenhum
objeto/pack efetivamente acessado escapa da raiz autorizada. Git real é estímulo aqui,
mas estado externo e bytes-sentinela tornam o oráculo independente.

### B3 — rota produtiva única

B3 cria exclusivamente `tests/git_refinement_productive_route_assessment.rs` e fixture
Git controlada separada. Exercita o consumidor real de `refine-revisions` até a seam L3,
sem usar códigos de exit como expectativa. Deve provar:

- o executável controlado observa exatamente o transcript da seam nova;
- cada ref é resolvida uma vez e somente OIDs chegam aos passos seguintes;
- falha de framing/contenção da seam impede extrator/comparador;
- não ocorre chamada aos helpers/processos históricos paralelos;
- os mesmos bytes produzem os mesmos `ArtifactFacts` que a rota B1 já fechada.

Se a API pública não permitir observar a rota sem decidir F09, A deve autorizar um spy
L3 mínimo. Alterar parsing CLI ou política de exit é proibido.

### B4 — contenção Windows

B4 cria gate Windows próprio, não um assert condicionado em Linux. Deve executar em host
Windows ou ambiente com semântica real de Job Object:

- processo líder e descendente são associados ao mesmo Job antes de código hostil;
- `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` ou propriedade equivalente é observada;
- timeout encerra líder e descendente;
- líder encerrado não deixa descendente vivo;
- handles são fechados e o chamador retorna dentro do watchdog;
- falha ao criar/atribuir Job retorna `ContainmentFailure` antes de publicar conteúdo.

O gate pode usar FFI `cfg(windows)` ou dependência já presente. Nova dependência exige
decisão L0 e aprovação antes de alterar `Cargo.toml`. Cross-compile serve apenas como
checagem auxiliar; runtime Windows é obrigatório para fechar este RED.

### C — correção por dependência

Somente após B1–B4 congelados e REDs registrados:

1. refatorar `run_controlled` para que deadline cubra líder, leitores e contenção até
   conclusão total;
2. parsear molduras/caps incrementalmente e abortar oversize sem esperar EOF;
3. implementar contenção Unix que não alegue reap não comprovado;
4. implementar Job Object Windows e tratamento de handles;
5. proteger o objeto/pack efetivamente lido contra symlink/escape e troca concorrente;
6. projetar `GitRevisionContent` no extrator existente e ligar a rota produtiva à seam
   única, removendo ou desativando o caminho paralelo.

Ordem pode mudar por dependência demonstrada, mas cada commit deve referenciar RED e
gate. Não alterar L0, budgets ou expectations depois de abrir produção.

### D — adversário final

D verifica cada RED isoladamente e a composição conjunta. Deve buscar:

- processo/reader fora do deadline;
- zombie ou descendente vivo sem pipe;
- `kill` confundido com reap integral;
- TOCTOU ou symlink não observado em object store;
- rota CLI ainda alcançando helper antigo;
- gates Unix apresentados como prova Windows;
- fallback que converte falha em `Missing`/`Unknown`/sucesso indevido;
- nova dependência ou decisão em L1/L2/L4 fora da seam autorizada.

## REDs congelados e critérios de fechamento

| RED | Gate obrigatório | Fechamento |
|---|---|---|
| R1 rota produtiva paralela | B3 | zero chamada ao adapter histórico; mesma seam e fatos |
| R2 Windows sem Job Object | B4 runtime Windows | líder/descendente encerrados e handles fechados |
| R3 oversized espera EOF | B1 | budget antes do deadline com pipe aberto |
| R4 symlink objeto/pack | B2 | nenhum byte externo chega ao Git/extrator |
| R5 líder sai/pipes/reap | B1 + B4 | retorno limitado, contenção encerrada, sem processo vivo |

F05 fecha somente com cinco linhas `PASS`. Não aceitar R2 como residual: suporte Windows
foi publicado. Se infraestrutura Windows não estiver disponível, P0101 termina
`BLOCKED`, ainda que R1/R3/R4/R5 fiquem verdes.

## Regressões obrigatórias

- B1–B4 novos;
- B1/B2 P0100: 7/7 + 4/4;
- gate Git histórico: 6/6;
- CLI refinement: 10/10;
- loader/extractor/comparador de refinamento;
- suíte completa do workspace;
- V5/V6/V7/V12 e reparador V5 dry-run;
- `rustfmt --check` somente nos Rust tocados e `git diff --check`.

## Classificações

- `RED`: qualquer um dos cinco comportamentos contradiz L0;
- `SPEC-GAP`: somente nova decisão inevitável não coberta pelo L0 vigente;
- `GATE-DEFECT`: cenário não é realmente exercitado, processo vaza, oráculo depende da
  implementação ou plataforma errada é usada como prova;
- `PASS`: RED confrontado no consumidor/plataforma corretos.

Fechamento somente `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Saídas esperadas

- Assessment 0030 e mapa causal A;
- quatro gates B1–B4 e fixtures segregadas;
- RED inicial reproduzível por gate;
- correções mínimas L3 e ligação L4 estritamente necessária;
- matriz R1–R5 final;
- `00_nucleo/relatorio-p0101-fechamento-reds-git-refinement.md`;
- F05 `CLOSED` ou `BLOCKED`.

P0101 não autoriza merge do branch bloqueado, push, release, mudança de exits/F09,
pipeline amplo/F08, schema/writer F01–F03, comparador L1, novo backend ou enfraquecimento
do suporte Windows. Integração em `master` só pode ser proposta depois de D fechar F05.
