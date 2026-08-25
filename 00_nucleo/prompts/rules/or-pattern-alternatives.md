# Prompt: regra V19 OrPatternAlternatives
Hash do Código: 5588686f

Owner exclusivo: `01_core/rules/or_pattern_alternatives.rs`.

Contabilizar alternativas condensadas em or-patterns e tornar a multiplicidade visível.
Não tratar alternativas equivalentes como novos braços nem reparsear source.

## Critério observável

V19 reporta multiplicidade exata das alternativas recebidas, preserva evidência e é
invariante à ordem externa sem reclassificar braços simples.
