# Passo operacional 0102 — contenção forte do Git e fechamento de F05

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch:** continuar `codex/audit-git-refinement-functional`
> **Baseline:** `c681c7a8fd419c48683553f88b6a3bf391f2032b`
> **Predecessor:** P0101 `BLOCKED`
> **Lote do backlog:** F05; nenhum novo componente

## Objetivo

Fechar somente os três bloqueios materiais restantes de F05, sem reabrir R1/R3:

1. R4: impedir que Git consuma objeto externo numa troca transitória que seja restaurada
   antes do pós-check;
2. R5: manter deadline e encerramento mesmo quando descendente executa `setsid`, muda de
   process group e conserva ou fecha os pipes herdados;
3. R2/R5 Windows: implementar Job Object, falhar fechado em criação/atribuição e provar
   lifecycle/handles em runtime Windows real.

P0102 não aceita varredura pathname antes/depois como prova de contenção, process group
como contenção integral nem `cfg(windows)`/cross-compile como evidência de runtime.

## Condições de entrada e saída

- worktree limpo no baseline exato acima;
- P0101 e seus gates permanecem imutáveis;
- gates novos são congelados em RED antes de reabrir produção;
- nenhuma mudança de L0, budget, plataforma suportada ou dependência ocorre por
  conveniência da implementação;
