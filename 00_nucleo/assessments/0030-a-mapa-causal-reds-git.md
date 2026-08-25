# Assessment 0030-A — mapa causal dos REDs Git

**Estado:** CONGELADO — somente leitura; produção e gates ainda proibidos  
**Data:** 2026-08-25  
**Passo:** P0101 / retomada de F05  
**Baseline funcional observado:** `ba6f3a1c6cf0142ff44075fce6cd903a5f3d1dcf`  
**Envelope operacional observado:** `d7da297`

## Fronteira e método

Este mapa leu P0101, Assessment/relatório P0100, o L0 resselado, ADR-0019,
`03_infra/git_refinement.rs` e somente o wiring consumidor em `04_wiring/main.rs`.
Não altera expectativa do L0, produção ou testes. Linhas abaixo referem-se ao baseline
observado e servem para localizar símbolos; o símbolo e o efeito são a identidade causal
primária caso uma edição posterior desloque linhas.

O contrato vigente concentra Git, subprocessos, filesystem, budgets e tipos
`GitRevision*` em L3. L4 pode apenas compor conteúdo/identidade com o extrator e o
comparador existentes. L1 permanece puro; apresentação, precedência e exit permanecem
reservados a L2/F09.

## Mapa R1–R5

| RED | Produtor e estado atual | Consumidor e efeito observável | Seam mínima autorizada | Gate que precisa atravessar |
|---|---|---|---|---|
| R1 — rota produtiva paralela | A seam nova é `load_revision_with_git` em `03_infra/git_refinement.rs:324–494`, mas não possui consumidor produtivo. Em paralelo, `git_command`/`run`/`resolve_commit`/`tree_entries`/`read_blobs`/`extract_revision_snapshot` permanecem em `03_infra/git_refinement.rs:504–903`. | O ramo real `RefineRevisions` importa somente os helpers históricos (`04_wiring/main.rs:37–39`), resolve cada ref por `resolve_commit` (`312–320`) e extrai pelos helpers paralelos (`321–330`). Assim argv, ambiente, framing, budgets e contenção confrontados em P0100 não governam o comando publicado; os fatos podem divergir da seam auditada. | Em L3, projetar `GitRevisionContent.paths` no `extract_snapshot_from_content` já compartilhado, preservando `oid`, `Missing` e razões tipadas. Em L4, ligação mínima para resolver uma vez o path absoluto do Git e chamar a mesma seam para cada ref; remover/desativar alcance produtivo do adapter histórico. Se observabilidade exigir, um spy mínimo em L3 pode expor chamadas sem decidir parsing ou exit. | B3 atravessa o consumidor real L4 até L3. Deve observar transcript da seam, contagem de resolução, bloqueio do extrator/comparador em falha e equivalência de `ArtifactFacts`; não usa exit como oráculo. |
| R2 — Windows sem Job Object | `isolate` e `terminate_group` têm implementação real apenas em Unix (`03_infra/git_refinement.rs:100–124`). A variante `cfg(not(unix))` não isola (`106–107`) e mata somente o líder com `Child::kill` (`126–131`). | `run_controlled` chama essa contenção antes/depois do spawn (`133–219`). Em Windows, descendentes podem sobreviver ao timeout ou ao líder, e não há handle/job que permita comprovar encerramento integral. Publicar erro ou conteúdo após contenção incompleta viola a taxonomia/lifecycle. | Variante L3 Windows que crie Job Object, configure encerramento de membros no close, associe o processo antes de código hostil, retenha/feche handles e converta falha de criação/atribuição/encerramento em `ContainmentFailure`. Nenhuma decisão pode subir a L1/L2/L4. | B4 precisa executar em runtime Windows com semântica real de Job Object e observar líder, descendente, handles, watchdog e falha de associação. `cfg`, inspeção ou cross-compile são apenas auxiliares. |
| R3 — oversized espera EOF | `run_controlled` acumula stdout até cap genérico e só entrega `Output` após status, EOF e joins (`03_infra/git_refinement.rs:155–240`). O parser de `cat-file` examina o header e detecta `size > MAX_BLOB_BYTES` apenas depois do retorno integral (`425–463`). | Um `cat-file` que escreve `blob 4194305` e mantém stdout aberto não deixa o parser ver a moldura. O watchdog chega a 10 s e retorna `Timeout`; não retorna `Unknown(BudgetExhausted)` imediatamente e pode reter bytes intermediários. | Em L3, tornar a leitura/parser de `cat-file` incremental e comunicar a classificação de moldura ao controlador enquanto o processo vive. Ao detectar size acima do budget, descartar todos os blobs, encerrar/reap a contenção e produzir a projeção `BudgetExhausted` sem esperar payload/EOF. Caps continuam sendo `InvalidFraming`. | B1 atravessa `load_revision_with_git` em L3 com header oversized e pipe aberto; observa classificação antes de 10 s, descarte integral e encerramento/reap. O cenário de cap excedido com pipe aberto distingue `InvalidFraming` de timeout. |
| R4 — symlink em objeto/pack acessível | `validate_public_inputs` valida `.git`, `objects`, `objects/info` e `objects/pack` como diretórios (`03_infra/git_refinement.rs:242–291`), mas não valida fanouts, loose objects, `.pack`/`.idx` nem troca após preflight. O adapter então entrega o repository ao Git (`333–427`). | Git pode abrir bytes através de symlink externo ou após troca regular→symlink entre preflight e leitura. Esses bytes podem alcançar `ls-tree`, `cat-file`, `GitPathContent::Blob` e o extrator, apesar do envelope autocontido. | L3 deve proteger cada objeto/pack efetivamente acessível no fluxo contra symlink/escape e TOCTOU, mantendo todos os componentes reais sob `.git/objects`. A correção não autoriza varredura como nova regra nem backend adicional; falha fecha como `ContainmentFailure` antes de qualquer publicação. | B2 atravessa preflight L3 e Git real como estímulo, com sentinela externa como oráculo independente. Cobre loose object, fanout, pack/idx, controle regular e troca concorrente. Nenhum byte-sentinela pode chegar ao Git/extrator. |
| R5 — líder sai, pipes e reap ficam fora do deadline | O loop de `run_controlled` termina assim que `child.try_wait()` retorna status (`03_infra/git_refinement.rs:198–223`). Depois disso, `stdout_reader.join()` e `stderr_reader.join()` não têm deadline (`224–231`). No timeout, mata o grupo, mas espera apenas o líder (`212–219`); `kill` não comprova reap/encerramento de todos os membros. | Descendente que herda stdout/stderr pode manter os leitores bloqueados após a saída do líder. O chamador espera sem limite; ou retorna sem prova de que membros sem pipes morreram. Isso viola a definição de operação até drenagem/status/contenção encerrada. | Refatorar o controlador L3 para que o mesmo deadline cubra líder, leitores e contenção até conclusão total; não fazer joins ilimitados. Encerrar grupo/job e comprovar o máximo permitido pelo mecanismo, retornando `ContainmentFailure` quando reap/encerramento não puder ser provado. A implementação Unix não pode alegar reap de descendente que o SO não permite ao processo coletar. | B1 confronta líder encerrado, pipes herdados/redirecionados e retorno limitado em Unix. B4 confronta a mesma propriedade com Job Object em Windows, inclusive descendente sem pipe. Os dois gates são necessários para o fechamento da linha. |

