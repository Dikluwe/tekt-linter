# Prompt: classificação de import ciente de dependências no `tekt-linter` (conserto da opção 3)

**Onde roda**: no **clone do `tekt-linter`** (seu próprio projeto Cristalino).
**Criado em**: 2026-06-06
**Estado**: `PROPOSTO`
**Camadas tocadas**: **L3** (`resolve_layer` + um registro de membros) e **L4**
(montar e injetar o registro). **As regras L1 (V3, V14) NÃO mudam.**
**Pré-requisito**: 0051 (premissa confirmada na fonte + teste com controle).
**Metodologia**: como o linter é um projeto Cristalino, o conserto segue a
disciplina dele — um prompt na `00_nucleo/prompts/` do linter, **TDD**, cabeçalho
de linhagem nos arquivos mudados, e `crystalline-lint .` **no próprio linter = 0
violações** ao final (a auto-validação que o linter prega).

---

## O que o 0051 confirmou (a base do conserto)

- `resolve_layer` (`03_infra/rs_parser.rs`) só resolve `crate::`/`super::`; todo
  `use lente_*::` cross-crate vira `Layer::Unknown`.
- O V3 (`forbidden_import.rs`) tem `if target == Unknown { return false }` — cego,
  por construção, a direção **entre crates**.
- O V14 (`external_type_in_contract.rs`) só guarda o L1, e dispara falso nos
  `lente_*` (intra-L1) e no `Kind` (`use EnumLocal::*` lido como pacote).

A raiz é uma suposição embutida: "Unknown = crate externo, ignora". Vale num
modelo de um-crate-por-camada; quebra no multi-crate. **O conserto é a montante**:
ensinar o `resolve_layer` as dependências reais, para o `target_layer` vir certo.
As regras L1 já fazem o resto.

---

## A especificação (os 6 casos — escrevê-los como testes PRIMEIRO, TDD)

| # | Cenário | Import | Resultado esperado |
|---|---|---|---|
| 1 | first-party cross-crate, **proibido** | arquivo **L3** com `use lente_<l4>::X` (membro L4) | resolve **L4** → **V3 dispara** (hoje calado — o buraco) |
| 2 | first-party cross-crate, **permitido** | **L1** com `use lente_core::X` (membro L1) | resolve **L1** → V3 calado (L1→L1 ok) **e V14 calado** (alvo L1, não Unknown) |
| 3 | externo declarado no L1 | **L1** com `use serde::X` (dep externa declarada) | **V14 dispara** (externo não autorizado) — segue pegando externo real |
| 4 | tipo local (o `Kind`) | **L1** com `use EnumLocal::*` (**não é dependência nenhuma**) | **não classificado** como import a camada/externo → **V14 calado** (falso positivo some) |
| 5 | `std` preservado | **L1** com `use std::collections::HashMap` | tratamento atual preservado (não vira V14); `std::fs` I/O segue pego pelo **V4** (inalterado) |
| 6 | intra-crate preservado | `use crate::shell::X` (`shell`→L2) num **L1** | resolução por `module_layer` (inalterada) → **V3 dispara** (o controle do 0051) |

Casos **1 e 4** falham hoje; **2** dispara falso V14 hoje; **3, 5, 6** já estão
certos (são as não-regressões). Escrever os seis, ver o estado atual, implementar,
ver os seis verdes.

---

## A mecânica (na arquitetura do linter)

1. **Registro membro→camada** (L3, infra): nome do pacote → diretório → camada
   (via `[layers]`). É I/O (lê os `Cargo.toml` dos membros, anda no workspace).
   Pode espelhar o `enumerar_membros` que o `tekt-cargo-dsm` fez (0044): parse
   direto dos `Cargo.toml`, sem subprocesso novo se der.
2. **Dependências declaradas de cada crate** (L3): do `Cargo.toml`
   (`[dependencies]` **e** `[dev-dependencies]`, por causa dos imports em teste —
   ex. o `lente_filtro` num teste, visto no 0050). Saber quais nomes são
   first-party (dep `path` para membro) e quais externas (crates.io). É o que
   distingue `Kind` (não-dep) de `serde` (externa).
3. **`resolve_layer` ciente** (L3) — dado o 1º segmento relevante do path:
   - **first-party** (no registro) → camada do membro.
   - **dep externa declarada** → externo (`Unknown`; o V14 aplica no L1, como hoje).
   - **nem dep** (tipo local, ex. `Kind`) → **não é import inter-crate/externo** →
     não classificar (não gerar `target_layer` que dispare V14/V3).
   - **`crate::`/`super::`** → como hoje (`module_layer`).
   - **`std`/`core`/`alloc`** → preservar o tratamento atual (não vira V14; o V4
     cuida de I/O).
   A assinatura do `resolve_layer` muda (passa a receber o registro + o contexto
   de deps do crate). Mudança **L3**.
4. **Montagem e injeção** (L4): construir o registro uma vez do workspace e
   injetá-lo no parser. Mudança **L4**.
5. **As regras L1 (V3, V14) NÃO mudam** — elas já agem sobre `target_layer`; o
   conserto faz o `target_layer` vir certo a montante.

---

## Restrições

