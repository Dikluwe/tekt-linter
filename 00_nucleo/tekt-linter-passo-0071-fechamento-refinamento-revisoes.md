# Passo operacional 0071 — fechamento do refinamento entre revisões

> **Natureza:** comando operacional temporário para o LLM; não é regra arquitetural
> **Estado:** gate local concluído; matriz externa e integração aguardam decisão humana
> **Identidade:** P0071, numeração operacional solicitada pelo humano; não é regra arquitetural
> **Continua:** Etapa B2 materializada no commit `58ce52b`
> **Autoridade:** testes e correções no branch dedicado; merge e instalação exigem gates próprios

## Objetivo

Fechar a Etapa B2 de refinamento entre revisões com evidência negativa suficiente para
considerar seguro o comando:

```bash
crystalline-lint refine-revisions <repository-root> \
  --before-ref <sha-ou-ref> \
  --after-ref <sha-ou-ref> \
  --contract refinement.toml
```

O fechamento deve provar que sucesso, violação, inconclusão e erro preservam o estado
do repositório; que os limites nunca viram aprovação silenciosa; e que B2 produz a
mesma semântica de B1 + Etapa A.

## Não objetivos

- comparar revisão com working tree;
- executar build, testes ou comandos do repositório analisado;
- buscar objetos ou refs pela rede;
- inicializar Git LFS ou submódulos;
- adicionar `gix`, `git2`, solver SMT ou wrapper genérico;
- implementar extração interprocedural;
- criar regra `V*`;
- renumerar este ou outros passos;
- fazer merge no `master` antes do gate final;
- substituir o binário instalado antes de todos os testes locais passarem.

## Estado inicial obrigatório

Antes de alterar arquivos:

1. confirmar branch `codex/refinement-validator`;
2. registrar HEAD e `git status --short`;
3. confirmar que `master` permanece em `4c9583d` ou registrar divergência legítima;
4. registrar hashes de HEAD, índice, diff rastreado, status NUL e stash;
5. executar o caso positivo já conhecido:

```bash
target/debug/crystalline-lint refine-revisions . \
  --before-ref 18a9b6e \
  --after-ref 0f4e5df \
  --contract refinement-self.toml
```

Resultado esperado: `PRESERVED`, exit `0`, OIDs completos e aviso de working tree
ignorado.

## Bloco A — fixtures de entrada e imutabilidade

Criar repositórios Git temporários e autocontidos. Cada fixture registra estado antes
e depois e compara, no mínimo:

- HEAD e branch;
- diff do índice;
- diff do working tree;
- status NUL, incluindo não rastreados;
- lista de stash;
- bytes de um arquivo local deliberadamente sujo.

Cobrir:

1. `PRESERVED` mantém todo o estado;
2. `VIOLATED` mantém todo o estado;
3. `UNKNOWN` mantém todo o estado;
4. ref inexistente termina com exit `2`;
5. ref semelhante a opção, como `--help`, não é interpretada como flag;
6. diretório que não é repositório Git termina com exit `2`;
7. erro em qualquer lado não publica snapshot parcial nem cria temporário no projeto;
8. duas refs simbólicas para o mesmo OID produzem resultado idêntico.

Não usar checkout, switch, restore, reset, stash, clean ou worktree nos comandos do
produto. Fixtures podem criar commits em seus próprios diretórios temporários para
preparar dados; essa escrita pertence ao harness, não ao linter analisado.

## Bloco B — semântica e equivalência B1/B2

Para o mesmo par de commits e contrato:

1. executar `refine-revisions`;
2. exportar externamente os trees para diretórios temporários;
3. executar `snapshot` em cada exportação;
4. executar `refine` sobre os snapshots;
5. afirmar mesmo exit code, mesmo veredito e mesmas testemunhas semânticas;
6. permitir apenas diferença de apresentação/proveniência previamente declarada.

Cobrir explicitamente:

- correção que retorna `PRESERVED`;
- regressão reversa que retorna `VIOLATED` com valores fonte/alvo;
- baseline pré-domínio que retorna `UNKNOWN(MissingObservable)`;
- arquivo ausente com `on_missing = unknown`;
- arquivo ausente com `on_missing = absent`;
- erro sintático que retorna `Unknown(OpaqueConstruction)`;
- ordem diferente de observáveis e paths sem alterar o resultado.

O teste histórico mínimo continua sendo:

