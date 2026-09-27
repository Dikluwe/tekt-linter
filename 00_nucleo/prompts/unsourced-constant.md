# Prompt: regra V21 HardcodedContextualValue
Hash do Código: 0f9889c5

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

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
