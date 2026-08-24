# Assessment 0008 — revisão adversarial independente (Agente C)

## Veredito

**RED — alegação 5.** `check_dangling_contracts` calcula corretamente o conjunto de traits pendentes, mas coleta diretamente da iteração de um `HashSet`; portanto a ordem pública não é determinística. No probe, 128 índices semanticamente idênticos produziram **128 ordens distintas**.

As alegações 1–4 e 6 passaram pelos contraexemplos executados. Não há SPEC-GAP para o achado: a alegação 5 exige expressamente ordem determinística e independência da ordem de inserção.

Escopo: assessment integral e produção dos quatro classificadores mais entidades necessárias. Não foi lido `tests/mechanical_rule_classifiers_assessment.rs`, nem mensagem ou artefato do Agente B. Produção e testes não foram alterados.

## Propriedades e mutações

| # | Prioridade | Ataque | Observação mecânica | Resultado |
|---|---:|---|---|---|
| P1 | P1 | V2: tabela exaustiva das 7 camadas × 2 estados de cobertura, com path Unicode decomposed. | Somente `L1 && !covered` gera uma V2 `Error`; path e posição `(1,0)` são preservados. | **PASS** — alegações 1 e 6. |
| P2 | P1 | V8: índice vazio e três aliens canônicos em ordem sentinela, incluindo NFC/NFD. | Vazio é identidade; há bijeção posicional 1:1, `Fatal`, path owned, posição `(0,0)` e evidência textual exata. | **PASS** — alegações 2 e 6. |
| P3 | P1 | V10: tabela exaustiva das 7 origens × 7 alvos. | Exatamente `origem ∈ {L1,L2,L3,L4} && alvo == Lab` gera uma V10 `Fatal`; as outras 45 combinações são isentas. | **PASS** — alegações 3 e 6. |
| P4 | P1 | V10: dois imports Unicode textualmente distintos, duplicação exata de um deles, um não-Lab e permutação total. | Três diagnósticos; duplicata preservada. Após comparar como multiconjunto `(line,message)`, as permutações são iguais; path/linha/texto permanecem completos. | **PASS** — alegações 4 e 6. |
| P5 | P1 | V11: `declared={A,B,C,é,e◌́}`, implemented e blanket sobrepostos, nível `Info`. | Resultado exato `{C,e◌́}`, sem colidir NFC/NFD; nível, rule id, location e evidência preservados. | **PASS** — diferença de conjuntos e alegação 6. |
| P6 | P0 | V11: construir 128 `ProjectIndex` com o mesmo conjunto de sete traits, variando inserção e a semente interna dos `HashSet`. | O multiconjunto é sempre correto, mas foram observadas 128 sequências de mensagens distintas. Uma execução equivalente pode mudar a ordem sem mudança semântica. | **RED** — determinismo da alegação 5. |

## Causa concreta e critério de fechamento

`dangling_contract::check_dangling_contracts` começa por `index.all_declared_traits.iter()`, filtra e chama `collect()` sem ordenar. A ordem de iteração de `std::collections::HashSet` não integra seu contrato e varia com o estado do hasher; a ordem de inserção tampouco deve ser usada como ordem pública.

Fechamento mecânico: materializar as traits pendentes, ordená-las por comparação textual total antes de criar as violações e confirmar que quaisquer permutações/inserções do mesmo conjunto produzem o mesmo vetor completo. A comparação deve continuar byte-sensitive para não fundir representações Unicode distintas.

## Reprodução

Probe não escaneável: `lab/assessment_mechanical_rule_classifiers_probe.rs.txt`.

```sh
cargo build --lib
cp lab/assessment_mechanical_rule_classifiers_probe.rs.txt \
  /tmp/assessment_mechanical_rule_classifiers_probe.rs
rustc --edition=2021 /tmp/assessment_mechanical_rule_classifiers_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/assessment_mechanical_rule_classifiers_probe
/tmp/assessment_mechanical_rule_classifiers_probe
```

Saída observada:

```text
PASS P1 V2 exhaustive layers x coverage
PASS P2 V8 empty identity/order/Unicode evidence
PASS P3 V10 exhaustive origin x target
PASS P4 V10 multiplicity/permutation/representation
PASS P5 V11 exact set difference/level/Unicode
RED P6 V11 produced 128 distinct orders for one semantic set
```

## Matriz priorizada

| Ordem | Ataque | Falha procurada | Estado |
|---:|---|---|---|
| 1 | Recriar o mesmo conjunto V11 sob inserções/hashers distintos | Ordem pública instável | **RED confirmado** |
| 2 | Duplicar e permutar imports Lab | Perda/invenção de multiplicidade ou evidência | PASS |
| 3 | Produto cartesiano origem × alvo em V10 | Camada omitida ou isenção indevida | PASS |
| 4 | Produto cartesiano camada × cobertura em V2 | Predicado incompleto | PASS |
| 5 | NFC/NFD nos quatro classificadores | Normalização nova ou evidência truncada | PASS |
| 6 | V8 vazio e sequência canônica | Violação inventada ou identidade reordenada | PASS |
