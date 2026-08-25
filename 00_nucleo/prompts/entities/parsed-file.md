# Prompt: representação canônica ParsedFile
Hash do Código: 263d4e35

Owner exclusivo: `01_core/entities/parsed_file.rs`.

Modelar a IR independente de parser usada pelas regras: imports, tokens, interfaces,
decisões e metadados. Preservar localização/multiplicidade; zero tipos tree-sitter ou I/O.

## Critério observável

Testes de transporte preservam campos, ordem, multiplicidade e localização; a API pública
não expõe tipos de parser nem executa acesso externo.
