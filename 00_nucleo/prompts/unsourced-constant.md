# Prompt: regra V21 HardcodedContextualValue
Hash do Código: PENDING_P0106

## Owner

`01_core/rules/unsourced_constant.rs`, exclusivamente.

## Instrução

Emitir V21 para constante não trivial que participa de escala binária entre fonte de
contexto e sumidouro geométrico sem `spec`, `rationale` ou `ref` fresca.

## Restrições

- regra pura; frescura entra pela porta L1;
- matching lexical por segmentos, nunca substring acidental;
- referências stale/unknown permanecem observáveis e fail-closed;
- preservar filtros explícitos de testes, tabelas e módulos de formato.

## Critérios

Os três eixos são necessários; triviais/isenções ficam silenciosos; candidato sem fonte
gera localização, snippet e razão estáveis; refs válidas silenciam somente o alvo.