- **As regras L1 (V3, V14) NÃO mudam.** Se a tentação for editar uma regra L1,
  **parar e reportar** — é sinal de que o diagnóstico do 0051 estava incompleto.
- **Não regredir**: `std`, intra-crate (`crate::`/`super::`), e o **linter
  passando em si mesmo** (0 violações).
- **Disciplina do linter**: prompt na `00_nucleo/prompts/` dele, linhagem nos
  arquivos novos/mudados, TDD, suíte do linter verde.
- Aditivo onde der; comportamento das regras inalterado por construção (o conserto
  é só na resolução de camada).

---

## O que NÃO fazer

- Mexer nas regras L1 (V3/V14) — o conserto é no `resolve_layer` e no registro.
- Regredir `std` ou intra-crate.
- Deixar o linter falhar na própria auto-validação.

---

## Critérios de Verificação

```
Dado os 6 casos como testes
Quando implementado o resolve_layer ciente
Então os 6 passam (1 e 4 falhavam; 2 disparava falso V14; 3/5/6 não regridem)

Dado a suíte do linter
Então verde

Dado crystalline-lint . NO PRÓPRIO LINTER
Então 0 violações (a auto-validação — o critério mais importante do linter)

Dado o linter consertado instalado e rodado no tekt-cargo-dsm
Então:
  - V3 = 0 (continua, mas agora SIGNIFICATIVO — capaz de pegar cross-crate; a
    prova da capacidade são os casos no linter, não o projeto, que é disciplinado)
  - removendo os seis lente_* do [l1_allowed_external] (a whitelist do 0050):
    V14 NÃO dispara mais nos lente_* (resolvem para L1) e o Kind some → V14 = 0
    (ou só externo real, se houver) — confirma que a whitelist virou desnecessária
```

---

## Resultado esperado

- O **registro membro→camada** + a leitura de deps (onde ficaram na arquitetura do
  linter).
- O **`resolve_layer` ciente** (o diff) + a injeção no L4.
- Os **6 testes** (TDD) verdes.
- A **suíte do linter verde** + `crystalline-lint .` = **0** no próprio linter.
- A **cascata no tekt-cargo-dsm**: V3 = 0 (agora significativo); com a whitelist do
  0050 removida, V14 = 0 (lente_* resolvem para L1, Kind some); confirmação de que
  a whitelist ficou desnecessária.
- **Laudo** em `00_nucleo/lessons/0052-…` (do `tekt-cargo-dsm`, registro do fluxo):
  o conserto no linter, os 6 casos, a auto-validação, e a cascata.

---

## Cuidados

- **O conserto é a montante** — se for preciso mexer numa regra L1, é o sinal de
  parar e repensar.
- **`std` e intra-crate** são as regressões a vigiar (casos 5 e 6 são as redes).
- **`dev-dependencies`** contam (imports first-party em teste — ex. o do 0050).
- **O linter passa em si mesmo** — a auto-validação dele; linhagem nos arquivos
  novos/mudados, na convenção do linter.
- **A whitelist do 0050 sai do `tekt-cargo-dsm`** na confirmação da cascata: era
  contorno; o conserto a torna desnecessária. Mudança reversível e correta — e
  fecha o débito que o 0050 abriu de propósito.
- Distinguir, no registro, um membro real (`lente_*` com `Cargo.toml`) de um path
  qualquer — usar o nome do pacote, não o do diretório (o `tekt-cargo-dsm` largou
  os prefixos numéricos no 0050; o nome do pacote é o estável).

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-06-06 | Conserto da opção 3, com a premissa confirmada no 0051. Classificação de import ciente de dependências, **a montante no `resolve_layer` (L3)** — as regras L1 (V3, V14) não mudam. Três casos: dep `path` para membro → camada do membro (first-party); dep externa declarada → externo (Unknown, V14 aplica no L1); não-dependência (tipo local como `Kind`) → não é import a classificar. `crate::`/`super::` e `std` preservados. Mecânica: registro membro→camada (L3, lê `Cargo.toml` dos membros + `[layers]`, espelha o `enumerar_membros` do 0044), leitura de `[dependencies]`+`[dev-dependencies]` por crate, `resolve_layer` recebe o registro+contexto, L4 monta e injeta. TDD com 6 casos (1 first-party proibido L3→L4 → V3 dispara; 2 first-party permitido → V3/V14 calados; 3 externo real → V14 dispara; 4 `Kind` → V14 calado; 5 `std` preservado; 6 intra-crate preservado/controle). Cascata: V3 volta a enxergar direção entre crates, V14 para de disparar nos `lente_*`, `Kind` some — de uma correção a montante só. Auto-validação: `crystalline-lint .` no próprio linter = 0. Confirmação no `tekt-cargo-dsm`: instalar o consertado, remover os seis `lente_*` da whitelist do 0050, V14 = 0 (whitelist vira desnecessária). | (no clone do `tekt-linter`) `03_infra/rs_parser.rs` (resolve_layer) + novo registro membro→camada (L3); `04_wiring` (montar/injetar); prompt e linhagem na convenção do linter; (depois, no `tekt-cargo-dsm`) `crystalline.toml` (remover a whitelist 0050) na confirmação; `00_nucleo/lessons/0052-...` |
