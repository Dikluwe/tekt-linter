# Parecer adversarial P0091 — roteamento MultiParser

## Primeiro D

Veredito: `BLOCKED` apesar de produção funcionalmente aprovada.

- hashes pós-materialização não estavam resselados;
- B1 e B2 compartilhavam a mesma identidade;
- o `Ok` do B2 verificava somente parte de `ParsedFile`;
- havia whitespace no Assessment.

## Repetição D

Veredito: `READY WITH RESIDUAL AUDIT`.

- onze hashes L0 conferidos;
- mudança posterior aos gates em `linter-core.md` e `language-parser.md` limitada à
  metadata `Hash do Código`, formalmente resselada;
- B1 permaneceu inalterado;
- B2 independente usa arquivo e commit separados, nove spies e comparação integral;
- política e ports pertencem a L1, adapters concretos a L3, L2 está fora e L4 é wrapper
  transparente;
- `Unknown` falha fechado; não há I/O nem tipos L3 na política;
- regressão global, auto-lint, hashes e formatação dirigida passaram.

Residual aceito: o autor Git genérico não prova sozinho a segregação cognitiva, mas o
registro operacional e o conteúdo do gate não mostram leitura proibida ou oráculo comum.
