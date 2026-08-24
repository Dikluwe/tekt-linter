# Passo operacional — refinamento entre revisões sem checkout

> **Natureza:** comando operacional temporário para o LLM; não é regra arquitetural
> **Estado:** investigação e materialização B2 concluídas no branch dedicado
> **Identidade:** descritiva e não numerada
> **Precede:** Etapa A (`refine`) e Etapa B1 (`snapshot`) já materializadas
> **Destino:** decisão absorvida por ADR-0019 e `refinement-validator.md`

## Objetivo

Permitir que o utilizador compare duas revisões de um repositório sem trocar branch,
alterar índice, aplicar stash, criar worktree persistente ou modificar arquivos:

```bash
crystalline-lint refine-revisions \
  --before-ref <sha-ou-ref> \
  --after-ref <sha-ou-ref> \
  --contract refinement.toml \
  <repository-root>
```

O comando deve reutilizar exatamente o extrator e o comparador existentes:

```text
revision A ──> immutable artifact source ──> snapshot A ─┐
                                                        ├──> refine
revision B ──> immutable artifact source ──> snapshot B ─┘
```

Não criar uma segunda semântica de extração para Git.

## Não objetivos

- provar equivalência funcional;
- observar mudanças não commitadas por padrão;
- executar o build, testes, hooks ou comandos do repositório;
- inicializar ou atualizar submódulos;
- baixar objetos, Git LFS ou dependências;
- resolver refs remotas pela rede;
- criar checkout/worktree persistente;
- introduzir wrapper de comandos ou SMT;
- reservar regra `V*`.

## Fase 0 — Medir e escolher o backend

Antes de escrever código de produto, comparar pelo menos três estratégias:

### A. Processo `git` em modo somente leitura

Exemplos a investigar: `git cat-file --batch`, `git ls-tree` e `git show
<tree>:<path>`. Não usar `git checkout`, `git switch`, `git restore`, `git reset`,
`git stash`, `git clean` ou `git worktree add`.

Avaliar:

- disponibilidade e versão mínima do executável;
- custo de um processo por arquivo versus protocolo batch;
- tratamento de paths arbitrários e bytes não UTF-8;
- possibilidade de configuração/alias alterar comportamento;
- como desabilitar prompts, rede, pager, filtros e hooks;
- limites de memória e tamanho por blob;
- mensagens e exit codes estáveis.

### B. Biblioteca Git embutida

Avaliar `gix`, `git2` ou alternativa equivalente sem escolher por conveniência.

Medir:

- nova árvore de dependências e impacto no binário;
- suporte a formatos de repositório usados pelo projeto;
- segurança de parsing de objetos não confiáveis;
- comportamento com alternates, shallow clones e partial clones;
- compatibilidade com a versão mínima de Rust;
- manutenção e superfície de supply chain.

### C. Exportação externa fornecida pelo utilizador

Manter o produto sem autoridade Git: o utilizador produz dois diretórios ou snapshots e
usa os comandos existentes. É o baseline de menor risco e deve permanecer alternativa
documentada mesmo se A ou B for escolhida.

## Gate arquitetural obrigatório

Produzir uma adenda ao ADR-0019 contendo:

1. backend escolhido e por que supera C;
2. autoridade externa concedida ao processo;
3. modelo de ameaça para repositórios e objetos não confiáveis;
4. política de rede, hooks, filtros, LFS e submódulos;
5. limites de bytes, arquivos, tempo e memória;
6. representação de ref inexistente, objeto ausente e clone parcial;
7. comportamento diante de working tree sujo;
8. política de temporários e recuperação;
9. compatibilidade com Windows, Linux e macOS;
10. impacto de dependências e distribuição.

Apresentar a decisão e **PARAR** antes de alterar L1–L4. Este passo não concede por si
só autorização para executar subprocessos Git nem adicionar dependência Git.

**Resultado da Fase 0 (2026-08-24):** a adenda do ADR-0019 recomendou o
backend A, com um processo `git` endurecido e protocolo `cat-file --batch-command`.
As medições e ressalvas estão em
`relatorio-investigacao-refinamento-revisoes.md`. O humano aprovou o gate e a B2 foi
materializada no branch `codex/refinement-validator`.

## Contrato funcional proposto

### Identidade de revisão

Resolver cada ref uma única vez para um object ID imutável antes da extração. O snapshot
registra o OID resolvido como `artifact_id`; o nome simbólico fornecido pode aparecer
apenas como metadado não usado na comparação.

Se a ref mudar durante a execução, o OID inicial continua sendo a autoridade.

### Working tree

O modo entre revisões ignora o working tree, inclusive arquivos modificados e não
rastreados. Deve informar isso claramente. Uma futura opção para comparar revisão com
working tree exige decisão separada porque mistura fontes com modelos de confiança
diferentes.

