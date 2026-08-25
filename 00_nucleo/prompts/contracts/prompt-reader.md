# Prompt: contrato PromptReader
Hash do Código: PENDING_P0106

## Owner

`01_core/contracts/prompt_reader.rs`, exclusivamente.

## Instrução

Declarar a porta pura usada por V1 e V5 para consultar existência e hash curto de um
prompt. A interface retorna `Option<String>` para ausência/falha sem conhecer I/O.

## Restrições

- zero filesystem e zero dependência de SHA em L1;
- não conhecer raiz concreta nem política de confinamento;
- manter a porta mockável por regras puras.

## Critérios

Mocks cobrem presente, ausente e hash conhecido; consumidores não precisam importar L3.
