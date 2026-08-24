# Assessment 0007 — revisão adversarial independente (Agente C)

Escopo congelado: `00_nucleo/assessments/0007-provenance-rules.md` e produção de V5/V6/V7. Não foi lido `tests/provenance_rules_assessment.rs`, nem artefato do Agente B. Produção e testes não foram alterados.

## Veredito

**RED — reabrir a alegação 4 de V6.** Há dois contraexemplos reproduzidos: o delta perde multiplicidade e sua descrição depende da ordem de enumeração. As alegações 1, 2, 3, 5 e 6 passaram pelos ataques abaixo. Não encontrei SPEC-GAP necessário para decidir os casos executados: a própria alegação 4 exige explicitamente multiplicidade observável e independência da ordem de extração.

## Propriedades e mutações (máximo 6)

| # | Prioridade | Propriedade / mutação | Observação mecânica | Resultado |
|---|---:|---|---|---|
| P1 | P1 | V5: produto cartesiano de header/hash declarado/hash atual; mutar apenas caixa de um hash e usar path sentinela. | Só `Some(declared) + Some(current) + declared != current` produz exatamente uma V5; mensagem contém ambos os valores literais e location conserva `01_core/hash.rs`. | **PASS** — alegações 1, 2 e parte da 6. |
| P2 | P1 | V6: permutar `[a,b]` para `[b,a]`, mantendo exatamente as mesmas assinaturas. | `check` retorna vetor vazio. | **PASS** — alegação 3 para permutação top-level. |
| P3 | P1 | V6: alterar isoladamente nome/parâmetros/retorno de função; nome/kind/members de tipo; reexport. | Cada uma das sete mutações produz exatamente uma V6. | **PASS** — alegação 3 e igualdade estrutural da alegação 6. |
| P4 | P0 | V6: snapshot `[f]`, atual `[f,f]` (e, simetricamente, a mutação inversa). | `check` retorna vazio porque `contains` trata toda ocorrência de `f` como já presente; `compute_delta` fica vazio apesar da desigualdade dos vetores. Falso sucesso: mudança de multiplicidade é aceita. | **RED** — viola diretamente a alegação 4. |
| P5 | P0 | V6: snapshot vazio; atual `[a,b]` versus `[b,a]`. | Ambas produzem V6, mas as mensagens são respectivamente `Delta: +fn a, +fn b` e `Delta: +fn b, +fn a`. A ordem recebida atravessa `compute_delta` e `InterfaceDelta::describe`. | **RED** — viola determinismo/independência da ordem de extração na alegação 4. |
| P6 | P1 | V7: inventário com `é.md`, `e\u{301}.md`, `z.md`; referenciar `z.md` e também o decoy `./z.md`; nível `Error`. | Exatamente dois órfãos, em ordem do `BTreeSet`, paths distintos NFC/NFD, nível injetado preservado; o decoy não cria igualdade por normalização. | **PASS** — alegações 5 e 6. |

### Causa concreta dos REDs

`prompt_stale::compute_delta` implementa diferença como `filter(|x| !other.contains(x))`. Isso é diferença de conjuntos representados por vetores, não diferença de multiconjuntos: nenhuma ocorrência é consumida ao fazer o pareamento. Além disso, os vetores resultantes preservam a ordem de `current`/`snapshot`, e `InterfaceDelta::describe` os percorre sem ordenação canônica.

Uma correção deve satisfazer simultaneamente:

1. pareamento por multiplicidade (cada ocorrência do outro lado só pode cancelar uma ocorrência); e
2. ordenação canônica total antes da descrição, cobrindo todos os campos da assinatura, não apenas `name`, para que homônimos semanticamente diferentes tenham desempate estável.

## Reprodução

Probe preservado como artefato não escaneável: `lab/p0075_provenance_rules_probe.rs.txt`.

Executado a partir da raiz:

```sh
cargo build --lib
cp lab/p0075_provenance_rules_probe.rs.txt /tmp/p0075_provenance_rules_probe.rs
rustc --edition=2021 /tmp/p0075_provenance_rules_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/p0075_provenance_rules_probe
/tmp/p0075_provenance_rules_probe
```

Saída observada:

```text
PASS P1 V5 truth-table/exact bytes/path
PASS P2 V6 permutation equivalence
PASS P3 V6 all public fields observable
RED P4 V6 multiplicity collapsed: [f,f] accepted as [f]
RED P5 V6 description order-dependent: ... Delta: +fn a, +fn b != ... Delta: +fn b, +fn a
PASS P6 V7 exact representation/cardinality/order/level
```

## Matriz priorizada de ataques

| Ordem | Ataque | Falso sucesso / instabilidade procurada | Status |
|---:|---|---|---|
| 1 | Duplicar/remover ocorrência idêntica em funções, tipos ou reexports | V6 ausente apesar de vetores estruturalmente diferentes | **RED confirmado** |
| 2 | Permutar adições/remoções semanticamente iguais | Mensagem/delta muda apenas pela ordem de extração | **RED confirmado** |
| 3 | Mutar cada campo de assinatura isoladamente | Campo omitido do critério de igualdade | PASS |
| 4 | NFC/NFD, caixa e segmentos textuais de path/hash | Igualdade indevida por normalização em L1 | PASS |
| 5 | Permutar inventário e injetar nível não-default em V7 | Ordem/cardinalidade/nível instáveis | PASS |
| 6 | Ausências e igualdade/diferença exata em V5 | Violação vacuosa, duplicada ou com evidência perdida | PASS |
