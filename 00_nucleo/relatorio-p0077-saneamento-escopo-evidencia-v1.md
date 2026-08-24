# Relatório P0077 — saneamento de escopo e evidência de V1

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência

- contrato do assessment 0009: `ce4a829`;
- REDs congelados: `8e5b54a`;
- contrato P0077: `70ff41e`.

A implementou sem ler assessment/gate/lab; B adaptou e endureceu o gate sem ler a
produção; C revisou a produção final sem ler testes de B.

## Materialização

`HasPromptFilesystem` agora entrega `Layer`. V1 usa essa informação já classificada e
não infere camada pelo path. L0, Lab e Unknown são isentos antes de qualquer consulta a
header, existência, path ou diretório estrito.

Em L1–L4, header ausente mantém a mensagem histórica. Header presente com prompt
inexistente produz mensagem distinta e inclui literalmente `prompt_path`. A política de
severidade por diretório estrito, cardinalidade, rule id e localização não mudou. V15
permaneceu intacta.

Os prompts causais de V1 e `HasPromptFilesystem` foram atualizados. O fluxo oficial
registrou hashes `a94bb0e5`/`46495747` para V1 e `4c753698`/`5b6a9b53` para a trait.

## Gates finais

- assessment 0009 endurecido: 6/6, zero ignorados;
- adversário final: 6/6, **NÃO REABRIR**;
- matriz V1 de sete camadas por três estados: verde;
- testes unitários: 626/626;
- fixtures: 83/83;
- todos os assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- dry-run de hashes: nada a corrigir;
- Rust tocado passa `rustfmt --check`;
- `git diff --check`: limpo.

Permanece o warning preexistente `print_tree` em `ts_parser.rs`. V1 preserva evidência
literal; escaping para terminais e formatos estruturados continua responsabilidade da
camada de apresentação, já coberta pelo assessment correspondente.

## Parada

Nenhum merge, instalação ou release foi realizado.
