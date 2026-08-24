# Passo operacional 0081 — fechamento do insumo normativo cego

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
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
