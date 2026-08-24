# Assessment adversarial — leitura imutável de revisões Git

## Escopo

Assessment retroativo do alvo `03_infra/git_refinement.rs`, sob as alegações congeladas
em `00_nucleo/assessments/0001-git-refinement.md` e a etapa B2 do prompt de
refinamento. Não atribui segregação histórica e não avalia testes existentes.

Os ataques abaixo devem ser executados contra cópias descartáveis de repositórios. Um
resultado adversarial é reproduzível quando fixture, comando, limite de tempo e estado
antes/depois são registrados sem depender de rede pública.

## Achados prioritários

### P0 — pathspec magic é aceito depois de `--`

`ls-tree -rz <oid> -- <path>` protege contra opções, mas `--` não desliga a linguagem
de pathspec. Um caminho de contrato como `:(glob)**/*.rs`, `:(icase)FILE.rs` ou
`:(exclude)...` pode selecionar entradas diferentes do caminho lógico declarado.

Ataque reproduzível:

1. Criar commit com `safe.rs` e `nested/secret.rs`, com conteúdo semanticamente
   distinto para a query.
2. Declarar o `file` do observável como um pathspec mágico aceito pelo parser de
   contrato, por exemplo `:(glob)**/*.rs`.
3. Comparar commits nos quais somente uma das correspondências muda.
4. Repetir com o literal `:(glob)**/*.rs` realmente versionado, para distinguir path
   literal de expressão.

No adapter, as respostas de `ls-tree` são inseridas pelos paths expandidos, enquanto a
consulta posterior procura a chave literal do contrato. O path solicitado pode assim
permanecer marcado como `Missing`, embora Git tenha retornado outros arquivos.

**Falso sucesso a procurar:** `PRESERVED` quando `on_missing = "absent"`, ou
`UNKNOWN(MissingObservable)` em vez de erro de entrada, apesar de a expressão ter lido
paths não declarados. A defesa mecânica é rejeitar qualquer pathspec magic ou enviar
um pathspec explicitamente literal, além de rejeitar toda resposta cujo path não seja
um dos paths exatos solicitados.

### P0 — timeout mata somente o filho direto

No timeout, o código chama `child.kill()` e `child.wait()`. Não cria nem encerra grupo
de processos. Um Git ou helper hostil pode criar um descendente que herda stdout ou
stderr e continua vivo após o erro. As threads leitoras ficam bloqueadas nesses FDs e
seus `JoinHandle`s são abandonados no ramo de timeout.

Ataque reproduzível:

1. Colocar primeiro no `PATH` um shim chamado `git` que inicia um filho de longa
   duração, faz o filho herdar stdout/stderr e mantém o pai vivo além de 10 s.
2. O descendente grava seu PID e um heartbeat em diretório temporário.
3. Invocar `refine-revisions` e aguardar o erro de timeout.
4. Verificar por alguns segundos se PID, heartbeat e FDs continuam ativos.

**Efeito colateral a procurar:** CLI retorna exit `2`, mas o heartbeat continua, o
descendente permanece vivo ou há threads/leitores bloqueados. A defesa precisa matar
a árvore/grupo de processos, fechar pipes e aguardar leitores com limite verificável.

### P0 — orçamento é aplicado depois da materialização integral de stdout

`cat-file --batch-command` é drenado integralmente para `Vec<u8>` antes de interpretar
headers e comparar 4 MiB/32 MiB. Logo os limites são semânticos, não limites de leitura
ou memória: um blob enorme já foi lido e alocado quando vira `BudgetExhausted`.

Ataque reproduzível:

1. Criar localmente blobs de 5 MiB, 40 MiB e um blob esparso/compactável muito maior;
   commitá-los nos paths observados.
2. Executar com limite externo de memória e medir RSS, bytes lidos e duração.
3. Repetir com vários OIDs únicos cuja soma exceda 32 MiB.

**Falso sucesso/efeito a procurar:** saída final corretamente `UNKNOWN`, porém RSS ou
I/O excede substancialmente o orçamento declarado; em limite de memória, abort/pânico
ou morte pelo sistema em vez de `UNKNOWN(BudgetExhausted)`. A defesa requer parsing
streaming do header, recusa antes de alocar o corpo e leitura/descarta limitada.

### P0 — objetos externos via alternates e promisor

Desabilitar configuração global/sistema e lazy fetch não elimina configuração local,
`objects/info/alternates`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, repositórios parciais ou
helpers de promisor. Alternates permitem que commit/tree/blob sejam lidos fora do
objeto database do repositório passado. `GIT_NO_LAZY_FETCH=1` deve bloquear fetch
automático, mas isso precisa ser provado para a versão de Git sob teste e para todas as
invocações.

Ataque reproduzível:

1. Criar repositório A cuja `objects/info/alternates` aponta para os objetos de B;
   deixar A sem o blob requerido e confirmar que `git cat-file` o encontra em B.
