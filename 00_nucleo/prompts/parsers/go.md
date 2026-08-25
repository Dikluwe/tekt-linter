# Prompt: parser Go para IR canônica
Hash do Código: 84f19573

Owner exclusivo: `03_infra/go_parser.rs`.

Traduzir árvore Go em `ParsedFile`, preservando imports, declarações, decisões e localizações.
Falha de parse permanece erro observável, não IR parcial silenciosa.

## Critério observável

Fixtures Go preservam imports, declarações, decisões e localizações; erro sintático não
produz `ParsedFile` parcial nem expõe tipos do parser.
