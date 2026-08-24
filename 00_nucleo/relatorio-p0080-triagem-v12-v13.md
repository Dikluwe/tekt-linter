# Relatório P0080 — triagem segregada de V12/V13

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** PASS funcional; sem alteração de produção, merge ou instalação

## Evidência

- contrato P0080 e assessment 0011: `e58f7fc`;
- B escreveu o gate sem ler V12/V13;
- C revisou contrato, prompts e produção sem ler testes de B.

## Resultado

V12 passou a matriz de sete camadas, seis kinds e dois estados de
`allow_adapter_structs`. O toggle afeta somente Struct/Class; ordem, multiplicidade,
nome, kind, path, linha e nível permaneceram completos.

V13 passou escopo L1, `is_mut`, os 18 tokens causais, casos próximos sem substring,
precedência de `mut`, múltiplas ocorrências e determinismo integral. O limite declarado
de aliases profundos não foi reclassificado: extração/semântica de tipos fica fora deste
assessment puro.

## SPEC-GAP documental

O assessment inicialmente exigia os 18 tokens sem enumerá-los. B não podia completar a
cobertura nominal sem violar sua segregação e registrou o gap. C confirmou a lista pelo
prompt causal e testou todos. O assessment agora contém uma clarificação explícita, sem
reescrever retroativamente o conhecimento de B.

## Gates finais

- gate B: 6/6, zero ignorados;
- adversário C: 6/6, nenhum RED;
- testes unitários: 628/628;
- fixtures: 83/83;
- todos os assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- dry-run de hashes: nada a corrigir;
- teste novo passa `rustfmt --check`;
- `git diff --check`: limpo.

Permanece o warning preexistente `print_tree` em `ts_parser.rs`.

## Parada

V12, V13, seus prompts e toda a produção permaneceram sem alteração. Nenhum merge,
instalação ou release foi realizado.
