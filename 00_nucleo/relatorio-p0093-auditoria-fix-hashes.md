# Relatório P0093 — auditoria de planejamento e execução fix-hashes

**Data:** 2026-08-25
**Branch:** `codex/audit-fix-hashes-planning`
**Baseline:** `6a325dc`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Escopo e saneamento

O lote auditou o caso de uso L2 `fix_hashes`, sua apresentação e o consumidor L4. O
writer L3 permaneceu fora do oráculo funcional. Antes dos gates, o L0 fechou cinco
`SPEC-GAP`: estados inválidos do plano, resultado ausente, semântica das duas escritas,
cardinalidade/continuidade e apresentação.

O contrato passou a exigir enums comparáveis, uma posição por V5, quatro causas de
indisponibilidade, dry-run total, ordem código/Hash A → prompt/Hash B e `PartialWrite`
explícito sem rollback inventado.

## Causalidade segregada

O gate B1 inicialmente inventou um port e foi rejeitado como `GATE-DEFECT`; foi corrigido
sem leitura de produção. O commit `b09aac5` congelou três gates independentes:

| Gate | SHA-256 | RED | GREEN |
|---|---|---:|---:|
| planejamento | `f1eb184536dc5526a6fdb402bb928e8906df6f96dad146754404075b27f3434b` | API ausente | 3/3 |
| execução | `24cc0a2c072c6c95bfbdbb1d1d6c951172c1aa4e106c27992b3b7d40829181a4` | API ausente | 4/4 |
| apresentação | `bbc5a7f9c9294e6d92c0b9dc6b9c4c61a081ffc06566b0802ed7b13609a4c223` | API ausente | 5/5 |

O confronto encontrou `Option`s/booleanos contraditórios, perda de cardinalidade por
`filter_map` e descarte do erro de metadata. O commit `895a378` corrigiu estritamente o
RED congelado.

## Matriz Tekt

| Responsabilidade | Camada | Evidência |
|---|---|---|
| violações V5 | L1 | entrada somente leitura |
| port, estados, plano, execução e apresentação | L2 | `02_shell/fix_hashes.rs` |
| hashes e escritas atômicas isoladas | L3 | adapter chama primitivas existentes |
| instanciação, injeção e reanálise | L4 | `04_wiring/main.rs` |
| oráculo black-box | B1/B2/B3 | três assessments congelados |

L2 não acessa filesystem nem importa L3. L3 não decide sucesso composto. L4 não
reimplementa a política.

## Validação e fechamento

- 628 testes unitários e todas as integrações/fixtures: PASS;
- gates P0093: 12/12 PASS;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- `--fix-hashes --dry-run --checks v5`: `Nothing to fix`;
- hashes L0 conferidos e `git diff --check`: PASS;
- adversário final: `PASS — READY WITH RESIDUAL AUDIT`.

Resíduos: não há rollback composto entre os dois arquivos; o exit code de
`PartialWrite` não está normatizado e pode ser zero se não restar V5; a falha entre as
duas escritas é coberta por spy, não por fixture end-to-end com I/O real. Nenhum desses
pontos contradiz o L0 atual. Não houve merge ou push.
