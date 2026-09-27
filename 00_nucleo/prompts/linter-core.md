# Prompt: composição do linter
Hash do Código: 9c0266e0

Owner exclusivo: `04_wiring/main.rs`.

Compor config, adapters, parsers, regras e shell respeitando L0→L1→L2→L3→L4. O wiring
pode paralelizar trabalho independente, mas ordena resultados antes de apresentar e mantém
falhas de infraestrutura distintas de violações. Nenhuma política de domínio nasce em L4.
O subcomando explícito `init` é roteado antes do pipeline de lint: valida raiz e destino,
obtém inventário L3, aplica plano L1, publica via adapter e apresenta pelo shell L2.

## Critério observável

O binário compõe todas as portas/adapters, ordena saída independentemente de Rayon e mantém
erro de infraestrutura distinto de violação; testes CLI e auto-lint cobrem o pipeline.

## Revisão 2026-09-26

Compor política L1 após análise e antes do veredito normal. Reparos continuam usando achados intrínsecos sem supressão pela política. --analysis-scope é somente leitura. Config explicitamente solicitada inexistente falha; configuração padrão ausente mantém defaults.
