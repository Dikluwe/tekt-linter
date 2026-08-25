# Assessment 0035 — migração de Núcleo Tekt para TOML

**Estado:** A CONGELADO — produção proibida até B1–B4  
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
