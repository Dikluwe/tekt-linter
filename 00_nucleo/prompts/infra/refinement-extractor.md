# Prompt: extração semântica de refinamento
Hash do Código: 2e4a23ea

Owner exclusivo: `03_infra/refinement_extractor.rs`.

Extrair observações canônicas dos bytes fornecidos, preservando ausência, opacidade e
erro de parse como estados distintos. Não consultar Git nem decidir veredito.

## Critério observável

Bytes equivalentes geram serialização idêntica; missing, opaque e parse error permanecem
distintos e nenhuma chamada Git participa da extração.
