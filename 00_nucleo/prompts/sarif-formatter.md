# Prompt: CLI e formatadores do linter
Hash do Código: 01de36a5

## Owner

`02_shell/cli.rs`, exclusivamente.

## Instrução

Definir argumentos e apresentar violações em text, JSON, SARIF e resumo N16. Validar
combinações de flags e mapear severidade para exit code sem reordenar resultados.

## Restrições

- apresentação não executa regras L1 nem muta paths;
- `--dry-run` exige operação mutável e operações incompatíveis falham;
- SARIF 2.1.0 e JSON permanecem válidos e determinísticos.

## Critérios

Parsers rejeitam combinações inválidas; cada formato preserva regra, severidade e
localização; quiet mantém apenas o código de saída.

## Revisão 2026-09-26

Aceitar --analysis-scope em lint text/SARIF, rejeitando combinação com reparos, emit-resolution ou n16-summary. Relatar arquivos lidos, parseados e erros, linguagens observadas, regras selecionadas, filtros de política, exclusões configuradas (sem alegar contagem dos arquivos excluídos), imports Unknown (externos ou não resolvidos) e limites. quiet não emite nada. --checks rejeita IDs desconhecidos e lista vazia, preserva all e v0.
