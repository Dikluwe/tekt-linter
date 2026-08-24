# Passo operacional 0080 — triagem segregada de V12/V13

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; PASS funcional, sem alteração de produção
> **Branch:** `codex/segregated-materialization`
> **Contrato permanente:** assessment 0011

## Objetivo

Executar o assessment 0011 sobre os classificadores puros de declarações em L4 e estado
global em L1, sem misturar defeitos de extração dos parsers.

## Execução

1. Congelar passo e assessment antes do gate.
2. B escreve no máximo seis propriedades sem ler produção.
3. C procura contraexemplos próprios após o primeiro gate, sem ler testes de B.
4. O orquestrador classifica resultados e preserva qualquer RED antes de correção.
5. Se tudo passar, executar suíte global, auto-lint e emitir relatório.

## Critérios de fechamento

- sete camadas, seis DeclarationKind e os dois estados de configuração cobertos;
- os 18 tokens V13, `is_mut` e casos próximos/imutáveis cobertos;
- ordem, multiplicidade e evidência preservadas;
- zero testes ignorados e produção intocada durante a triagem.

## Parada

Não fazer merge, instalação ou release.
