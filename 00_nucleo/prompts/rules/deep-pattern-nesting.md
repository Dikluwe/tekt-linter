# Prompt: regra V18 DeepPatternNesting
Hash do Código: 9890cf6b

Owner exclusivo: `01_core/rules/deep_pattern_nesting.rs`.

Sinalizar padrões decisórios cuja profundidade excede o limite configurado. A métrica é
estrutural, determinística e independente de formatação do source.

## Critério observável

V20 cruza somente o limiar estrutural configurado; permutação/whitespace não muda contagem
ou localização e outras métricas decisórias permanecem independentes.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
