# Prompt: fachada dos contratos L1
Hash do Código: f68a81d5

Owner exclusivo: `01_core/contracts/mod.rs`.

Declarar somente a fachada nominal das portas L1. Sem adapters, I/O, regras ou composição.

## Critério observável

A fachada compila reexportando apenas contratos L1; análise de imports não encontra
implementação concreta, filesystem ou dependência L2/L3/L4.
