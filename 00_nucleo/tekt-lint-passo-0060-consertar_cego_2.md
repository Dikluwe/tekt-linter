# Laudo 0060 — consertar cego #2 (referência de caminho cross-crate fora do `use`)

**Onde roda**: cruza linter (clone do 0052) e lente (`tekt-cargo-dsm`). Continuação de 0058–0059.
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/consertar-cego-2.md`](prompts/consertar-cego-2.md)
**Camadas tocadas**: L3 (`rs_parser`: 2ª fase de `extract_imports` —
`collect_path_refs`, `try_emit_path_ref`, `is_local_or_std_first_segment`,
`scan_token_tree`). Doc de referência atualizado em `prompts/parsers/rust.md`.
**Nenhuma regra mudou** — é **escopo de extração**, não resolução de nome.

---

## O que o 0058/0059 deixou (e o que este laudo fecha)

O oráculo (0058) achou três cegos; o 0059 fechou #1 (alias) e #3 (dep renomeada) e
**verificou** o #2 como aberto. O #2: `collect_imports` só visitava
`use_declaration`/`extern_crate_declaration` — qualquer referência cross-crate por
**caminho qualificado fora do `use`** (`crate_x::ITEM` em expressão, tipo, atributo
ou macro) era invisível. A aresta, e qualquer V3/V9/V14 que ela implicava, sumiam.

O #2 é **conhecido** e a forma é **possível na linguagem** (caminho qualificado
inline é Rust cotidiano) — possibilidade, não prevalência, justifica fechar o
falso-negativo.

## A armadilha (preservada): não trocar falso-negativo por falso-positivo

O 0058 mostrou **0 só-linter** — o resolvedor nunca inventa aresta que o compilador
não vê. Alargar a extração **preservou isso**: todo caminho coletado resolve o 1º
segmento pelo **mesmo `classify_import`**, e só vira aresta se for membro first-party
ou dep externa de verdade. Caminho local (`crate::`/`self::`/`super::`/`Self`) e
stdlib (`std`/`core`/`alloc`) são excluídos **antes** de classificar
(`is_local_or_std_first_segment`).

## Conserto — 2ª fase de `collect_imports` (`collect_path_refs`)

### A — posições estruturadas
`scoped_identifier` (expressão / qualificação de chamada) e `scoped_type_identifier`
(tipo, incl. argumentos de genérico) viram candidatos; o caminho completo (`&'a str`
do buffer) resolve pelo `classify_import` existente. Caminhos aninhados reincidem no
mesmo 1º segmento → absorvidos pela dedup.

### B — atributo/macro (parte frágil, escopo honesto)
Conteúdo de `attribute_item`/`macro_invocation` vem como `token_tree`.
`scan_token_tree` varre os filhos directos por **inícios de caminho** — um
`identifier` seguido de `::` e não precedido por `::` — e resolve o **1º segmento**
(granularidade de crate). É o suficiente para V3/V14, que dependem só do 1º segmento.

### C — deduplicação
`seen` parte dos 1ºs segmentos dos `use` já emitidos; um path-ref cujo 1º segmento já
está em `seen` é descartado. Um crate visto por `use` **e** por caminho inline = uma
aresta; N path-refs ao mesmo crate = uma aresta. Path-refs emitem `ImportKind::Direct`.

## Fixtures bite-proof (`tests/fixtures.rs`)

Positivas (mordem `[V3]` só após o conserto; L2 `shell` → L4 `wiremod`, **sem `use`**):
- **`v03e_fail_pathref_expr`** — expressão `wiremod::go();` → `[V3]`.
- **`v03f_fail_pathref_type`** — tipo de retorno `-> wiremod::Thing` → `[V3]`.
- **`v03g_fail_pathref_attr`** — `#[arg(default_value_t = wiremod::N)]` → `[V3]`.

Negativas (guarda contra falso-positivo):
- **`v03h_pass_pathref_local`** — `crate::wire::Thing` inline → `[]`. **Bite-proof**:
  o módulo interno `wire` está mapeado L4, logo se a guarda `crate::` caísse o
  resolvedor produziria L2→L4 e um `[V3]` espúrio (provado: removi `crate` da guarda
  → fixture virou `["V3"]`; revertido → `[]`).
- **`v03i_pass_pathref_std`** — `std::cmp::max(1, 2)` inline → `[]`. Robustez: stdlib
  é isento a jusante por construção (V14 isenta std; V3 ignora `Unknown`).

Mais 8 testes unitários em `rs_parser.rs` (expr/tipo/atributo coletados; local/super/
std fora; dedup `use`+inline e inline+inline). Suíte: **486 unit + 53 fixtures verde**.

## Mutação (escopo alterado)

`cargo mutants --file rs_parser.rs --re 'collect_path_refs|try_emit_path_ref|is_local_or_std|scan_token_tree|extract_imports'`:
**27 mutantes, 19 mortos, 7 sobreviventes, 1 inviável, 0 timeouts.**

Os **veredito-críticos estão todos mortos**: extração ligada/desligada
(`collect_path_refs`/`extract_imports`/braços `scoped_identifier` e `token_tree`),
detecção de início de caminho (`identifier`+`::`), guarda local/std (ambos os
sentidos), emissão e dedup (`||`).

Os **7 sobreviventes não mudam veredito** (multiset de IDs):
- **3 de número de linha** (`row + 1` → `row * 1`, em `collect_path_refs` e
  `scan_token_tree`): afetam só a **posição reportada** — trilha "oráculo de
  posição/severidade", fora de escopo deste prompt.