## Pontos causais e dependências

1. **R3 e R5 compartilham o controlador.** A refatoração de `run_controlled` é a
   dependência estrutural de ambos: eventos de stdout precisam alcançar a decisão antes
   de EOF, e a decisão precisa manter deadline até leitores e contenção terminarem.
   Corrigir somente o parser pós-`Output` não fecha R3; adicionar somente outro timeout
   ao líder não fecha R5.
2. **R2 é a variante Windows do mesmo lifecycle, mas exige prova própria.** O contrato
   comum pode ser modelado na L3, porém o mecanismo Unix não é oráculo para Job Object.
   R5 pode ficar verde no Unix e continuar bloqueado no Windows.
3. **R4 precede qualquer processo.** A contenção do object database deve fechar antes de
   `controlled_command`; não depende da projeção de fatos de R1. A troca concorrente
   impede tratar uma simples varredura antecipada como prova suficiente.
4. **R1 deve consumir R2–R5, não duplicá-los.** Ligar o comando produtivo antes do
   fechamento interno permite reproduzir os REDs via L4, mas o fechamento de R1 exige
   que sua única rota herde a seam final. Não é autorizado copiar budgets, ambiente,
   parsing ou taxonomia para os helpers históricos.
5. **Projeção L3→extrator é dependência de B3.** `extract_snapshot_from_content`, já
   usado em `extract_revision_snapshot` (`03_infra/git_refinement.rs:871–893`), é o ponto
   de convergência para provar os mesmos `ArtifactFacts`. `GitPathContent::Missing`
   fornece ausência; `MissingObject`/`ForbiddenObjectKind` projetam
   `Unknown(PartialContract)` e `BudgetExhausted` projeta
   `Unknown(BudgetExhausted)`, sem criar semântica Git em L1.