2. Executar refinamento em A e depois tornar B inacessível; comparar os resultados.
3. Criar repositório partial-clone local com objeto promisor ausente e um upload-pack
   sentinela que somente registra invocações, sem rede real.
4. Repetir definindo `GIT_ALTERNATE_OBJECT_DIRECTORIES` no ambiente do processo.

**Falso sucesso/efeito a procurar:** `PRESERVED` depende de bytes externos a A, ou o
sentinela promisor é executado. Se alternates forem permitidos, a alegação deve dizer
isso explicitamente e congelar/proteger sua proveniência; se não forem, limpar ambiente
e rejeitar alternates/promisor antes da leitura.

## Outros ataques reproduzíveis

### P1 — configuração local e ambiente Git não estão herméticos

O comando fixa algumas opções, mas Git ainda lê `.git/config` e variáveis como
`GIT_DIR`, `GIT_WORK_TREE`, `GIT_OBJECT_DIRECTORY`,
`GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_CONFIG_COUNT` e seus pares key/value.

- Definir `GIT_DIR` para outro repositório e passar uma raiz inocente a `-C`.
- Definir object directory externo que contém OIDs compatíveis.
- Adicionar includes locais/condicionais com arquivo-sentinela e tornar o include
  ilegível para observar se afeta a execução.
- Usar `GIT_CONFIG_COUNT` para tentar reabilitar protocolo, promisor ou outra opção e
  registrar a precedência efetiva frente aos `-c` do adapter.

**Oráculo:** o repositório nominal não pode ser silenciosamente substituído. Ou o
ambiente relevante é limpo e a configuração permitida é enumerada, ou o recibo não
pode alegar hermeticidade. Falha de config deve ser erro/`UNKNOWN`, nunca ausência
conhecida.

### P1 — refs hostis e confusão de revisão

`rev-parse --verify --end-of-options` é boa proteção contra opções, mas a expressão
acrescenta `^{commit}` ao texto recebido. Exercitar refs com espaços, newline, `@{...}`,
`:/regex`, `^`, `~`, `^{tree}`, `--help`, Unicode e nomes próximos de SHA.

Ataques:

- Criar refs legais com nomes limítrofes e confirmar que o OID retornado é commit
  completo e usado nos passos seguintes.
- Mover a ref entre resolução e leitura, em loop. O snapshot e os witnesses devem
  continuar usando somente o OID já resolvido.
- Injetar um shim de Git que retorna OID hex truncado ou excessivamente longo no
  `rev-parse`. A validação atual aceita qualquer sequência hexadecimal não vazia; o
  próximo comando pode resolver abreviação ou produzir erro tardio.

**Oráculo:** nenhum ref vira opção; mudança posterior da ref não muda o artifact ID;
OID malformado nunca chega a `ls-tree`. Exigir comprimento/formato compatível com o
object format efetivo, não apenas “algum hex”.

### P1 — framing `ls-tree` permissivo e resposta não solicitada

O parser aceita qualquer registro com três campos e insere o path retornado, inclusive
um path não solicitado. Também não valida o formato do OID nessa fase. Com Git real,
pathspec magic fornece a via mais natural; com shim controlado, testar:

- registro sem NUL final, tab ausente, metadata com campo extra, path não UTF-8;
- mode/type incoerente, OID não hexadecimal, path duplicado e path não solicitado;
- `100644 blob` para path pai/filho conflitante.

**Oráculo:** todos produzem erro de entrada ou inconclusivo, nunca `Missing`/`Absent`
que possa contribuir para `PRESERVED`. Respostas fora do conjunto literal solicitado
devem ser rejeitadas.

### P1 — framing batch aceita sufixo não consumido

Para cada OID, o parser valida header, tamanho e newline. Porém, após consumir os OIDs
esperados, não verifica `cursor == bytes.len()`. Uma resposta extra ou lixo final pode
ser ignorado. Além disso, todo framing é retido antes da validação.

Use shim de Git que reproduza uma resposta válida e acrescente: header extra, byte
isolado, segundo blob, NUL ou megabytes de lixo.

**Falso sucesso a procurar:** snapshot/`PRESERVED` apesar de stdout conter framing
adicional inválido. Defesa: consumo exato do stream e erro em qualquer byte residual.

### P1 — objeto ausente não pode virar ausência conhecida

Construir commit/tree válido e remover somente o blob alcançável no objeto database de
uma cópia descartável. Repetir com alternates presente/ausente e promisor marcado.

**Oráculo:** `cat-file` ausente ou incompleto bloqueia/gera insuficiência tipada; nunca
alimenta `on_missing = "absent"`. Nenhum fetch/helper pode ser chamado para reparar o
objeto.

### P1 — path count não limita custo total do contrato

