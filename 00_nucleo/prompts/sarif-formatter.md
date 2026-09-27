# Prompt: CLI e formatadores do linter
Hash do Código: f6040ed0

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

## Revisão 2026-09-26 — completude dos reparos

O alcance distingue sourceReadErrors, syntaxErrors, unsupportedLanguageFiles e emptySourceFiles, além de filesParsed e sourceFilesRead; os contadores derivam dos erros tipados, sem interpretar mensagens.

## Revisão 2026-09-26 — instrumentação de resolução

Rejeitar --emit-resolution combinado com --fix-hashes ou --update-snapshot, inclusive dry-run, para que nenhuma operação solicitada seja ignorada. O modo de instrumentação preserva quiet e não publica resolução parcial em análise incompleta.

## Revisão 2026-09-26 — localizações SARIF

Quando a linha do diagnóstico é zero (localização desconhecida ou achado no arquivo inteiro), preservar artifactLocation e omitir region; nunca inventar linha 1. Linhas positivas são preservadas. Colunas internas base zero são convertidas para base um com soma verificada; se não representáveis, omitir somente startColumn, sem panic ou wraparound. Testar achados mistos e erro real de raiz. Referência: https://docs.oasis-open.org/sarif/sarif/v2.1.0/os/schemas/sarif-schema-2.1.0.json (region exige coordenadas positivas).

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
