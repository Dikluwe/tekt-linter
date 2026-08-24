# Passo operacional 0081 — fechamento do insumo normativo cego

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; gap fechado por reteste cego limpo
> **Branch:** `codex/segregated-materialization`
> **Base:** SPEC-GAP documental do P0080

## Objetivo

Fechar a evidência do assessment 0011 sem alterar V12/V13: tornar o L0 normativo um
insumo autorizado e hash-pinned e repetir a cobertura dos 18 tokens com agente novo, sem
histórico da triagem anterior e sem acesso à produção.

## Execução

1. Absorver no protocolo segregado a exigência de contrato completo ou referência L0
   autorizada por caminho e SHA-256.
2. Fixar no assessment 0011 o prompt e a seção normativa de V13.
3. Criar agente novo sem contexto herdado. Ele valida o SHA-256, lê apenas contratos/L0
   autorizados e cria um gate novo, sem ler produção, testes anteriores ou lab.
4. O gate deve enumerar nominalmente os 18 tokens a partir de L0, testar cada um e provar
   que não importa constantes do alvo.
5. Executar suíte global, auto-lint e registrar relatório.

## Critérios de fechamento

- hash do insumo validado antes da leitura normativa;
- 18/18 tokens testados por nome em gate independente novo;
- V12/V13 e produção sem alteração;
- nenhum `SPEC-GAP` residual na cobertura nominal;
- nenhum merge, instalação ou release.

## Resultado

O contrato passou a autorizar insumos L0 por caminho, seção e SHA-256, sem liberar a
leitura do alvo L1. Um primeiro ensaio foi descartado porque uma formatação global poderia
ter lido mecanicamente arquivos proibidos. Ele não integra a evidência de fechamento.

Um segundo agente, criado sem contexto herdado, validou antes da leitura o SHA-256
`eb2ca06d26e0978c08e64aec0ed23c7848cf1b56f2b82547aa055e2a45e03c01`, extraiu os
18 tokens do L0 autorizado e produziu um gate independente. O resultado foi 18/18 tokens
nominais cobertos e 5/5 propriedades aprovadas, sem leitura dos alvos, testes anteriores,
lab ou histórico Git.

V12/V13 e o restante da produção não foram alterados. A suíte global, a verificação de
hashes, o auto-lint, o `rustfmt` dirigido ao gate e `git diff --check` passaram. Portanto,
não resta `SPEC-GAP` nominal neste lote.
