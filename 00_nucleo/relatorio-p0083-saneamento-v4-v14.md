# Relatório P0083 — saneamento de V4 e V14

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Resultado:** PASS

## Alterações

V4 não exigiu mudança funcional. Seu L0 passou a incluir C, C++, Zig, Go, Java e Elixir,
e o gate foi corrigido para avaliar near misses contra a tabela completa. Um novo ataque
prova que tabelas de linguagens diferentes não vazam entre si.

V14 manteve a semântica evoluída de autorização por pacote e item. O L0 passou a declarar
essa granularidade, a política de imports de teste e as isenções intra-crate. A produção
foi alterada somente para:

- emitir o nome do pacote, em vez do path completo, na violação;
- preservar `@scope/pkg` ao receber um subpath npm scoped.

Os hashes normativos finais são:

- V4: `b28dcdd672fce804b371137611151508870607b36122ace262fbf0edbb6650d6`;
- V14: `1ffd15999df65e7a82b8c1302e5e06741de2b7d945e6c837b65ecbb3fb16a6bf`.

## Validação

- gate V4: 7/7;
- gate V14: 9/9;
- testes unitários: 628/628;
- fixtures: 83/83;
- demais gates de integração: PASS;
- hashes em modo seco: `Nothing to fix`;
- auto-lint V1/V5/V7: `No violations found`;
- `rustfmt --check` dirigido e `git diff --check`: PASS.

Permanece apenas o aviso preexistente de `print_tree` não utilizada em
`03_infra/ts_parser.rs`. Nenhum merge, instalação ou release foi realizado.
