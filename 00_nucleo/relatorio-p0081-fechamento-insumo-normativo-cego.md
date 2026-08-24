# Relatório P0081 — fechamento do insumo normativo cego

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Contrato-base:** commit `79b9265`
**Resultado:** PASS; `SPEC-GAP` encerrado

## Mudança de protocolo

O verificador segregado pode receber valores normativos completos ou consultar um L0
explicitamente autorizado por caminho, seção e SHA-256. A autorização não se estende ao
alvo L1, aos testes existentes, ao lab ou ao histórico da implementação.

Para V13, o insumo autorizado foi
`00_nucleo/prompts/rules/mutable-state-core.md`, seção
`Tokens proibidos em posição static`, com SHA-256
`eb2ca06d26e0978c08e64aec0ed23c7848cf1b56f2b82547aa055e2a45e03c01`.

## Evidência aceita

Um agente novo, sem contexto herdado, validou o hash antes da leitura normativa e criou o
gate independente `tests/normative_v13_tokens_retest_clean.rs`. O gate não importa a lista
da produção e enumera nominalmente os 18 tokens derivados do L0 autorizado.

Resultados:

- 18/18 tokens normativos exercitados nominalmente;
- 5/5 propriedades aprovadas;
- escopo exclusivo de L1 confirmado;
- `static mut`, precedência, near misses, ordem e cardinalidade confirmados;
- nenhuma alteração nos classificadores V12/V13 ou em outra produção.

## Evidência excluída

O primeiro ensaio após a mudança foi descartado integralmente. Uma chamada global de
formatação poderia ter efetuado leitura mecânica de fontes proibidas, mesmo depois da
construção do gate. O descarte impede que uma execução ambígua sustente a conclusão.

## Validação global

- `cargo test --workspace --quiet`: PASS, incluindo 628 unitários, 83 fixtures e o novo
  gate com 5/5;
- reparo de hashes em modo seco: `Nothing to fix`;
- auto-lint V1/V5/V7: `No violations found`;
- `rustfmt --edition 2021 --check` dirigido ao gate: PASS;
- `git diff --check`: PASS.

Permanece apenas o aviso preexistente de função de teste `print_tree` não utilizada em
`03_infra/ts_parser.rs`. Ele não afeta o fechamento. Não houve merge, instalação ou
release.