| Antes | Depois | Resultado |
|---|---|---|
| `18a9b6e` | `0f4e5df` | `PRESERVED` |
| `f8a0dae` | `0f4e5df` | `UNKNOWN(MissingObservable)` |

Os oráculos de contexto, campo e autoridade devem continuar aceitando a correção e
rejeitando a regressão no fluxo manual existente. Se forem transportados para commits
temporários, B2 deve reproduzir os mesmos vereditos.

## Bloco C — árvores Git hostis e efeitos proibidos

Criar fixtures atomizadas para:

1. symlink no path observável: não seguir e retornar `Unknown(UnsupportedParser)`;
2. submódulo/gitlink no path observável: não inicializar e retornar inconclusivo;
3. ponteiro Git LFS: tratar como blob cru, sem executar `git-lfs`;
4. configuração de clean/smudge filter com comando sentinela: sentinela não executa;
5. hook sentinela configurado no repositório: sentinela não executa;
6. replace object configurado: `GIT_NO_REPLACE_OBJECTS=1` mantém o OID original;
7. objeto ausente/prometido: nenhuma tentativa de rede e exit `2` ou razão tipada;
8. path com espaços, hífen inicial e caracteres Unicode: framing permanece correto;
9. path não UTF-8, quando a plataforma permitir: erro fechado, nunca leitura errada;
10. blob cujo conteúdo imita o framing batch: bytes são lidos pelo tamanho declarado.

Para provar ausência de execução externa, usar marcadores locais dentro do diretório
temporário e afirmar que não foram criados. Não depender apenas de inspeção do código.

## Bloco D — budgets e terminação

Fixar testes de fronteira para os valores vigentes:

- 512 paths aceitos; 513 rejeitados;
- blob de exatamente 4 MiB aceito;
- blob acima de 4 MiB nunca truncado e nunca `PRESERVED`;
- soma de exatamente 32 MiB aceita quando os limites individuais permitirem;
- soma acima de 32 MiB nunca `PRESERVED`;
- processo Git bloqueado por mais de 10 segundos é encerrado;
- erro de framing, tamanho inválido e resposta truncada são fechados.

O teste de timeout não deve substituir o Git real do sistema globalmente. Injetar o
executável/process runner por uma seam de L3 ou executar um helper controlado pela
fixture. Se a seam exigir abstração nova, mantê-la em L3/L4; L1 continua sem Git,
processo, relógio ou I/O.

Budget esgotado deveria chegar como `Unknown(BudgetExhausted)` quando já existe um
contrato válido e a insuficiência pertence à evidência. Se a implementação atual
retornar apenas erro de entrada, registrar a divergência e corrigir antes de fechar.

## Bloco E — portabilidade e formatos de repositório

### Obrigatório antes do merge

- Linux no ambiente local/CI;
- Windows e macOS em CI;
- Git mínimo declarado ou matriz que demonstre a versão suportada;
- paths e argumentos sem shell nas três plataformas;
- ausência de dependência nova e `cargo test --workspace` verde.

### Pode ser segunda rodada, mas deve ficar rastreado

- repositório SHA-256;
- alternates;
- shallow clone com objetos locais suficientes;
- partial clone com objeto ausente e fetch proibido.

Se a infraestrutura disponível não permitir algum caso, não declarar suporte provado.
Abrir item explícito no relatório com plataforma, risco e condição de fechamento.

## Bloco F — documentação e interface

Atualizar, conforme os resultados:

1. ADR-0019, removendo linguagem proposta que já foi aprovada;
2. `refinement-validator.md`, somente se a semântica mudar;
3. relatório de investigação, distinguindo casos testados e adiados;
4. ajuda CLI e documentação pública com Git mínimo, budgets e working tree ignorado;
5. mensagem de erro para Git ausente, ref inválida, objeto ausente e timeout;
6. política de exit code de budgets: nunca exit `0`;
7. exemplo reproduzível de `refine-revisions`.

Resselar hashes causais apenas depois da última mudança no prompt. O auto-fix de hashes
não pode esconder drift semântico: revisar o diff antes de aceitar o resselo.

## Gate de fechamento local

Executar:

```bash
cargo test --workspace
target/debug/crystalline-lint .
git diff --check
```

Além disso:

