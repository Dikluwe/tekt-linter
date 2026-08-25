# Prompt: regra V20 RangePattern
Hash do Código: 228c7dc6

Owner exclusivo: `01_core/rules/range_pattern.rs`.

Detectar ranges em padrões decisórios cuja cobertura fica opaca. Preservar forma inclusiva,
limites e localização extraídos; zero heurística textual.

## Critério observável

V18 sinaliza somente IR classificada como range pattern e preserva limites/localização;
texto semelhante fora de padrão permanece silencioso.
