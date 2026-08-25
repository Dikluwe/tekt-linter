# Prompt: parser Elixir para IR canônica
Hash do Código: ac5222d8

Owner exclusivo: `03_infra/elixir_parser.rs`.

Traduzir árvore Elixir em `ParsedFile`, preservando imports, declarações, decisões e
localizações. Grammar/tree-sitter não atravessa a fronteira L3.

## Critério observável

Fixtures Elixir preservam imports, declarações e decisões na IR com linhas corretas;
source inválido retorna erro e nenhum tipo tree-sitter aparece na API L1.
