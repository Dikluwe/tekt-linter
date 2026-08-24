# Relatório P0075 — saneamento determinístico do delta V6

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência e segregação

- contrato do assessment 0007: `9dcad86`;
- REDs independentes congelados: `7301a71`;
- contrato operacional P0075: `444d1a7`.

A implementou sem ler gate/assessment/lab; B endureceu o gate sem ler a produção; C
revisou a produção final sem ler os testes de B. Os dois REDs iniciais tinham sido
reproduzidos independentemente por B e C antes da correção.

## Materialização

V6 agora interpreta funções, tipos e reexports como multiconjuntos. O pareamento consome
uma ocorrência por igualdade, portanto uma duplicata acrescentada ou removida permanece
visível. Os seis vetores do delta são ordenados canonicamente:

- funções por `(name, params, return_type)`;
- tipos por `(name, kind_rank, members)`, com rank explícito das seis variantes;
- reexports por texto.

A ordem interna de `params` e `members` continua semântica e não é normalizada. A ordem
pública dos grupos em `describe()` não mudou.

## Linhagem

O prompt causal `prompt-stale.md` absorveu a nova semântica e mantém `Hash do Código`
`2cf86141`. A tentativa manual inicial registrou um hash de prompt incorreto; o fluxo
oficial `--fix-hashes` detectou e corrigiu o header para `4f4edb28`. Um dry-run posterior
retornou `Nothing to fix` e o auto-lint V1/V5/V7 ficou limpo.

## Gates finais

- gate 0007 endurecido: 6/6, zero ignorados;
- adversário final: 6/6, **NÃO REABRIR**;
- testes unitários: 622/622;
- fixtures: 83/83;
- assessments Git, registry, entidades, config/walker, prompt I/O e apresentação: verdes;
- refinamento: 10/10; selo segregado: 16/16;
- Rust tocado passa `rustfmt --check`;
- `git diff --check`: limpo;
- auto-lint V1/V5/V7: limpo.

O `cargo fmt --all --check` continua vermelho por formatação histórica disseminada fora
do escopo; nenhum desses arquivos foi reformatado. Permanece também o warning preexistente
`print_tree` em `ts_parser.rs`.

## Limite e parada

`describe()` mostra nome da função e nome/kind do tipo, não todos os campos usados no
desempate. O delta estrutural é completo e a mensagem é determinística; ampliar o detalhe
público exigiria contrato próprio. Nenhum merge, instalação ou release foi realizado.
