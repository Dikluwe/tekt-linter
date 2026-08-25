# Prompt: fachada da infraestrutura L3
Hash do Código: 41ecb845

Owner exclusivo: `03_infra/mod.rs`.

Declarar somente a fachada nominal dos adapters L3. Não conter política L1 nem composição L4.

## Critério observável

A fachada compila declarando adapters L3 sem executar composição; não contém regra de
domínio nem instancia o pipeline L4.
