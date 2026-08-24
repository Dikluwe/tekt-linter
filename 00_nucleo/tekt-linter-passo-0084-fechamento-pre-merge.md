# Passo operacional 0084 — fechamento pré-merge da materialização segregada

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; READY WITH RESIDUAL AUDIT
> **Branch:** `codex/segregated-materialization`
> **Base de comparação:** merge-base com `Tekt`
> **Predecessor:** P0083

## Objetivo

Fechar o ciclo acumulado neste branch com evidência suficiente para uma decisão de merge,
sem iniciar novas auditorias do linter. O passo verifica somente o delta já produzido,
reconcilia seu estado documental e confirma que os consumidores relevantes não regrediram.

## Limite de escopo

Este passo pode:

- ler todo o delta do branch contra a base;
- executar gates, suíte, auto-lint e smoke tests;
- corrigir inconsistências documentais de estado;
- corrigir exclusivamente defeitos introduzidos pelo próprio delta, se um RED os provar;
- produzir matriz e relatório final.

Este passo não pode:

- abrir auditoria de uma regra ou componente ainda não coberto;
- sanear avisos históricos do Typst Cristalino;
- atualizar em massa os hashes do Typst Cristalino;
- instalar, publicar, fazer release ou executar o merge;
- misturar mudanças preexistentes do worktree do Typst com este branch.

## Execução

### 1. Congelar a superfície do branch

Registrar:

- merge-base, HEAD e lista de commits;
- número de arquivos e volume do delta;
- arquivos de produção, contratos, prompts, testes e relatórios alterados;
- regras e componentes efetivamente tocados.

Qualquer mudança posterior deve ser classificada como correção do fechamento ou excluída.

### 2. Reconciliar assessments e passos

Construir uma tabela com todos os assessments 0001–0012 contendo:

- alvo;
- RED/SPEC-GAP original;
- passo de saneamento correspondente;
- gate final;
- estado atual verificável;
- commit de fechamento.

Estados antigos como `CONGELADO PARA TRIAGEM/INVESTIGAÇÃO` devem ser atualizados somente
quando um relatório e um gate posterior provarem o fechamento. Divergência sem prova é
`SPEC-GAP`, não deve ser reclassificada por inferência.

### 3. Auditoria adversarial do delta

Um agente independente recebe a lista congelada de alegações e commits, mas não os
relatórios conclusivos. Ele procura:

- produção alterada sem gate correspondente;
- gate que importa expectativas da produção;
- L0 atualizado depois do gate sem novo vínculo por hash;
- RED mencionado historicamente e não encerrado;
- mudança funcional escondida como reparo de hash/formatação;
- arquivo novo não alcançado pela suíte ou pelo wiring;
- contradição entre assessment, relatório e estado do código.

Os achados são congelados antes de qualquer correção.

### 4. Validação integral do linter

Executar, no mínimo:

1. `cargo test --workspace`;
2. `cargo run --quiet -- . --fix-hashes --dry-run`;
3. auto-lint V1/V5/V7 no próprio repositório;
4. `rustfmt --check` apenas nos arquivos Rust alterados pelo branch;
5. `git diff --check`;
6. confirmação de worktree limpo após o commit de fechamento.

O aviso preexistente de `print_tree` não utilizada pode ser registrado, mas não saneado
neste passo, salvo se o delta provar que foi introduzido pelo branch.

### 5. Smoke test no Typst Cristalino

Usar diretamente o binário deste branch, sem instalação e sem escrita no projeto Typst:

- executar as regras funcionalmente alteradas pelo branch;
- executar passagem arquitetural sem V5/V6 para separar violações de drift;
- executar reparo de hashes somente em modo seco;
- confirmar antes e depois que o worktree Typst não foi modificado pelo teste.

Os 415 candidatos de hash já observados são dívida do consumidor e não autorizam reparo
automático dentro deste passo.

### 6. Matriz final de rastreabilidade

Emitir uma linha por unidade fechada:

| Unidade | Produção/L0 tocado | Gate independente | Resultado | Commit |
|---|---|---|---|---|

A matriz deve cobrir materialização/refinement, infraestruturas saneadas e regras
auditadas. Arquivos alterados apenas por hash devem ser agrupados e identificados como
mudança não funcional.

### 7. Veredito

O relatório final usa exatamente um estado:

- `READY TO MERGE`: todos os critérios passaram e não resta RED/SPEC-GAP do delta;
- `BLOCKED`: existe RED funcional, evidência ausente ou inconsistência documental não
  resolvida;
- `READY WITH RESIDUAL AUDIT`: o delta está fechado, mas componentes fora dele continuam
  pendentes para outro branch.

O estado esperado, se todas as verificações passarem, é `READY WITH RESIDUAL AUDIT`.

## Critérios de fechamento

- superfície do branch congelada e explicada;
- assessments 0001–0012 reconciliados por evidência;
- toda mudança funcional coberta por gate;
- nenhum RED/SPEC-GAP residual pertencente ao delta;
- suíte global, hashes, auto-lint, formatação e diff verdes;
- smoke test Typst sem V4/V14 e demais regressões introduzidas pelo branch;
- worktrees do linter e do Typst sem mudanças causadas pelo passo;
- matriz e relatório final emitidos;
- nenhum merge, instalação ou release executado.

## Saída esperada

- `00_nucleo/assessments/0013-fechamento-pre-merge.md`;
- `00_nucleo/relatorio-p0084-fechamento-pre-merge.md`;
- atualização comprovada dos estados documentais necessários;
- recomendação explícita de merge ou bloqueio.

Após `READY WITH RESIDUAL AUDIT`, o merge pode ser realizado em uma ação separada. A
auditoria restante deve continuar em um novo branch e em lotes menores.

## Resultado

O delta congelado em `1b7e18f` contém 207 arquivos, 14.771 inserções e 568 remoções
contra o merge-base `75a5665`. Os bloqueios documentais encontrados pelo adversário foram
reconciliados e o whitespace histórico foi removido. Suíte, hashes, auto-lint,
`git diff --check` e smoke test no Typst passaram.

Os arquivos Rust novos passam `rustfmt --check`. Arquivos legados tocados apenas por hash
ou por saneamentos acumulados ainda exibem drift contra o rustfmt atual; formatá-los
criaria um refactor transversal fora do escopo. Isso fica como auditoria residual, não
como RED funcional do delta.

O veredito e a matriz completa estão no assessment 0013 e no relatório P0084. Nenhum
merge, instalação ou release foi executado.
