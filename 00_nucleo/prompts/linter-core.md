# Prompt: composição do linter
Hash do Código: 1186a2d4

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

## Revisão 2026-09-26 — completude dos reparos

Reparos de hashes e snapshots são bloqueados com saída 2 quando houver V0 ou PARSE, inclusive em dry-run e com checks restritos. Raiz inválida no formato N16 produz diagnóstico normal, nunca panic. Transportar contagens tipadas de parsing para o relatório de alcance.

## Revisão 2026-09-26 — instrumentação de resolução

No modo --emit-resolution, falhas de leitura ou parsing impedem publicação de resultados parciais: stdout vazio, diagnóstico em stderr e saída 2. Com --quiet, manter somente o exit code. Violações arquiteturais não impedem instrumentação de fontes completamente parseadas.

## Revisão 2026-09-26 — raiz independente dos checks

Validar a raiz local antes de carregar configuração ou criar adapters, para qualquer seleção de checks. Raiz inexistente ou arquivo em lugar de diretório produz V0 no formato normal solicitado, saída 1 e silêncio em quiet. Em --emit-resolution, raiz inválida produz somente diagnóstico em stderr e saída 2, nunca texto em stdout JSONL. Diretório existente vazio continua válido.

## Revisão 2026-09-26 — completude do resumo N16

Antes de publicar n16-summary, exigir leitura e parsing completos das fontes selecionadas. Se houver erro de leitura ou fonte não parseada, sair com código 2, não publicar estatísticas em stdout e apresentar os diagnósticos em stderr; quiet suprime ambos os canais. Falhas não são inferidas da severidade configurada. Diretório vazio válido continua produzindo resumo vazio com sucesso.

## Revisão 2026-09-26 — ausência versus erro de configuração

Carregar configuração via porta L3 load_optional, sem Path::exists que suprime erros de inspeção. Config padrão realmente ausente permite defaults; config explícita ausente sai 2; configuração existente inválida ou ilegível sai 1. Erros de carregamento e ausência explícita respeitam --quiet, sem alterar o exit code.
