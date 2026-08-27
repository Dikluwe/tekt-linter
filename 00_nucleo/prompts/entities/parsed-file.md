# Prompt: representação canônica ParsedFile
Hash do Código: 51d04dde

Owner exclusivo: `01_core/entities/parsed_file.rs`.

Modelar a IR independente de parser usada pelas regras: imports, tokens, interfaces,
decisões e metadados. Preservar localização/multiplicidade; zero tipos tree-sitter ou I/O.

Transportar também `SourceRole` exatamente como recebido em `SourceFile`. A IR não
reclassifica o papel por AST, conteúdo ou cobertura.

## Critério observável

Testes de transporte preservam campos, ordem, multiplicidade e localização; a API pública
não expõe tipos de parser nem executa acesso externo.