- executar os três casos reais `PRESERVED`, `VIOLATED` e `UNKNOWN`;
- confirmar working tree do repositório analisado inalterado antes/depois;
- confirmar zero warnings/errors novos no auto-lint;
- confirmar zero drift de hash;
- confirmar que `Cargo.toml` e `Cargo.lock` não ganharam biblioteca Git;
- registrar contagens finais de testes e fixtures;
- manter a implementação em commit separado no branch dedicado.

Se qualquer fixture de segurança falhar, corrigir no branch e repetir o bloco completo.
Não fazer merge nem instalar o binário enquanto o gate local estiver vermelho.

## Gate de integração

Depois do gate local verde, apresentar ao humano:

1. commits que serão integrados (`f8a0dae` até o commit final de fechamento);
2. diff resumido contra `master`;
3. resultados Linux/macOS/Windows;
4. casos ainda não provados;
5. comportamento e autoridade do novo subprocesso Git;
6. caminho e versão do binário que será substituído.

O merge no `master` exige autorização humana explícita. Preferir merge que preserve os
commits do branch ou estratégia indicada pelo humano; não reescrever histórico sem
pedido. Após o merge, repetir suíte e auto-lint no `master`.

## Gate de atualização do sistema

Somente após integração aprovada:

1. construir release a partir do commit integrado;
2. executar smoke tests no binário release;
3. localizar exatamente o binário instalado atual;
4. registrar versão/hash antigo e novo;
5. pedir autorização antes de substituir arquivo fora do workspace;
6. manter meio de recuperação do binário anterior;
7. executar `--help`, lint local e `refine-revisions` pelo binário instalado;
8. informar claramente qual arquivo foi substituído e como recuperar.

Não confundir `target/debug/crystalline-lint` ou `target/release/crystalline-lint` com o
binário efetivamente instalado no sistema.

## Critérios de aceitação

1. Todos os casos obrigatórios A–D possuem fixture automática e passam.
2. `PRESERVED`, `VIOLATED`, `UNKNOWN` e erros mantêm o repositório intacto.
3. Symlink, submódulo, hook, filtro e LFS não executam conteúdo externo.
4. Objeto ausente não acessa rede.
5. B1 + Etapa A e B2 produzem o mesmo veredito para a mesma transformação.
6. Ausência real, entrada não suportada e objeto ilegível permanecem distintos.
7. Timeout e budgets nunca produzem `PRESERVED`.
8. OIDs completos aparecem na proveniência e testemunhas.
9. Linux passa localmente; Windows/macOS passam em CI ou ficam explicitamente bloqueados.
10. Suíte, auto-lint, hashes e `git diff --check` passam.
11. Nenhuma biblioteca Git é adicionada.
12. Merge e instalação ocorrem somente depois dos respectivos consentimentos.

## Relatório final exigido

Separar:

- commit e branch testados;
- matriz de fixtures e resultados;
- estado Git antes/depois;
- vereditos e exit codes;
- budgets exercitados;
- efeitos externos cuja ausência foi provada;
- plataformas e formatos de repositório testados;
- limitações ainda abertas;
- resultado da integração no `master`;
- binário instalado, hash anterior/novo e recuperação;
- itens ainda adiados: revisão versus working tree, SMT e análise interprocedural.

## Parada obrigatória

Executar primeiro A–F e o gate local no branch. **PARAR** e apresentar o relatório
antes de fazer merge no `master` ou substituir o binário instalado no sistema.

## Resultado da execução local

Executado em 2026-08-24 no branch `codex/refinement-validator`. Os blocos locais
produziram fixtures para vereditos, entradas inválidas, imutabilidade, equivalência
B1/B2, symlink, gitlink, hooks/filtros sentinela, timeout e budgets. O estouro de blob
foi corrigido para resultar em `Unknown(BudgetExhausted)`.

Suíte final: 581 testes unitários, 83 fixtures gerais e 10 testes CLI, todos verdes.
Auto-lint terminou com exit 0 e sem drift causal. O relatório detalhado está em
`relatorio-fechamento-refinamento-revisoes.md`.

A parada obrigatória foi alcançada. Windows, macOS, SHA-256, alternates, shallow e
partial clone continuam sem evidência neste host; merge e atualização do binário não
foram executados.

Após autorização humana para continuar, foi preparada uma matriz CI específica em
Ubuntu, macOS e Windows. Seu resultado remoto permanece pendente; isso não equivale a
evidência local inventada.