### Extração

O extrator B1 deve receber uma fonte abstrata de arquivos imutáveis, em vez de exigir
filesystem concreto. A semântica de query, captura, normalização, cardinalidade,
`Absent` e `Unknown` permanece idêntica.

Ausência de arquivo no tree:

- respeita `on_missing` do observável;
- não é erro global por si só;
- não pode ser confundida com falha ao ler um objeto que deveria existir.

### Resultado

Reutilizar `RefinementVerdict` e os exit codes existentes:

- `0`: `PRESERVED`;
- `1`: `VIOLATED`;
- `2`: `UNKNOWN` sem violação ou erro de entrada/proveniência.

Testemunhas devem incluir os dois OIDs e versões do extrator.

## Fixtures RED obrigatórias

### Segurança e imutabilidade

1. comando não altera `HEAD`, branch, índice, working tree ou stash;
2. working tree sujo permanece byte-a-byte igual após sucesso, violação e erro;
3. nomes de ref parecidos com opções não são interpretados como flags;
4. path do contrato não permite fuga do tree lógico;
5. symlink no tree não permite leitura do filesystem hospedeiro;
6. nenhum hook, smudge/clean filter, LFS ou submódulo é executado;
7. nenhuma tentativa de rede ocorre para objeto ausente.

### Semântica

8. duas refs para o mesmo OID produzem snapshots semanticamente idênticos;
9. Etapa A `18a9b6e` → Etapa B1 produz `PRESERVED` no contrato próprio;
10. baseline pré-domínio `f8a0dae` produz `UNKNOWN(MissingObservable)`;
11. os três oráculos locais aceitam correção e rejeitam regressão;
12. arquivo ausente respeita `on_missing = unknown | absent`;
13. erro de parse continua `Unknown(OpaqueConstruction)`;
14. ordem de arquivos e objetos não altera bytes do snapshot;
15. `refine-revisions` e `snapshot + refine` produzem o mesmo veredito.

### Limites

16. blob acima do orçamento produz razão estável, nunca truncamento silencioso;
17. clone parcial/objeto ausente produz erro local sem fetch;
18. ref inválida termina com exit 2 e não cria snapshot parcial publicado;
19. repositório que não é Git termina com exit 2 sem alterar nada;
20. timeout/orçamento esgotado nunca produz `PRESERVED`.

## Materialização proposta após aprovação

1. L1: porta somente leitura para obter conteúdo por path lógico e identidade imutável;
2. L1: nenhuma referência a Git, subprocesso, OID concreto ou biblioteca externa;
3. L3: adapter Git escolhido e adapter filesystem B1 sob a mesma porta;
4. L2: argumentos e mensagens de provenance/working-tree ignorado;
5. L4: resolver OIDs, extrair dois snapshots em memória e chamar o comparador;
6. snapshots temporários somente sob opção explícita de diagnóstico;
7. nenhuma escrita no repositório analisado;
8. cache somente depois de chavear por OID, contrato, extrator e limites;
9. documentação, SARIF e testes de caixa-preta;
10. autoaplicação no branch dedicado antes de considerar merge.

## Guardas contra atalhos

É proibido:

- esconder `git checkout` em script ou diretório temporário;
- confiar em ref sem congelá-la em OID;
- executar comando montado como string de shell;
- aceitar configuração Git do repositório que habilite execução externa;
- seguir symlink de um tree para fora da fonte lógica;
- buscar objetos automaticamente;
- tratar erro de leitura como arquivo ausente conhecido;
- duplicar o comparador ou o normalizador B1;
- serializar timestamp no snapshot;
- usar working tree como substituto silencioso de objeto ausente.

## Critérios de aceitação

1. ADR-0019 atualizado e aprovado antes de código de produto.
2. Backend escolhido com medição e modelo de ameaça documentados.
3. Nenhum estado Git ou arquivo do repositório muda nos testes.
4. Produto não acessa rede, hooks, filtros, LFS ou submódulos.
5. Extração B1 e B2 compartilham uma única semântica.
6. OIDs resolvidos aparecem nas testemunhas e snapshots.
7. `Unknown` cobre objetos, parses e orçamentos inconclusivos.
8. Equivalência de veredito com o fluxo manual é demonstrada.
9. Testes, auto-lint e hashes L0 passam.
10. Implementação fica em commit separado no branch de refinamento.

## Relatório final exigido

Separar:

- backend e autoridade concedida;
- estado Git antes/depois dos testes;
- OIDs e working tree usados na medição;
- vereditos dos baselines e oráculos;
- objetos/casos inconclusivos;
- limites e plataformas testadas;
- dependências adicionadas;
- riscos ainda abertos;
- itens que continuam adiados: revisão versus working tree, wrapper, SMT e análise
  interprocedural.
