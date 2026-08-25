# Prompt: fachada dos use-cases L2
Hash do Código: 69e07abd

Owner exclusivo: `02_shell/mod.rs`.

Declarar a fachada nominal do shell. Não conter adapters concretos ou wiring de processo.

## Critério observável

A fachada compila expondo apenas use-cases/apresentação L2; não instancia parser, filesystem
ou processo e não declara política de regra.
