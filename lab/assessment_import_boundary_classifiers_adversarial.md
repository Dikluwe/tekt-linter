# P0078 / Assessment 0010 — revisão adversarial independente (Agente C)

## Veredito

**RED — alegação 6.** A V9 usa `target_subdir` para decidir pertinência, mas não o inclui na violação. Dois imports com o mesmo `path` e linha, direcionados a subdiretórios internos distintos, produzem diagnósticos estruturalmente idênticos. A evidência não permite reconstruir qual fronteira interna foi atravessada.

As alegações 1–5 passaram nos ataques executados. Não há SPEC-GAP para o achado: o assessment exige que a evidência V9 siga as garantias de V3, e a revisão solicitada explicita `target_subdir` relevante como evidência a preservar.

Escopo lido: P0078 e assessment 0010 integrais, prompts causais de V3/V9, produção dos dois alvos e entidades/traits necessárias. Não foi lido `tests/import_boundary_classifiers_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e gate não foram alterados.

## Propriedades e mutações

| # | Prioridade | Ataque | Observação mecânica | Resultado |
|---|---:|---|---|---|
| P1 | P1 | V3: produto cartesiano das sete origens × sete destinos. | As 49 células coincidem exatamente com a matriz pública; destino `Unknown` e origens L0/Lab/Unknown são sempre isentos. | **PASS** — alegação 1. |
| P2 | P1 | V3: quatro `ImportKind`, cada um repetido como produção e test-origin, todos proibidos. | Guard desligado retorna só quatro imports de produção; ligado retorna oito. O kind não altera pertinência. | **PASS** — alegação 2. |
| P3 | P1 | V3: sequência `[NFC, NFD, NFC]`, com linhas sentinela. | Três `Error` na ordem 9/4/9; duplicata, source path, linha, camadas e import path aparecem sem normalização. | **PASS** — alegações 3 e evidência. |
| P4 | P1 | V9: sete origens × sete destinos × `target_subdir ∈ {None, porta, interno}`. | Só origem L2/L3 + destino L1 + `Some(interno)` viola. Todas as outras células são vazias. | **PASS** — alegação 4. |
| P5 | P1 | V9: portas `entities`, `é`, `e◌́` contra caixa, prefixo próximo, NFC/NFD e `None`. | Igualdade é textual exata; NFC/NFD permanecem distintos quando declarados separadamente, caixa/prefixo não passam e `None` fica isento. | **PASS** — alegação 5. |
| P6 | P0 | V9: quatro kinds com mesmo `path`/linha, alternando `internal_a`/`internal_b` e test-origin. | Guard e cardinalidade passam (2 desligado, 4 ligado), porém as duas violações de produção são iguais e nenhuma mensagem contém o subdir decisivo. | **RED** — completude da evidência na alegação 6. |

## Causa concreta e critério de fechamento

O filtro V9 consulta `import.target_subdir`, mas o mapper da violação formata somente `import.path`. `target_subdir` não aparece em nenhum outro campo da `Violation`. Portanto duas classificações L3 distintas podem colapsar na mesma evidência pública, embora a multiplicidade numérica seja preservada.

Critério mecânico de fechamento: para toda V9 emitida a partir de `Some(subdir)`, a mensagem deve conter literalmente o `target_subdir` que causou a rejeição, sem lowercase, normalização ou reconstrução a partir de `import.path`. Ordem, cardinalidade, guard, level e location devem permanecer inalterados.

## Reprodução

Probe preservado fora do auto-lint: `lab/assessment_import_boundary_classifiers_probe.rs.txt`.

```sh
cargo build --lib
cp lab/assessment_import_boundary_classifiers_probe.rs.txt \
  /tmp/assessment_import_boundary_classifiers_probe.rs
rustc --edition=2021 /tmp/assessment_import_boundary_classifiers_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/assessment_import_boundary_classifiers_probe
/tmp/assessment_import_boundary_classifiers_probe
```

Saída observada:

```text
PASS P1 V3 exhaustive 7x7 matrix
PASS P2 V3 guard and all ImportKind variants
PASS P3 V3 multiplicity/order/Unicode/evidence
PASS P4 V9 exhaustive origin/target/subdir matrix
PASS P5 V9 exact textual ports/None/Unicode/prefix
RED P6 V9 omits decisive target_subdir; internal_a/internal_b diagnostics are identical
```

## Matriz priorizada

| Ordem | Ataque | Falha procurada | Estado |
|---:|---|---|---|
| 1 | Mesmo path/linha, subdirs internos distintos em V9 | Evidência indistinguível | **RED confirmado** |
| 2 | V9 origem×destino×subdir | Célula omitida ou inventada | PASS |
| 3 | V3 origem×destino | Assimetria na matriz 7×7 | PASS |
| 4 | Guards × quatro kinds | Supressão além de test-origin | PASS |
| 5 | Duplicatas e permutações | Perda de multiplicidade/ordem | PASS |
| 6 | Caixa, prefixos e NFC/NFD nas portas | Igualdade textual indevida | PASS |
