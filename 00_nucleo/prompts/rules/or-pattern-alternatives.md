# Prompt: regra V19 OrPatternAlternatives
Hash do Código: c6ade72d

Owner exclusivo: `01_core/rules/or_pattern_alternatives.rs`.

Contabilizar alternativas condensadas em or-patterns e tornar a multiplicidade visível.
Não tratar alternativas equivalentes como novos braços nem reparsear source.

## Critério observável

V19 reporta multiplicidade exata das alternativas recebidas, preserva evidência e é
invariante à ordem externa sem reclassificar braços simples.

## Precisão e limites da métrica

- V19 permanece `Info`, com uma nota por braço que tenha `or_alternatives > 1`.
- `or_alternatives` soma alternativas escritas em todos os grupos do padrão,
  conforme o contrato de `DecisionArm`; não mede combinações nem testes executados.
  No limite `u16::MAX`, apresentar a contagem como "pelo menos 65535".
- Mensagem: "Este braço agrupa alternativas. Executar o corpo não comprova
  exercitar cada alternativa; verificar os testes pertinentes." Acrescentar
  contagem de alternativas escritas nos grupos, snippet e guard presente/ausente.
  Exportar essa explicação comum como `REVIEW_NOTE` para a apresentação textual
  consolidada, preservando integralmente a mensagem individual da regra.
- Não afirmar "cobertura subestima N×", déficit de testes ou número de casos
  alcançáveis. Testes adjacentes e justificativas L0 não provam cobertura real.
- Agrupamentos legítimos como `"yaml" | "yml"` e complementos nominais de V16
  continuam permitidos; a nota não recomenda duplicar corpos para eliminá-la.

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-10-16 | Mensagem observacional, contagem escrita distinta de cobertura, contexto de guard e limite explícito | or_pattern_alternatives.rs |
| 2026-10-16 | Compartilha explicação comum com o relatório sem alterar mensagens individuais ou severidade | or_pattern_alternatives.rs |