- **4 da guarda `prev_is_colon`** (`i > 0` / `i - 1` no `scan_token_tree`):
  impedem que um **segmento intermédio** (`b` em `a::b::c` dentro de um `token_tree`)
  seja mal-emitido como crate — **precisão de sub-caminho da parte B**, não o veredito
  de nenhuma aresta estruturada. É o **residual nomeado** (ver abaixo).

## Validação pelo oráculo (re-run)

- **Reprodutor `oraculo/biteproof_pathref`** (0059): `a` usa `b` só no
  `b::Thing` (tipo + expressão), **sem `use`**. O linter agora **emite a aresta a→b**
  (`target_crate: "b"`, first-party) — **o cego #2 fecha no reprodutor**. (Antes:
  `collect_imports` não extraía nada.)
- **Lente** (`tekt-cargo-dsm`): linter=22, lente=21, **acordo=21, cego-linter=0**
  (mantido do 0059), **só-linter=1**.
- **Triagem do só-linter `lente_app → lente_infra`** (o prompt manda triar aresta
  nova): é **real**, não inventada. `04_wiring/app/src/erro.rs:87` usa
  `lente_infra::ErroAdaptador::JsonInvalido(...)` inline, **sem `use lente_infra`**,
  dentro de um `#[cfg(test)]`. `lente_infra` é dep real (`path = "../../03_infra"`).
  O linter (textual, vê todo o código) resolve a ref; a lente (grafo do compilador,
  não-test) **exclui legitimamente** arestas só-de-teste. O invariante "**o linter não
  inventa aresta**" **vale** — a aresta existe. É o **achado bom** que o prompt previu.
- **Self-lint do linter = 0** ✓ (critério primário). Rodar o linter **na lente** dá
  apenas **1 warning V12 pré-existente** (`enum ErroLente` declarado em L4) — V12 lê
  `declarations`, **não** `imports`; esta mudança não o toca. **0 nova violação** (a
  aresta nova L4→L3 é legal: wiring→infra).
- **Parte B na fonte real**: `02_shell/cli/src/args.rs` referencia `lente_catalogo`
  **só** por atributo (`#[arg(... help = lente_catalogo::HELP_GRAFO)]`,
  `#[command(about = lente_catalogo::ABOUT_CLI)]`), **sem `use lente_catalogo`** no
  ficheiro — a varredura de `token_tree` vê a ref. (No nível de crate a aresta
  `cli→catalogo` já existia via `use ... as cat` em `saida.rs`, então não adiciona
  aresta nova ao oráculo; mas prova a parte B na fonte real.)

## Estatuto do #2 (dito com precisão)

**FECHADO** para as **posições estruturadas** (expressão, tipo, genérico,
qualificação de chamada) e para **atributo/macro** ao nível de **crate** (1º segmento).

**Residual nomeado** (cego **mais estreito**, não o #2 inteiro re-aberto):
1. **Precisão de sub-caminho em `token_tree`**: para refs em atributo/macro só o nome
   do crate é capturado, não o caminho completo — logo o **subdir do V9 a partir de
   atributo** não é resolvido (precisaria de ≥3 segmentos). V3/V14 não são afetados.
2. **Caminhos em corpos de macro não-estruturados**: o que a grammar entrega como
   tokens crus sem expor `identifier :: identifier` permanece invisível.

Nenhum mascarado: ambos são da parte B (a frágil), declarada com escopo honesto.

## Critérios de Verificação

- [x] Pré-condição (0059 presente; `biteproof_pathref` existe; corpus verde).
- [x] Extração estendida a `scoped_identifier`/`scoped_type_identifier`/caminhos de
      tipo, resolvendo o 1º segmento por `classify_import`; local/`std` ficam fora.
- [x] Atributo/macro `token_tree` varrido; residual (precisão de sub-caminho;
      corpo de macro não-estruturado) **documentado como cego mais estreito**.
- [x] Dedup preservada (uso por `use` + path inline = uma aresta / uma violação).
- [x] Fixtures positivas (expr/tipo/atributo) mordem `[V3]` só após o conserto;
      negativas (local/`std`) não criam violação (local **bite-proof**).
- [x] Mutação re-rodada: **0 sobreviventes que mudam veredito** no escopo alterado
      (7 residuais = posição de linha + precisão de sub-caminho da parte B).
- [x] Oráculo: `biteproof_pathref` **fecha**; lente self-lint 0; **cego-linter 0**.
      *Nuance honesta*: só-linter passou de 0→**1** (`lente_app→lente_infra`),
      **triado como real** (ref em `#[cfg(test)]` que a lente exclui) — o invariante
      "linter não inventa aresta" vale; é o achado bom previsto, não regressão.
- [x] Laudo ao fim; estatuto do #2 dito com precisão; residual nomeado; nada mascarado.

## Histórico de Revisões

- 2026-06-08 — Conserto do cego #2: 2ª fase de extração (`collect_path_refs`) para
  refs cross-crate por caminho fora do `use` (parte A estruturada; parte B
  atributo/macro ao nível de crate; parte C dedup). Guarda local/std. 3 fixtures
  positivas + 2 negativas (local bite-proof). Mutação re-rodada (0 veredito-mudante).
  Oráculo: reprodutor fecha; só-linter 0→1 triado como real (`#[cfg(test)]`).
  Residual de precisão de sub-caminho/macro nomeado.
