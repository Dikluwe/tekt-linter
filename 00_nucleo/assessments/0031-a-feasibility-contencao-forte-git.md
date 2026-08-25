# Assessment 0031-A — feasibility da contenção forte do Git

**Estado:** CONGELADO — A somente leitura; produção e gates não foram alterados  
**Data:** 2026-08-25  
**Passo:** P0102  
**Baseline funcional:** `c681c7a8fd419c48683553f88b6a3bf391f2032b`  
**Envelope observado:** `070172763587f0f94f1b05192ee84715bafb4d01`

## Integridade e fronteira

Os onze insumos L0 hash-pinned de P0102 foram recalculados antes desta análise e todos
coincidem com os valores declarados. O próprio passo possui SHA-256
`77407b9b97633292c149bc36f0aa2b0f23c138d5e0d9609d03999ed812b730a5`.
O worktree estava limpo na abertura. A leu os contratos, os fechamentos P0101, a
produção L3, o wiring consumidor, os gates P0101 e o manifesto de dependências. Não
editou produção nem gates.

O contrato fixa o efeito — objetos efetivamente acessados reais e internos, operação
limitada até leitores e contenção terminarem, process group Unix e Job Object Windows —
mas não autoriza automaticamente qualquer mecanismo privilegiado ou cópia do banco.
P0102 exige que escolhas inevitáveis desse tipo sejam tratadas como `SPEC-GAP`.

## Instantes de abertura e consumo do object database

O adapter atual não abre objetos. Ele entrega a raiz ao executável Git em três processos
independentes; logo os instantes materiais pertencem ao Git e não são observáveis pela
API Rust atual:

| Classe | Abertura/consumo material | Janela relevante |
|---|---|---|
| loose object | `rev-parse`, `ls-tree` ou `cat-file` resolve o OID, percorre `.git/objects/<fanout>/<resto>`, abre o arquivo e infla/verifica os bytes | cada lookup pode reabrir fanout/arquivo; o descritor aberto preserva o inode, mas não prova que o caminho resolvido no instante da abertura era interno |
| `.idx` | Git enumera/seleciona packs e abre/mapeia o índice para localizar OID e offset | enumeração, abertura e eventual reabertura/mmap não coincidem com scan do adapter |
| `.pack` | após consultar o índice, Git abre/mapeia o pack e consome header/deltas/bytes no offset | `.idx` e `.pack` são abertos separadamente; rename/troca pode produzir par de identidades diferente |

`validate_object_database` percorre nomes com `read_dir`, consulta
`symlink_metadata` e depois `canonicalize`. Essas próprias operações não formam uma
resolução atômica; cada componente pode mudar entre consultas. Os scans antes e depois
de cada subprocesso provam somente dois estados discretos. Uma fixture pode instalar um
symlink depois do primeiro scan, deixar Git abrir os bytes externos e restaurar o
arquivo regular antes do segundo. O pós-check verde não revoga bytes já lidos. Hardlink
também é arquivo regular sem symlink e não revela, por pathname/canonicalização, se o
inode tem origem/alias externo.

Consequentemente, nenhum número finito de pre/post-scans fecha R4. A garantia exige
controlar a abertura efetiva (backend que use handles relativos e `NOFOLLOW`), ou dar ao
Git uma visão imutável/confinada do banco durante toda a operação.

## Lifecycle causal: líder, sessão, pipes e reap

1. `Command::spawn` cria o líder. Em Unix, `process_group(0)` cria apenas um novo
   process group, na sessão existente.
2. stdin é escrito/fechado; stdout e stderr são retirados do `Child` e dois readers são
   iniciados.
3. O líder pode criar descendentes. Enquanto permanecem no grupo, `kill(-pgid,
   SIGKILL)` os alcança; `kill(-pgid, 0)` só prova membros ainda nesse grupo.
4. Um descendente pode executar `setsid`, adquirir nova sessão/grupo, fazer double-fork
   e sobreviver ao grupo original. Fechar os pipes remove o sintoma nos readers, mas não
   prova término. Mantê-los abertos faz `join` depender de EOF do fugitivo.
5. `Child::wait` coleta apenas o líder. Descendentes não são necessariamente filhos
   esperáveis pelo adapter; mesmo um subreaper precisa descobrir, sinalizar e coletar
   adotados, com política para concorrência e escopo do processo hospedeiro.
6. Hoje o deadline governa o loop do líder, mas os `join` normais são bloqueantes e, no
   caminho de timeout, também ocorrem depois da decisão sem watchdog próprio.

Portanto grupo, sessão, árvore de parentesco, pipes e readers são identidades distintas.
“Líder reaped”, “grupo vazio”, “EOF nos pipes” e “todos os descendentes encerrados” não
são provas intercambiáveis.

## APIs e dependências disponíveis

| Plataforma | Recursos presentes | Capacidade e limite |
|---|---|---|
| Rust comum | `std::process`, threads, channels e relógio monotônico; nenhuma dependência direta de OS em `Cargo.toml` | permite state machine/deadline dos readers; não oferece árvore de processos nem abertura pathname confinada |
| Unix genérico | `CommandExt::process_group`, FFI local mínima para `kill` | contém somente o grupo; não contém `setsid`; não permite reap de processos não filhos |
| Linux host | `/proc` e kernel Linux; cgroup v2, pidfd, namespaces e subreaper são mecanismos possíveis, não dependências/políticas aprovadas | cgroup pode conter escape de sessão, mas criação/migração/cleanup e permissões não são garantidos; namespace/subreaper altera semântica global e ainda requer desenho de enumeração/reap |
| macOS suportado pela matriz | process groups disponíveis; sem cgroup Linux e sem API portátil equivalente inventariada | uma solução Linux não fecha o contrato Unix/macOS |
| Windows | Win32 Job Objects e criação suspensa/atribuição atômica são mecanismos adequados; gate P0101 já usa FFI de observação | produção não tem FFI/dependência direta. `windows-sys` aparece apenas transitivamente no lock e não é API disponível ao crate sem dependência declarada |
| Runtime atual | targets instalados somente `x86_64-unknown-linux-gnu` e `x86_64-unknown-linux-musl`; `wine`/`wine64` ausentes | nenhuma implementação Windows pode obter `PASS` nesta execução |

