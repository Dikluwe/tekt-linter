# Relatório P0076 — saneamento da ordem pública de V11

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência

- contrato do assessment 0008: `86a7a15`;
- RED e evidência adversarial: `04d3017`;
- contrato P0076: `a8da100`.

A implementou sem ler gate/assessment/lab; B endureceu o gate sem ler a produção; C
revisou a produção final sem ler testes de B.

## Materialização

V11 continua calculando exatamente
`declared - (implemented ∪ blanket)`. A diferença é materializada em vetor, ordenada por
`sort_unstable()` sobre `&str` e só então convertida em violações. Não houve alteração
dos conjuntos globais, pertinência, cardinalidade, nível, mensagem ou localização.

O prompt causal absorveu a ordem pública canônica. Os hashes foram gerados pelo fluxo
oficial `--fix-hashes`: prompt no header `0dd35453` e código no prompt `b535b5dc`.

## Gates finais

- assessment 0008 endurecido: 6/6, zero ignorados;
- adversário final: 6/6, **NÃO REABRIR**;
- 512 reconstruções equivalentes: vetor completo idêntico;
- testes unitários: 624/624;
- fixtures: 83/83;
- todos os assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- dry-run de hashes: nada a corrigir;
- Rust tocado passa `rustfmt --check`;
- `git diff --check`: limpo.

Permanece o warning preexistente `print_tree` em `ts_parser.rs`.

## Parada

Nenhum merge, instalação ou release foi realizado. V2, V8 e V10 permaneceram intactas.