- F05 fecha somente se R1–R5 estiverem `PASS` no consumidor e runtime corretos;
- ausência de runtime Windows encerra P0102 como `BLOCKED`, sem merge/push;
- resultado permitido: `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Insumos L0 hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| Assessment P0101 | `00_nucleo/assessments/0030-fechamento-reds-git-refinement.md` | `c58082915d0d02576c3664d9c1e9757dc43448d173f92c7ffe7d442a478f35fc` |
| relatório P0101 | `00_nucleo/relatorio-p0101-fechamento-reds-git-refinement.md` | `798d67554ddb559447c2473c7f6fb5b98ce9cbc852320e31b544db3d346ab36d` |
| contrato Git | `00_nucleo/prompts/refinement-validator.md` | `9ab972915e8f21e6c0fc323686d507fb2cb4b590de6d987b454e05642f167818` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| produção L3 | `03_infra/git_refinement.rs` | `42bab723efa948b3025a70154d2087493d7104fa9186ba02fc5347e6a4614d65` |
| wiring L4 | `04_wiring/main.rs` | `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` |
| gate lifecycle P0101 | `tests/git_refinement_stream_lifecycle_assessment.rs` | `c946032e31c083d051705f4bfe3ff66c8d03d5894822a99cb47ab8fe7af615f0` |
| gate objetos P0101 | `tests/git_refinement_object_containment_assessment.rs` | `8d612adac31fc168b2904b4ef32c82f34573a6e753780edf9e9a8a35e4a33925` |
| gate Windows P0101 | `tests/git_refinement_windows_job_assessment.rs` | `7c1541991e8b303767c3d5c0e1b8c1f89599cb4e0cd97775713bc8feed59fc35` |

O Assessment 0031 deve recalcular estes hashes antes de qualquer gate. Divergência é
`BLOCKED`, não autorização para resselar automaticamente.

## Fronteiras arquiteturais Tekt

- L1 permanece puro e recebe somente fatos/valores projetados.
- L2 continua dona de apresentação, precedência e códigos de saída.
- L3 é dona de processos, handles, deadlines, filesystem, object store, framing e
  projeção `GitRevisionContent` → `ArtifactFacts`.
- L4 conserva apenas seleção do executável e composição da rota publicada.
- mecanismo específico de SO fica atrás de uma seam privada L3; não duplicar parsing,
  budgets ou taxonomia entre Unix e Windows.
- nova dependência, cópia/staging de object database, cgroup, namespace, pidfd,
  subreaper ou FFI adicional exige inventário de portabilidade, cleanup e custo antes da
  implementação. Decisão não coberta pelo L0 é `SPEC-GAP` e pausa C.

## Protocolo segregado

### A — feasibility e mapa causal

A é somente leitura e produz
`00_nucleo/assessments/0031-a-feasibility-contencao-forte-git.md`:

- enumera o instante exato de abertura/consumo de loose object, `.pack` e `.idx`;
- demonstra por que pre/post-scan não fecha a janela;
- enumera líder, grupo, sessão, descendentes, pipes, readers e operações de reap;
- verifica APIs já disponíveis por plataforma e dependências presentes;
- separa requisito já normativo de decisão nova inevitável;
- propõe seam mínima L3 e matriz de cleanup/erro, sem editar produção.

A deve responder explicitamente se contenção forte pode ser implementada com os recursos
atuais. Se exigir nova política de budget, staging, privilégio ou plataforma, congela
`SPEC-GAP`; não escolhe silenciosamente.

### B1 — gate TOCTOU transitório

B1 cria exclusivamente `tests/git_refinement_transient_object_race_assessment.rs` e
fixtures próprias. O estímulo deve sincronizar três fases observáveis:

1. objeto/pack interno regular no preflight;
2. troca por symlink externo exatamente durante a abertura/leitura pelo Git;
3. restauração do arquivo interno antes do retorno do subprocesso e do pós-check.

O gate cobre loose object, fanout e par `.pack`/`.idx`; usa bytes-sentinela externos e
prova que a troca e a restauração realmente ocorreram. Resultado aceito:
`ContainmentFailure` antes de qualquer publicação. Um gate que apenas deixa o symlink
instalado é `GATE-DEFECT`.

Controles positivos usam objetos internos regulares. O gate não transforma a varredura
completa do banco em requisito.

### B2 — gate de escape de sessão e readers

B2 cria exclusivamente `tests/git_refinement_session_escape_assessment.rs` e fixtures
próprias Unix. Deve confrontar separadamente:

- descendente executa `setsid`, mantém stdout/stderr abertos e líder termina;
- descendente executa `setsid`, fecha pipes e continua vivo;
- cadeia de dois descendentes em que o intermediário termina;
- escape ocorrido antes e durante o timeout;
- retorno dentro de watchdog externo de 15 segundos, readers concluídos e nenhum PID da
  fixture vivo.

PIDs devem ser publicados pela fixture e verificados por identidade, evitando matar
processo alheio por reutilização. Cleanup defensivo pertence à fixture, mas sua execução
não pode ser confundida com sucesso do adapter.

### B3 — gate Windows Job Object v2

B3 substitui o gate insuficiente somente após registrar `GATE-DEFECT`; cria
`tests/git_refinement_windows_job_v2_assessment.rs`. Deve rodar em Windows real e provar:

- Job criado e configurado com kill-on-close antes do código hostil;
- líder e descendentes associados antes de publicar conteúdo;
- timeout e saída antecipada do líder encerram toda a árvore;
- descendente sem pipes também é contido;
- handles retornam ao baseline após repetição, dentro de tolerância congelada;
- falhas injetadas de criação, configuração e atribuição retornam
  `ContainmentFailure`, sem bytes publicados;
- watchdog externo limita cada cenário.

Fault injection deve ficar numa seam privada/testável L3, não numa variável de ambiente
produtiva. Cross-compile é somente regressão auxiliar.

### B4 — regressão de R1/R3

B4 não cria expectativa nova. Executa os gates P0101 de rota, protocolo e overflow e
congela seus resultados antes de C. Qualquer regressão é `RED` próprio e impede a
correção dos três bloqueios.

### C — implementação por dependência

Somente após A e B1–B4 congelados:

1. introduzir uma seam privada de contenção/lifecycle L3 com estado explícito para
   spawn, associação, deadline, readers, término e cleanup;
2. implementar contenção Unix que continue responsável por descendentes após
   `setsid`/mudança de grupo, ou bloquear se isso exigir política não autorizada;
3. garantir que timeout governe também conclusão dos readers — nenhum `join` bloqueante
   pode ocorrer fora do mecanismo de deadline;
4. substituir pre/post-scan por acesso/isolamento que impeça o Git de abrir bytes fora da
   raiz durante toda a operação; preservar identidade do arquivo efetivamente lido;
5. implementar Job Object Windows e fault injection privada;
6. remover a varredura recursiva parcial somente quando o mecanismo forte a tornar
   redundante e os controles positivos permanecerem verdes.

Commits de produção devem citar R2, R4 ou R5 e o gate causal correspondente. Não alterar
gates congelados para acomodar a implementação.

### D — adversário final

D é somente leitura e tenta invalidar cada fechamento:

- troca e restaura symlink entre todos os checkpoints;
- busca hardlink, rename, fanout, pack/idx desencontrado e descritor reaberto;
- faz `setsid`, double-fork, fecha pipes e mantém processo vivo;
- verifica deadline dos readers independentemente do estado do líder;
- diferencia kill, wait do líder e encerramento de toda a contenção;
- no Windows, mede handles e injeta falha em cada API Job;
- confirma que R1/R3 e arquitetura Tekt não regrediram;
- rejeita teste pulado, cross-compile ou cleanup da fixture como `PASS`.

## Matriz de fechamento

| RED | Gate | Fechamento mínimo |
|---|---|---|
| R1 | B4/P0101 B3 | continua 3/3 e rota única |
| R2 | B3 Windows v2 | Job/árvore/handles/faults `PASS` em Windows real |
| R3 | B4/P0101 B1 | overflow incremental continua `PASS` |
| R4 | B1 | troca transitória restaurada nunca fornece bytes externos |
| R5 | B2 + B3 | `setsid`/double-fork e árvore Windows encerrados; readers limitados |

F05 fecha somente com as cinco linhas `PASS`. `GATE-DEFECT` precisa ser corrigido e
reconfrontado por evidência nova; `SPEC-GAP` precisa de decisão L0 separada antes de C.

## Regressões obrigatórias

- B1–B4 P0102;
- B1–B4 P0101, mantendo B4 antigo identificado como evidência histórica insuficiente;
- P0100 protocolo 7/7 e timeout 4/4;
- Git histórico 6/6 e CLI refinement 10/10;
- loader, extractor e comparador de refinamento;
- suíte completa do workspace;
- V5/V6/V7/V12 e reparador V5 dry-run;
- `rustfmt --check` somente nos Rust tocados e `git diff --check`.

## Saídas esperadas

- `00_nucleo/assessments/0031-contencao-forte-git.md`;
- mapa feasibility A;
- gates B1–B3 e registro B4;
- RED inicial reproduzível e hash-pinned;
- correções mínimas L3/L4, se autorizadas;
- matriz R1–R5 final;
- `00_nucleo/relatorio-p0102-contencao-forte-git.md`;
- F05 `CLOSED` ou `BLOCKED`.

P0102 não autoriza merge, push, release, redução de suporte Windows, mudança de exits/F09,
pipeline amplo/F08, schema/writer F01–F03, comparador L1 ou novo backend. Integração só
pode ser proposta depois de D fechar R1–R5 e o relatório registrar
`READY WITH RESIDUAL AUDIT`.
