# Prompt: extração semântica de refinamento
Hash do Código: 2a568694

Owner exclusivo: `03_infra/refinement_extractor.rs`.

Extrair observações canônicas dos bytes fornecidos, preservando ausência, opacidade e
erro de parse como estados distintos. Não consultar Git nem decidir veredito.

## Critério observável

Bytes equivalentes geram serialização idêntica; missing, opaque e parse error permanecem
distintos e nenhuma chamada Git participa da extração.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