## Plano de observação segregada dos gates

### B1 — `git_refinement_stream_lifecycle_assessment.rs`

- entra exclusivamente pela API pública `load_revision_with_git`;
- fixture controla o terceiro processo `cat-file`, PIDs e pipes;
- observa tempo limitado, variante tipada, ausência de bytes publicados e término dos
  PIDs da fixture;
- diferencia header oversized (`BudgetExhausted`) de cap de transcript
  (`InvalidFraming`), e líder encerrado com pipe herdado de término normal;
- não observa apresentação ou exit CLI.

### B2 — `git_refinement_object_containment_assessment.rs`

- monta repositório temporário autocontido e altera somente objetos/packs próprios;
- atravessa `validate_public_inputs` e a leitura Git efetiva da seam L3;
- usa sentinela externa/estado do filesystem como oráculo, não mensagens do Git;
- sincroniza a troca concorrente para provar falha fechada entre preflight e acesso;
- não transforma Git real no único oráculo e não exige uma varredura global por
  princípio.

### B3 — `git_refinement_productive_route_assessment.rs`

- atravessa `RefinementCommand::RefineRevisions` no consumidor L4 e a seam L3 real;
- executável controlado registra cada argv/transcript e torna chamadas históricas
  paralelas detectáveis;
- spy mínimo, se indispensável, pertence a L3 e observa chamadas/entrega ao extrator;
- compara `ArtifactFacts` com a rota de conteúdo B1 já fechada e prova que falha L3
  impede extração/comparação;
- parsing, precedência, texto e exit permanecem opacos para não antecipar F09.

### B4 — gate Windows

- deve ser arquivo/gate próprio executado em host Windows ou ambiente com Job Object
  real;
- fixture cria líder e descendente e permite observar associação antes do código hostil,
  kill-on-close, timeout, handles e retorno no watchdog;
- falha induzida de criação/atribuição deve resultar em `ContainmentFailure` sem conteúdo;
- ausência de runtime Windows é bloqueio, não skip convertido em `PASS`.

## Disponibilidade de plataforma

O host observado é Linux. `rustup target list --installed` contém somente
`x86_64-unknown-linux-gnu` e `x86_64-unknown-linux-musl`; `wine` e `wine64` não estão no
`PATH`. Conforme a restrição do passo, não foi tentada instalação, download ou emulação.
Logo B4 runtime Windows é impossível nesta execução e R2 permanece bloqueante mesmo que
uma implementação `cfg(windows)` ou checagem cruzada auxiliar venha a existir.

## Classificação de A

| RED | Classificação causal | Gate |
|---|---|---|
| R1 | `RED` confirmado | B3 |
| R2 | `RED` confirmado; prova runtime `BLOCKED` neste host | B4 |
| R3 | `RED` confirmado | B1 |
| R4 | `RED` confirmado | B2 |
| R5 | `RED` confirmado | B1 + B4 |

Não foi encontrado `SPEC-GAP`: o L0 vigente define seams, budgets, taxonomia,
autocontenção e lifecycle suficientes. A também não classifica `GATE-DEFECT`, pois os
novos gates ainda não foram escritos. **Parecer A: `PASS` para abertura segregada de
B1–B4, com bloqueio de fechamento já conhecido para B4/R2 enquanto não houver runtime
Windows proporcional.** Produção permanece proibida até os gates serem congelados e os
REDs iniciais registrados.
