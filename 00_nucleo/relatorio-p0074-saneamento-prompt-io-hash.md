# Relatório P0074 — saneamento segregado de prompt I/O e hashes

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência

- assessment/RED: `525e42c`;
- L0 P0074: `1915216`;
- implementação, migração e gates: `69e923b`.

A implementou sem ler gate/adversário; B escreveu e executou o gate sem ler produção;
C revisou a produção após o primeiro gate sem ler testes de B.

## Fechamento

- paths de prompt confinados, incluindo symlink final, intermediário, root e ancestral;
- hash byte-exato, removendo somente uma meta canônica no escopo autorizado;
- CRLF, BOM, newline final, espaços e decoys permanecem observáveis;
- snapshot exige seção, marcador único fora de fences e schema fechado;
- walker exige diretório local, não segue links, ordena e falha fechado;
- digest possui oito hex minúsculos;
- writers preservam bytes, CRLF e permissões, usam temporário exclusivo e não deixam
  resíduos sob corrida concorrente;
- cache fixa Some/None por instância.

## Divergências classificadas

O adversário reabriu o gate para roots symlink, metas fora do header, snapshots fora de
seção/fences e writer com meta ausente. Roots e escopos foram corrigidos. Meta ausente
no writer foi classificada conforme D3: escrita é estritamente substitutiva e retorna
erro sem alterar bytes.

Um reteste adversarial inicialmente manteve RED de root ancestral por executar binário
temporário ligado a artefato concorrente obsoleto. Rebuild completo + recompilação do
probe confirmou `reader=false snapshot=false walker=false`; o veredito final passou a
**NÃO REABRIR**.

## Migração de hash

O algoritmo anterior reconstruía linhas e normalizava line endings. O fluxo
`--fix-hashes` migrou 73 headers e atualizou as metas dos prompts consumidores. O
fixture positivo V5 com digest hardcoded também foi recalculado. Após a migração, o
linter reportou zero drift.

## Gates finais

- assessment P0074: 6/6, zero ignorados;
- testes unitários: 618/618;
- fixtures: 83/83;
- demais assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- `git diff --check`: limpo;
- adversário: **NÃO REABRIR**.

Permanece o warning Rust preexistente `print_tree` em `ts_parser.rs`. TOCTOU por troca
externa do destino continua sem seam público determinístico e não foi elevado a prova.

## Parada

Nenhum merge, instalação ou release foi realizado. A análise integral do linter pode
continuar com V5/V6/V7 sobre a nova identidade byte-exata.