O limite conta paths únicos, não specs/queries. Muitas specs podem apontar para o mesmo
blob e causar custo elevado no extrator após uma única leitura. Criar milhares de
observáveis sobre um path dentro do orçamento de paths e medir CPU/tempo total.

**Oráculo:** a operação inteira respeita orçamento/timeout documentado e termina como
insuficiente, não fica ilimitada depois que os processos Git terminam. O timeout atual
cobre cada subprocesso Git, não a extração nem a operação completa.

### P2 — deduplicação por OID altera contabilidade de orçamento

Vários paths com o mesmo blob são lidos uma vez e contam uma vez no total. Isso é
eficiente, mas o significado de “32 MiB por revisão” precisa ser explícito: bytes de
objetos únicos ou soma lógica por path. Crie centenas de paths para o mesmo blob de
4 MiB e confirme a política escolhida.

### P2 — symlink, gitlink e modes inesperados

Criar tree por plumbing contendo `120000 blob`, `160000 commit`, modes regulares
aceitos e modes inesperados. Consultar cada path com `on_missing = "absent"`.

**Oráculo:** symlink/gitlink sempre resulta em `Unknown(UnsupportedParser)`; mode/type
inesperado é erro. Nenhum conteúdo apontado por symlink e nenhum submódulo é aberto.

### P2 — preservação sob erros e concorrência

Antes/depois de cada ataque, capturar sem executar lógica do alvo:

- bytes de `.git/HEAD`, `.git/index`, `packed-refs`, refs soltas, logs e stash;
- árvore de nomes, sizes e hashes sob `.git` e no working tree;
- `git status --porcelain=v2 --untracked-files=all`;
- PIDs/processos sentinela e arquivos externos usados por helpers.

Rodar com worktree limpo e sujo, índice com alterações, stash, refs movidas
concorrentemente, permissões somente leitura e objeto corrompido.

**Oráculo:** nenhum byte preexistente muda, nenhum lock/temporário fica, HEAD/index/
stash não mudam e nenhum processo sobrevive. Separar mudanças feitas pelo próprio
harness (por exemplo, mover ref concorrente) das atribuíveis ao comando.

## Quatro ataques recomendados para teste black-box

| Ordem | Fixture/ação exata | Falso sucesso ou efeito colateral procurado |
|---|---|---|
| 1 | Contrato com `file = ":(glob)**/*.rs"`, dois `.rs` divergentes e `on_missing = "absent"` | `PRESERVED` ou ausência conhecida após Git expandir paths não literais |
| 2 | Shim `git` cria descendente com stdout herdado e heartbeat, depois bloqueia por mais de 10 s | comando retorna timeout, mas descendente/heartbeat/thread leitora permanece |
| 3 | Blob observado de 40+ MiB sob limite externo de memória; medir RSS e I/O | orçamento de 32 MiB só aparece após ler/alocar tudo, ou processo aborta em vez de `UNKNOWN` |
| 4 | Repo A usa alternate de B e, separadamente, promisor local com helper-sentinela | `PRESERVED` depende de objeto fora de A ou helper/fetch é executado |

Os ataques 1 e 2 têm maior chance de revelar defeito observável diretamente. O ataque
3 testa se “orçamento” é proteção de recursos ou apenas classificação posterior. O
ataque 4 decide uma ambiguidade de especificação importante: alternates podem ser uma
fonte local legítima para Git, mas contradizem uma leitura forte de repositório
autocontido e precisam de política explícita.

## Matriz priorizada

| Prioridade | Vetor | Alegação pressionada | Resultado seguro |
|---|---|---|---|
| P0 | pathspec magic | paths hostis não reinterpretados | literal exato ou rejeição antes de Git |
| P0 | descendente no timeout | encerramento sem descendentes/leitores | árvore morta, pipes fechados, retorno limitado |
| P0 | blob enorme | orçamento/timeout | recusa streaming antes de alocação excessiva |
| P0 | alternate/promisor | sem fetch/protocolo externo; proveniência | política explícita, zero helper inesperado |
| P1 | ambiente/config local | isolamento do repositório | ambiente limpo ou influência declarada |
| P1 | OID hex de tamanho inválido | somente OIDs resolvidos | formato completo validado antes de uso |
| P1 | resposta `ls-tree` extra | framing/path exato | erro, nunca ausência conhecida |
| P1 | sufixo batch ignorado | framing inválido | consumo exato ou erro |
| P1 | objeto removido | objeto ausente não preserva | erro/UNKNOWN e zero fetch |
| P1 | milhares de specs em um path | orçamento global | limite de operação/extrator |
| P2 | aliases do mesmo blob | definição dos 32 MiB | contabilidade documentada e determinística |
| P2 | symlink/gitlink/mode | somente blob regular | UNKNOWN/erro sem seguir entrada |
| P2 | worktree/índice/refs/stash | preservação byte-idêntica | snapshot antes/depois idêntico |
