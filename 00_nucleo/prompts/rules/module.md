# Prompt: fachada das regras L1
Hash do Código: aa3f0b6a

Owner exclusivo: `01_core/rules/mod.rs`.

Declarar somente a fachada nominal das regras puras. Não executar pipeline nem importar L2/L3.

## Critério observável

A fachada compila declarando módulos de regras L1; não contém dispatch, adapter ou importação
de shell/infra, verificável pelos gates de fronteira.
