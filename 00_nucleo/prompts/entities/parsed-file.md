# Prompt: representação canônica ParsedFile
Hash do Código: 5e29b10e

Owner exclusivo: `01_core/entities/parsed_file.rs`.

Modelar a IR independente de parser usada pelas regras: imports, tokens, interfaces,
decisões e metadados. Preservar localização/multiplicidade; zero tipos tree-sitter ou I/O.

Transportar também `SourceRole` exatamente como recebido em `SourceFile`. A IR não
reclassifica o papel por AST, conteúdo ou cobertura.

## Critério observável

Testes de transporte preservam campos, ordem, multiplicidade e localização; a API pública
não expõe tipos de parser nem executa acesso externo.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 28, 32, 144. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `[layers]`, `[l1_ports]`, `[wiring_exceptions]`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
