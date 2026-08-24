# Relatório P0079 — saneamento da evidência de V9

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência

- contratos P0078/assessment 0010: `c02110a`;
- RED congelado: `efffcfb`;
- contrato P0079: `b19046f`.

A implementou sem ler assessment/gate/lab; B endureceu o gate sem ler produção; C
revisou a produção final sem ler testes de B. O gate de B reproduziu o RED antes de
passar sobre o worktree corrigido.

## Materialização

O filtro V9 conserva o `target_subdir` autorizado a alcançar a construção da violação.
A mensagem inclui literalmente import path e subdir rejeitado. `None` continua isento e
não recebe placeholder. Matriz, portas, guard, ImportKind, cardinalidade, ordem, nível e
localização não mudaram. V3 e todas as fronteiras L3 permaneceram intactas.

O prompt causal absorveu a evidência final. O fluxo oficial registrou `4253c633` no
header da produção e `eff86053` como Hash do Código no prompt.

## Gates finais

- assessment 0010 endurecido: 6/6, zero ignorados;
- adversário final: 6/6, **NÃO REABRIR**;
- testes unitários: 628/628;
- fixtures: 83/83;
- todos os assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- dry-run de hashes: nada a corrigir;
- Rust tocado passa `rustfmt --check`;
- `git diff --check`: limpo.

Permanece o warning preexistente `print_tree` em `ts_parser.rs`.

## Parada

Nenhum merge, instalação ou release foi realizado.