No Windows, criar o processo normalmente e chamar `AssignProcessToJobObject` depois do
`spawn` deixa código hostil correr antes da associação. A implementação correta precisa
criação suspensa seguida de associação antes de `ResumeThread`, ou associação no ato da
criação via atributos suportados. Isso implica uma seam de spawn Win32, inventário de
handles/quoting/herança e fault injection privada. Raw FFI `cfg(windows)` evita uma nova
crate, mas não reduz esse custo nem substitui runtime real.

## Seam mínima L3 proposta

Sem mudar API pública, L1, L2 ou L4, `run_controlled` pode delegar a uma seam privada
L3 conceitual:

```text
Containment::spawn(command, fault_seam) -> RunningOperation
RunningOperation::poll(now) -> leader/readers/containment events
RunningOperation::terminate(cause)
RunningOperation::finish(deadline) -> status + bounded streams | containment error
```

O estado explícito deve distinguir `Prepared`, `SpawnedNotContained`, `Contained`,
`StdinClosed`, `ReadersRunning`, `LeaderExited`, `Terminating`, `ContainedEmpty`,
`ReadersDone` e `Closed`. Conteúdo só pode sair de `Closed`. A seam específica de SO
fica privada; framing, budgets e taxonomia continuam únicos acima dela.

Para R4, uma segunda seam privada pode representar uma `ObjectView` mantida viva pelos
três processos. Porém sua implementação não pode ser escolhida em C enquanto o L0 não
decidir entre acesso por backend, snapshot/staging ou sandbox filesystem. Alterar o
ambiente com `GIT_OBJECT_DIRECTORY`, executar processos adicionais ou copiar objetos
contraria, sem nova decisão, o ambiente exato e os três processos fixados pelo contrato.

## Matriz de cleanup e erro

| Ponto de falha | Cleanup mínimo | Resultado normativo |
|---|---|---|
| preparar contenção/job falha | fechar handles/recursos parciais; nenhum filho liberado | `ContainmentFailure` |
| spawn/atribuição falha | filho ainda suspenso deve ser terminado; fechar thread/process/job e pipes | `ContainmentFailure` se contenção incompleta; `ProcessFailure` somente se nenhum filho escapável existiu |
| escrita stdin/I/O de controle falha | fechar stdin, terminar contenção, limitar readers e coletar o que for coletável | `ProcessFailure` apenas após contenção comprovada; senão `ContainmentFailure` |
| framing/cap/budget decide cedo | descartar bytes, fechar pipes graváveis, terminar contenção e readers sob o mesmo deadline | classificação causal somente após cleanup comprovado; senão `ContainmentFailure` |
| deadline vence | fechar stdin, terminar toda a contenção, limitar/drain/fechar readers, coletar líder/membros | `Timeout` somente com encerramento comprovado; senão `ContainmentFailure` |
| líder sai antes dos descendentes | não fazer join ilimitado; continuar governando contenção/readers | sucesso/framing apenas após contenção vazia; incapacidade de provar vira `ContainmentFailure` |
| cleanup de object view falha | nenhuma publicação; remover staging/mount/handles conforme política futura | `ContainmentFailure` |

## Normativo versus decisões novas

Já é normativo: seam pública e taxonomia; L3 dona de processo/filesystem; três comandos
e ambiente exato; 10 segundos por operação; readers dentro da operação; Job Object com
kill-on-close no Windows; falha fechada sem publicação; objetos efetivamente acessados
reais, internos e sem symlink; R1/R3 sem regressão.

Decisões novas inevitáveis:

- **SG-1 — contenção Unix após `setsid`: `SPEC-GAP`.** É necessário escolher suporte e
  política entre cgroup, namespace/subreaper, mecanismo específico por SO ou redução
  explícita de plataformas. Isso envolve privilégio, disponibilidade, efeitos no
  processo hospedeiro e cleanup. Process group sozinho é insuficiente.
- **SG-2 — visão forte do object database: `SPEC-GAP`.** É necessário escolher entre
  staging/snapshot, sandbox/mount/namespace, ou substituir o acesso do Git por backend
  controlado. A escolha muda I/O, budget, número/ambiente de processos, atomicidade,
  hardlinks e portabilidade; P0102 proíbe selecioná-la silenciosamente.
- **Windows Job: não é `SPEC-GAP` quanto ao efeito**, pois o L0 já exige Job Object.
  FFI/handles precisam do inventário acima e de implementação privada L3. A evidência
  continua `BLOCKED` por ausência de runtime Windows real.
- **Deadline dos readers: não é `SPEC-GAP`.** Pode ser implementado com recursos Rust
  presentes, desde que threads bloqueadas não sejam abandonadas como falso cleanup.

## Parecer A

**Contenção forte integral não é implementável com os recursos e decisões atualmente
autorizados.** A state machine de readers e uma implementação Windows podem ser
materializadas parcialmente; Windows não pode ser validado aqui. R4 e R5/Unix exigem
resolver SG-2 e SG-1 antes de C. Logo A autoriza somente a materialização segregada dos
gates B1–B4 para congelar evidência, mas **C permanece `BLOCKED`** e F05 não pode fechar
nem ser integrado neste passo sem decisão L0 separada.
