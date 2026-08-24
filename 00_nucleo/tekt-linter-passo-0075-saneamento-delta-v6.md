# Passo operacional 0075 — saneamento determinístico do delta V6

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0007, commits `9dcad86` e `7301a71`

## Objetivo

Fechar os dois REDs de V6 sem ampliar sua responsabilidade: `compute_delta` deve calcular
diferença de multiconjuntos e devolver uma representação canônica, para que interfaces
equivalentes e mensagens equivalentes independam da ordem do parser.

## Decisões congeladas

1. `PublicInterface` continua armazenando vetores, mas V6 interpreta cada família
   (`functions`, `types`, `reexports`) como multiconjunto. Cada ocorrência de um lado
   cancela no máximo uma ocorrência estruturalmente igual do outro.
2. Mera permutação, inclusive com duplicatas na mesma multiplicidade, produz delta vazio.
   Acrescentar ou remover uma duplicata produz exatamente uma entrada no delta.
3. Igualdade de função cobre `name`, `params` e `return_type`. Igualdade de tipo cobre
   `name`, `kind` e `members`. Não normalizar strings nem ordenar `params`/`members`, pois
   sua ordem faz parte da assinatura.
4. Cada um dos seis vetores do `InterfaceDelta` sai em ordem total canônica. Funções
   ordenam lexicograficamente por `(name, params, return_type)`; tipos por
   `(name, kind_rank, members)`; reexports por texto. `kind_rank` é explícito e estável,
   não depende de discriminante, locale ou endereço.
5. `InterfaceDelta::describe` conserva sua ordem de grupos pública: funções adicionadas,
   funções removidas, tipos adicionados, tipos removidos, reexports adicionados e
   reexports removidos. Como cada grupo já é canônico, a mensagem também é canônica.
6. A correção permanece pura em L1, sem I/O, cache, filesystem ou alteração da política
   de quando V6 é aplicável. V5 e V7 não serão modificadas.
7. O prompt causal `prompts/rules/prompt-stale.md` deve absorver estas decisões antes do
   fechamento, e seu hash de linhagem deve ser atualizado pelo fluxo oficial do linter.

## Segregação e execução

- A implementa a partir deste contrato e do prompt causal sem ler o gate do assessment
  0007 nem os artefatos adversariais em `lab/`.
- B ativa o gate congelado e pode acrescentar casos simétricos de funções, tipos e
  reexports sem ler a produção modificada.
- C revisa a implementação somente após o primeiro gate verde, sem ler testes de B.
- O orquestrador classifica divergências, executa suíte completa, todos os assessments,
  auto-lint V1/V5/V7 e `git diff --check`.

## Critérios de fechamento

- gate 0007: 6/6, zero ignorados;
- multiplicidade testada nas três famílias e nos dois sentidos;
- invariância por permutação com desempate por todos os campos;
- testes anteriores e fixtures verdes;
- adversário declara **NÃO REABRIR** ou apresenta novo RED reproduzível;
- relatório final registra commits, migração de hash e limites residuais.

## Parada

Não fazer merge, instalação ou release. Se um novo RED exigir decisão além destas sete,
congelar a evidência e interromper antes de ampliar o escopo.
