# Prompt: composição do linter
Hash do Código: 07e23fdf

Owner exclusivo: `04_wiring/main.rs`.

Compor config, adapters, parsers, regras e shell respeitando L0→L1→L2→L3→L4. O wiring
pode paralelizar trabalho independente, mas ordena resultados antes de apresentar e mantém
falhas de infraestrutura distintas de violações. Nenhuma política de domínio nasce em L4.

## Critério observável

O binário compõe todas as portas/adapters, ordena saída independentemente de Rayon e mantém
erro de infraestrutura distinto de violação; testes CLI e auto-lint cobrem o pipeline.
