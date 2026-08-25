# Prompt: regra V18 DeepPatternNesting
Hash do Código: 0c1aae71

Owner exclusivo: `01_core/rules/deep_pattern_nesting.rs`.

Sinalizar padrões decisórios cuja profundidade excede o limite configurado. A métrica é
estrutural, determinística e independente de formatação do source.

## Critério observável

V20 cruza somente o limiar estrutural configurado; permutação/whitespace não muda contagem
ou localização e outras métricas decisórias permanecem independentes.
