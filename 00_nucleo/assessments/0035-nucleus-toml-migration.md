# Assessment 0035 — migração de Núcleo Tekt para TOML

**Estado:** READY WITH RESIDUAL AUDIT — parecer pré-merge favorável
**Data:** 2026-08-25  
**Passo:** P0107  
**Baseline:** `80807823f63339509bd325ae85e30bb1205d6725`

## Inventário hash-pinned

O manifesto `0035-nucleus-toml-inventory.tsv` fixa 26 arquivos: 8 `PRODUCTION`, 8
`FIXTURE`, 4 `NORMATIVE-DOC` e 6 `HISTORICAL-DOC`. Há exatamente um arquivo físico
`.tekt`, fixture `path.tekt`; produção não contém Núcleo Tekt real.

Mapa físico fechado:

```text
tests/fixtures/nucleus_wiring/00_nucleo/prompts/_nuclei/path.tekt
→ tests/fixtures/nucleus_wiring/00_nucleo/prompts/_nuclei/path.toml
```

Referências/pins da fixture `a.md` e `b.md` migram para `path.toml`. O SHA-256 raw dos bytes
permanece `76eca72e6d377a7e69723c95027093ad8e35f5d27d85b948d9f14d8316bcf89c`.
O valor `424df624a791b44060553caffea232f29315db10e58ab28c21f844e3575635fe`
é o digest efetivo v1 do núcleo sem dependências, não seu hash raw. O path lógico e hashes
de consumers mudam intencionalmente; o digest do próprio núcleo sem dependências não usa
seu path e portanto permanece `424df…`.

## Superfície prevista

- parser/path validator: sufixo canônico `.toml`;
- wiring: código apontando para `_nuclei/*.toml` é V26;
- inventário: `.tekt` legado é erro explícito, nunca arquivo ignorado;
- schema TOML, claims, dependências, DAG, limites e algoritmos permanecem;
- docs normativos convergem; P0105/Assessment 0033/relatório recebem nota superseded;
- P0106 e seus classificadores permanecem evidência histórica;
- resselo final inclui os owners dos prompts normativos alterados e qualquer pin
  transitivo descoberto pelo plano, sem antecipar writes.

## Hipóteses RED

| ID | Hipótese |
|---|---|
| R1 | trocar filtro para `.toml` pode aceitar TOML genérico como núcleo |
| R2 | `.tekt` legado pode desaparecer silenciosamente do walker |
| R3 | código pode usar `_nuclei/*.toml` como `@prompt` |
| R4 | rename pode alterar bytes/schema além da identidade de path |
| R5 | hashes raw, efetivos e pins podem divergir após rename |
| R6 | resselo pode escrever antes de inventário/grafo íntegros |
| R7 | documentação normativa pode manter dois formatos ativos |

## Classificação

- **RED:** R1–R7 se reproduzidos por gate ou implementação;
- **gate:** fixture/texto esperado desatualizado, sem ambiguidade de contrato;
- **SPEC-GAP:** qualquer dúvida sobre extensão, namespace ou compatibilidade. A decisão
  congelada é `.toml` simples, `_nuclei`, schema fechado e rejeição explícita de `.tekt`.

## Segregação

B1 cobre formato/namespace; B2 grafo/identidade; B3 wiring/V26; B4 hashes/transação.
Somente após os quatro gates congelados o lote produtivo pode começar. Projetos Tekt,
Bateia e tekt-cargo-dsm ficam fora da superfície de escrita.

## Fechamento P0107

| Gate | Evidência | Resultado |
|---|---|---|
| A | manifesto 26/26 hash-pinned | PASS |
| B1 | paths `.toml`, schema no namespace, legado observável | PASS |
| B2 | dependências/identidade/hash de path | PASS |
| B3 | wiring real e V26 código→núcleo | PASS |
| B4 | compatibilidade sem núcleo e transação herdada | PASS |
| rename | `path.tekt`→`path.toml`, similaridade Git 100% | PASS |
| raw bytes | SHA-256 `76eca72e…` antes/depois | IDÊNTICO |
| resselo | dry-run 2 pares; real; dry-run `Nothing to fix` | PASS |
| auto-lint | V1/V5/V7/V15/V26 | 0/0/0/0/0 |
| regressão | `cargo test` | PASS |
| inventário | arquivos físicos `*.tekt` fora de `target` | 0 |
| diff | `git diff --check` | PASS |

Commits segregados:

- `607fc0e` — inventário A;
- `2eeca36` — gates RED B1–B4;
- `5fcdc2d` — implementação e rename;
- `fc23047` — ADR-0022 Rev. 1 e registros;
- `601db8f` — resselo atômico.

## RED/gate/SPEC-GAP

- **RED R1 fechado:** `.toml` genérico dentro de `_nuclei` é parseado contra schema
  fechado e falha V26.
- **RED R2 fechado:** `.tekt` em qualquer posição do inventário gera diagnóstico legado;
  não é interpretado nem ignorado.
- **RED R3 fechado:** `@prompt` dirigido a `_nuclei` gera V26 para código produtivo.
- **RED R4 fechado:** rename Git 100% e SHA-256 raw idêntico.
- **RED R5 fechado:** gates de hash/transitividade e wiring passaram; fixture permanece
  limpa após atualização dos paths.
- **RED R6 fechado:** resselo só ocorreu após B1–B4 e inventário íntegro.
- **RED R7 fechado:** ADR e prompts normativos usam `.toml`; menções `.tekt` ativas servem
  exclusivamente para rejeição. P0105/P0106/0033 preservam cronologia classificada.
- **SPEC-GAP fechado:** ADR-0022 Rev. 1 decide `.toml` simples, namespace `_nuclei`, schema
  fechado e ausência de alias legado.

## Residual audit

`cargo fmt --check` global continua falhando em arquivos históricos já divergentes no
baseline P0106. Todos os arquivos Rust tocados por P0107 passam `rustfmt --check` isolado;
P0107 não introduziu novo delta. O auto-lint retorna exit 1 por achados históricos de
complexidade fora do escopo, mas não contém V1, V5, V7, V15 ou V26.

## Parecer pré-merge

P0107 está apto a merge: a mudança é uma migração de representação, não uma nova linguagem
ou expansão semântica. O merge deve ocorrer em operação separada deste fechamento. Tekt,
Bateia e tekt-cargo-dsm não foram modificados.
