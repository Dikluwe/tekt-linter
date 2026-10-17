# Prompt: CLI e formatadores do linter
Hash do Código: b2cfe253

## Owner

`02_shell/cli.rs`, exclusivamente.

## Instrução

Definir argumentos e apresentar violações em text, JSON, SARIF e resumo N16. Validar
combinações de flags e mapear severidade para exit code. Preservar a ordem dos
resultados, exceto pela consolidação textual das notas informativas V19 descrita abaixo.

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

## Revisão 2026-10-16 — consolidação informativa V19

No relatório de texto, reunir somente V19 de nível `Info` em um bloco final
"Notas informativas V19", com totais de braços e arquivos e agrupamento por path
em ordem lexical. Preservar cada ocorrência, inclusive duplicatas, e a ordem
original dentro de cada arquivo. Os outros diagnósticos preservam sua ordem.

Exibir uma vez a explicação comum `rules::or_pattern_alternatives::REVIEW_NOTE`,
deixando explícito que agrupamentos legítimos não são defeitos nem exigem separar
corpos iguais. Nas entradas, remover somente esse prefixo exato e o espaço que
o separa do detalhe. Se o prefixo não existir, preservar a mensagem inteira.
Não interpretar a mensagem para inferir contagens, guard, equivalência ou cobertura.
Os totais contam ocorrências e arquivos, nunca alternativas ou testes pendentes.

Cada entrada preserva contagem, guard e snippet, sem truncar, e informa linha e
coluna base um. Linha zero é desconhecida; coluna com overflow é omitida. Cada
arquivo é identificado pelo path humano existente. Não criar bloco vazio nem
declarar ausência de achados quando houver notas.

SARIF mantém os resultados individuais, mensagens completas, localizações e `note`.
V19 configurado como Warning/Error/Fatal segue a apresentação normal, sem ser
incluído no bloco informativo. `--quiet`, `--analysis-scope` e `--fail-on`
preservam seus contratos; Info continua sem reprovar a execução.

Validar relatório misto, notas em vários arquivos, multiplicidade, mensagens
alternativas, localizações limítrofes, texto/SARIF/quiet no CLI e severidade elevada.

## Revisão 2026-10-16 — referência informativa V19

Adicionar `--v19-reference <SARIF>` ao lint normal text/SARIF, exigindo V19 nos
checks e rejeitando subcomandos, reparos, emit-resolution e n16-summary. A opção
não escreve a referência. Sem a opção, preservar o relatório textual existente.

O SARIF normal inclui `runs[0].properties.v19Reference`, envelope versionado
`tekt-v19-reference-v1` com `selected`, `complete` e `notes`. Cada nota Info V19
contém path relativo à raiz em URI lossless, mensagem integral, linha e coluna
interna base zero. A referência pode estar vazia. Somente envelopes dessa versão,
selecionados e completos, são aceitos; SARIF legado sem envelope deve ser regerado.
Rejeitar JSON malformado, múltiplos runs, campos ausentes/inválidos e paths não
relativos/canônicos. Referência sem V19 selecionado ou com análise incompleta não
equivale a referência vazia. Limite de leitura: 10 MiB (orçamento explícito de I/O).

Comparar somente notas Info V19, como multiconjuntos. Primeiro casar path e mensagem
exatos, ignorando linhas/colunas (movimentos e duplicatas preservados). Não colapsar
whitespace: espaços dentro de literais são evidência. Depois, entre os restantes,
casar apenas pares únicos no mesmo path/linha/coluna conhecidos como `updated`.
Restantes sem nota antiga não casada no arquivo são `new`; havendo ambiguidade,
usar `newOrUpdated`. Demais são `unchanged`. Não emparelhar por ordem ou proximidade.
Renomear arquivo cria identidade nova; mudanças simultâneas de texto e posição
podem ser ambíguas. Comparar a mesma raiz lógica e escopo; não inferir resolução,
correção ou cobertura a partir de notas ausentes.

Texto: acrescentar resumo de novas/alteradas/novas ou alteradas/sem alteração e
rótulos por entrada, mantendo todas as notas e detalhes. SARIF: mensagens e
localizações permanecem integrais; `result.properties.v19Change` e
`run.properties.v19Comparison` carregam os destaques e totais. O envelope exportado
sempre representa as notas atuais, sem carregar estados da referência anterior.

Nenhum estado muda a severidade, suprime achado ou altera `should_fail`; não há
meta de zerar notas nem cobrança de testes. Quiet preserva silêncio inclusive para
erros da referência. Testar multiconjuntos, deslocamentos, alterações, ambiguidade,
literais, paths, referência vazia/inválida/incompleta e integração text/SARIF/quiet.
