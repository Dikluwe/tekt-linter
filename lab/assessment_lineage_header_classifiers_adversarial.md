# Assessment 0009 — revisão adversarial independente (Agente C)

## Veredito

**RED — alegações 1 e 6.** V1 não consegue restringir-se a L1–L4 porque `HasPromptFilesystem` não expõe a camada; o mesmo estado inválido produz V1 também em L0, Lab e Unknown. Além disso, quando o header existe mas sua referência não existe, V1 omite o `prompt_path` da evidência e produz exatamente a mesma mensagem usada para header ausente.

As alegações 2–5 passaram nos ataques executados. Não classifico os achados como SPEC-GAP: o escopo de camadas e a completude da evidência estão explicitamente congelados no assessment. A limitação da trait explica o RED da alegação 1, mas não o torna ambíguo.

Escopo lido: assessment integral, prompts causais de V1/V15, produção dos dois alvos e entidades/traits necessárias. Não foi lido `tests/lineage_header_classifiers_assessment.rs`, nem mensagem ou artefato do Agente B. Produção e gate não foram alterados.

## Propriedades e mutações

| # | Prioridade | Ataque | Observação mecânica | Resultado |
|---|---:|---|---|---|
| P1 | P0 | V1 com header ausente nas sete camadas; o dublê carrega `Layer`, embora a trait não permita entregá-la à regra. | V1 é emitida nas sete camadas. L0, Lab e Unknown são três falsos positivos contra o escopo L1–L4. | **RED** — alegação 1. |
| P2 | P1 | Nas quatro camadas obrigatórias, produto cartesiano `header ∈ {None,Some}` × `exists ∈ {false,true}`. | Só `Some + true` passa; os outros três estados geram exatamente uma V1, nunca mais de uma, com rule id/path/posição corretos. | **PASS** — alegação 2. |
| P3 | P1 | Strict dir `01_core/contracts` contra descendente, o próprio diretório e prefixos próximos `contracts_extra`/`contract`. | Somente o próprio componente e seu descendente são `Fatal`; prefixos textuais próximos são `Error`. | **PASS** — alegação 3. |
| P4 | P1 | V15: produto cartesiano das sete camadas × cardinalidades 0–3. | Exatamente uma V15 `Error` somente para 2+ refs em L1–L4; todos os demais estados são vazios. | **PASS** — alegação 4. |
| P5 | P1 | V15 com `[é, e◌́, é]`, path Unicode e duas execuções idênticas. | Mensagem preserva quantidade 3, ordem, duplicata e representações distintas; path/posição são preservados e as execuções são iguais. | **PASS** — alegações 5 e 6 para V15. |
| P6 | P0 | V1 com header `00_nucleo/prompts/ausente-é.md` presente, mas `prompt_file_exists=false`; comparar com header ausente. | A violação não contém o path do prompt e sua mensagem é byte a byte igual à de ausência total de header. A evidência não identifica qual referência causal falhou. | **RED** — alegação 6. |

## Causas concretas e critérios de fechamento

### RED P1 — escopo não representável pela trait

`prompt_header::check` recebe apenas `prompt_header`, `prompt_file_exists` e `path`. Não há `layer()` em `HasPromptFilesystem`, e a função não possui outro parâmetro que represente o escopo. Assim, dois arquivos de camadas diferentes mas com os mesmos três valores são observacionalmente indistinguíveis para V1.

Critério mecânico: expor a camada no contrato puro (ou restringir explicitamente antes da chamada por um contrato congelado equivalente) e provar a tabela 7 × estados. A correção não deve inferir camada pelo path, pois isso duplicaria decisão de L3.

### RED P6 — causa perdida na mensagem

O predicado colapsa `header=None` e `header=Some(path), exists=false` em `has_valid_header=false`, depois usa uma única mensagem constante. O segundo estado já fornece `header.prompt_path`, mas ele não chega à evidência.

Critério mecânico: manter cardinalidade máxima 1, mas produzir evidência que distinga header ausente de referência inexistente e inclua literalmente o `prompt_path` no segundo caso, sem normalização.

## Reprodução

Probe não escaneável: `lab/assessment_lineage_header_classifiers_probe.rs.txt`.

```sh
cargo build --lib
cp lab/assessment_lineage_header_classifiers_probe.rs.txt \
  /tmp/assessment_lineage_header_classifiers_probe.rs
rustc --edition=2021 /tmp/assessment_lineage_header_classifiers_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/assessment_lineage_header_classifiers_probe
/tmp/assessment_lineage_header_classifiers_probe
```

Saída observada:

```text
RED P1 V1 false positives outside L1-L4: ["L0", "Lab", "Unknown"]
PASS P2 V1 header/exists truth table in L1-L4
PASS P3 V1 strict path component boundary
PASS P4 V15 exhaustive layer x cardinality
PASS P5 V15 order/duplicates/Unicode/evidence/determinism
RED P6 V1 missing-reference evidence omits prompt path and equals absent-header message
```

## Matriz priorizada

| Ordem | Ataque | Falso sucesso/positivo procurado | Estado |
|---:|---|---|---|
| 1 | Header inválido em L0/Lab/Unknown | V1 fora do escopo | **RED confirmado** |
| 2 | Header existente apontando para prompt ausente | Evidência sem identidade causal | **RED confirmado** |
| 3 | Tabela header × exists em L1–L4 | Estado inválido aceito ou duplicado | PASS |
| 4 | Prefixos próximos de strict dirs | `Fatal` por prefixo textual | PASS |
| 5 | V15 7 camadas × cardinalidade | Limiar/escopo incompleto | PASS |
| 6 | V15 duplicatas e NFC/NFD | Normalização, perda ou reordenação | PASS |
