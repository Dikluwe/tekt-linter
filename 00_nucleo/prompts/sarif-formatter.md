# Prompt: CLI e formatadores do linter
Hash do Código: PENDING_P0106

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
