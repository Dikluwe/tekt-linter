# Prompt: parser Go para IR canônica
Hash do Código: PENDING_P0106

Owner exclusivo: `03_infra/go_parser.rs`.

Traduzir árvore Go em `ParsedFile`, preservando imports, declarações, decisões e localizações.
Falha de parse permanece erro observável, não IR parcial silenciosa.
