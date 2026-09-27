# Prompt: parser Go para IR canônica
Hash do Código: 59a521d6

Owner exclusivo: `03_infra/go_parser.rs`.

Traduzir árvore Go em `ParsedFile`, preservando imports, declarações, decisões e localizações.
Falha de parse permanece erro observável, não IR parcial silenciosa.

## Critério observável

Fixtures Go preservam imports, declarações, decisões e localizações; erro sintático não
produz `ParsedFile` parcial nem expõe tipos do parser.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
