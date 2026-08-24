# P0079 — revisão adversarial final (Agente C)

## Veredito

**NÃO REABRIR.** O RED de evidência da V9 foi fechado sem alterar pertinência, guard, cardinalidade ou ordem. O probe independente passou em 6/6 ataques, incluindo regressão pública completa da matriz V3.

Escopo lido: P0079 integral, prompt causal final de V9, produção final V9 e entidades/traits necessárias; V3 foi lida somente para executar a regressão solicitada. Não foi lido `tests/import_boundary_classifiers_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e gate não foram alterados.

## Ataques executados

| # | Ataque | Critério mecânico | Resultado |
|---|---|---|---|
| P1 | V9: sete origens × sete destinos × `target_subdir ∈ {None, porta, interno}`. | Somente origem L2/L3, destino L1 e `Some(interno)` não-porta gera violação. | **PASS** |
| P2 | Dois imports com mesmo path, linha e demais campos, variando apenas `internal_a`/`internal_b`. | Duas violações distintas; cada mensagem contém literalmente seu subdir causal. | **PASS** |
| P3 | Subdirs rejeitados NFC, NFD, caixa distinta, vazio e string com newline/tab/NUL/apóstrofo. | Todos são copiados literalmente; NFC/NFD/caixa produzem mensagens distintas e vazio aparece como `L1 ''`. | **PASS** |
| P4 | Comparar `None` com `Some("")`, depois declarar a string vazia como porta. | `None` sempre é isento; vazio viola quando ausente das portas e passa quando configurado como porta. | **PASS** |
| P5 | Quatro `ImportKind`, produção/test-origin, duplicata e ordem sentinela. | Guard desligado conserva linhas `[9,7]`; ligado conserva `[9,4,7,9]`; duplicata, rule id, `Error`, source path e coluna 0 permanecem. | **PASS** |
| P6 | V3 pela API pública: matriz completa sete origens × sete destinos. | As 49 células mantêm a pertinência anterior e diagnósticos V3 preservam nível/linha. | **PASS** |

## Evidência da correção

V9 agora usa `filter_map` para transportar conjuntamente `(import, subdir)` depois de verificar `Some(subdir)` e ausência em `L1Ports`. O mapper da violação recebe exatamente o valor que participou da decisão e o inclui no `format!` junto ao import path.

Isso mantém `None` fora da criação da violação e evita reconstrução do subdir a partir do path. A ordem continua sendo a ordem de entrada filtrada, e `ImportKind` permanece transparente.

## Reprodução

Probe preservado fora do auto-lint: `lab/p0079_adversarial_final_probe.rs.txt`.

```sh
cargo build --lib
cp lab/p0079_adversarial_final_probe.rs.txt /tmp/p0079_adversarial_final_probe.rs
rustc --edition=2021 /tmp/p0079_adversarial_final_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/p0079_adversarial_final_probe
/tmp/p0079_adversarial_final_probe
```

Saída observada:

```text
PASS P1 V9 exhaustive pertinence matrix
PASS P2 same path/line retains distinct target_subdir evidence
PASS P3 literal Unicode/NFC-NFD/case/empty/control subdirs
PASS P4 None exemption and empty-string identity
PASS P5 guard/kinds/multiplicity/order/location/level
PASS P6 public V3 7x7 regression
```

## Limite residual

O probe assume `target_layer` e `target_subdir` já resolvidos, como exige o contrato puro. Resolução de imports e configuração de portas continuam pertencendo à fronteira L3 e não foram reimplementadas nesta revisão.
