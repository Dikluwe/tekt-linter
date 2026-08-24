# Passo operacional 0079 — saneamento da evidência de V9

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0010 e P0078, commits `c02110a` e `efffcfb`

## Objetivo

Fechar o RED de V9 preservando na violação o `target_subdir` que participou da decisão.
Imports distintos na fronteira arquitetural não podem colapsar na mesma evidência.

## Decisões congeladas

1. Pertinência permanece exatamente: origem L2/L3, destino L1 e `target_subdir=Some`
   ausente de `L1Ports`, após aplicação do guard `check_test_imports`.
2. A mensagem V9 passa a incluir literalmente o `target_subdir` rejeitado e o import
   path já existente. Não inferir subdir do path nem normalizar nenhum dos dois.
3. `None` continua isento e, por construção, nunca alcança a criação da violação. Não
   criar placeholder ou novo diagnóstico para esse estado.
4. Rule id, nível, source path, linha, coluna, cardinalidade e ordem permanecem iguais.
5. Caixa, Unicode, NFC/NFD, prefixos e string vazia continuam identidades textuais
   distintas conforme o conjunto de portas recebido.
6. V3, `L1Ports`, traits, parsers, configuração e wiring não serão modificados.
7. O prompt causal `prompts/rules/pub-leak.md` absorve a evidência final e os hashes são
   atualizados pelo fluxo oficial `--fix-hashes`.

## Segregação e execução

- A implementa passo e prompt causal sem ler assessment/gate/lab.
- B endurece e ativa o gate sem ler produção modificada.
- C revisa após o primeiro gate verde sem ler testes de B.
- O orquestrador executa suíte completa, assessments, auto-lint e `git diff --check`.

## Critérios de fechamento

- assessment 0010: 6/6, zero ignorados;
- dois subdirs diferentes com mesmo import path/linha geram violações distintas;
- subdir hostil é preservado literalmente;
- V3 e toda a matriz V9 permanecem verdes;
- adversário declara **NÃO REABRIR** ou apresenta novo RED reproduzível.

## Parada

Registrar relatório final. Não fazer merge, instalação ou release.
