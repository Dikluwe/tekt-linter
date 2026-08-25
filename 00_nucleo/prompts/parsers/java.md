# Prompt: parser Java para IR canônica
Hash do Código: be7122c5

Owner exclusivo: `03_infra/java_parser.rs`.

Traduzir árvore Java em `ParsedFile`, preservando imports, tipos, métodos, decisões e
localizações. Nenhum tipo de parser é exposto a L1.

## Critério observável

Fixtures Java preservam imports, tipos, métodos, decisões e linhas; erro sintático falha
integralmente e a API pública entrega apenas IR L1.
