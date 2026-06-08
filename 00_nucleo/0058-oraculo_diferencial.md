# Laudo 0058 — oráculo diferencial linter × lente (a virada para fora)

**Onde roda**: cruza o **linter** (clone do 0052) e a **lente** (`tekt-cargo-dsm`).
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/oraculo-diferencial.md`](prompts/oraculo-diferencial.md)
**Primeira peça da virada "para fora"**: verificar arquitetura real, não regressão interna.
**Camadas tocadas**: linter — `--emit-resolution` (instrumentação L2/L4, **fora do selo
de veredito**); novo crate standalone `oraculo/` (não-membro do linter); exceção de
órfão do prompt.

---

## Por que é diferente de 0054–0057

A mutação acha a distinção que o código **já faz** mas nenhuma fixture testa. **Não**
acha a que o código **deixa de fazer** — a deriva real (a inversão L2→L4 da anamnese
veio do mundo, não de mutar o linter). O oráculo é uma **segunda computação
independente**: a lente resolve dependências pelo grafo do compilador (fork
`cargo-modules`); o linter, por análise textual (`classify_import`). Caminhos
independentes → não compartilham pontos cegos. Onde discordam, um dos dois é cego.

## Passo-zero (gate do prompt) — PASSA, demonstrado

- Fork `cargo-modules 0.27.0` instalado (com `uses_kind`).
- `lente_wiring::montar_grafo_workspace` produziu grafo real do próprio workspace da
  lente: **507 nós, 1749 arestas, 0 fantasmas** (36s). Não é inferência — rodado.
- Débito da lente nomeado: 35 testes `#[ignore]` (E2E git/workspace/fork, nunca em CI)
  — o caminho real funciona; só não é exercitado por padrão.

## A. Linter — `--emit-resolution` (instrumentação, fora do selo)

Novo modo: para **todo** import do workspace emite JSON Lines — `source`,
`source_layer`, `import`, `first_segment` (candidato a crate-alvo), `kind`,
`target_layer`, `target_subdir`, e **`is_unknown`** (os silenciosos, que o SARIF
esconde). Não muda regra nenhuma; pula o scan de prompts (roda em workspace sem
`00_nucleo`). Self-lint segue 0; suíte 478 unit + 46 fixtures verde.

## B. Harness — crate `oraculo/` (standalone no repo do linter)

Path-dep `lente_wiring`/`lente_core`; invoca o binário do linter por caixa-preta.
Excluído do self-lint. Fixa o cwd no workspace (a lente roda `cargo metadata` no cwd
herdado em parte da detecção-por-nome).

**Observável comum (v1): arestas cross-crate first-party `(crate-origem → crate-alvo)`**
— onde a independência é máxima e exatamente o que o linter cego punha em `Unknown`.
Escopo removido **simetricamente**: intra-crate, std/externas, alvos não-membros. A
camada é modo-comum (mesma projeção `[layers]` nos dois) — contexto, não sinal.

**Artefato de projeção achado e corrigido**: o `cargo-modules` chaveia o grafo pelo
nome do **alvo** (o bin `lente` do pacote `lente_app`), não do pacote. Sem reconciliar,
as arestas de `lente_app` viravam falso "só-linter". Fix: mapa `nome-de-alvo →
nome-de-pacote` via `cargo metadata` (targets[].name).

## C. Prova-de-mordida — PASSA

Workspace sintético com **dependência renomeada**: `a` faz `use alias::Thing;` onde
`alias = { package = "b" }`. O linter (textual) marca `is_unknown` (`alias` não é
membro); a lente (compilador) resolve `a → b`. O oráculo **reporta** `a -> b` como
cego-linter. Numa entrada sabidamente divergente, morde — não é cego.

## D. Primeira corrida real + triagem

### Workspace 1 — `tekt-cargo-dsm` (lente, 11 crates)

`linter=19, lente=21, acordo=19`. **0 só-linter. 2 cego-linter**, triados:

| Discordância | Natureza | Mecanismo (da fonte) |
|---|---|---|
| `lente_app → lente_catalogo` | **cego-linter** | `use lente_catalogo as cat;` — `classify_import` não tira o sufixo `" as cat"` antes de resolver o crate → cai em `LocalItem` → import invisível. **Cego a import de crate com alias.** |
| `lente_cli → lente_catalogo` | **cego-linter** | cli não tem `use`; referencia `lente_catalogo::HELP_X` só dentro de atributos `#[arg(...)]` (clap). O linter só extrai `use`/`extern crate`. **Cego a referência de caminho fora de `use`.** |

### Workspace 2 — bite-proof (sintético, multi-crate, compilável)

`1 cego-linter`: `a -> b` (**dep renomeada**, ver C). Dobra como prova-de-mordida e
como 2º workspace; o corpus real variado é a próxima trilha (fora de escopo).

## Achados (LISTADOS, não consertados aqui — cada um vira prompt próprio)

1. **Cego a import de crate com alias** (`use crate_x as y;`): o sufixo `" as y"` não
   é removido antes de resolver o 1º segmento → `LocalItem`, aresta invisível.
2. **Cego a referência de caminho cross-crate fora de `use`** (`crate_x::ITEM` inline,
   sobretudo em posição de macro/atributo): só `use`/`extern crate` é extraído.
3. **Cego a dependência renomeada** (`[dependencies] y = { package = "x" }`): o 1º
   segmento do import é a chave `y`, não o pacote `x` → não casa membro → `Unknown`.

Os três são o mesmo modo de falha da anamnese (uma forma real que o modelo textual
não representa), agora achados sistematicamente por uma segunda computação.

## Critérios de Verificação

- [x] Pré-condição: lente produz grafo real (507 nós, demonstrado); fork 0.27.0.
- [x] Linter emite resolução de todo import, incl. `Unknown`, parseável; fora do selo.
- [x] Harness roda os dois, projeta ao observável comum (remoção simétrica + canon
      bin→pacote), alinha e diffa — reprodutível por um comando.
- [x] Comparação primária no nível de **aresta**; camada como modo-comum anotado.
- [x] Oráculo **morde** (bite-proof: `a -> b` reportado).
- [x] Corrida em ≥2 workspaces; **toda** discordância triada (2 cego-linter + 1
      artefato de projeção corrigido + 1 cego na bite-proof; 0 só-linter restante).
- [x] Pontos cegos **listados como achados**, não consertados; cada um → prompt seguinte.
- [x] Self-lint = 0; suíte verde; nada mascarado.

## Fora de escopo (prompts seguintes)

- **Consertar** cada um dos 3 cegos (um prompt por forma: alias, path-ref, renomeada).
- **Corpus de projetos reais variados** — escalar o oráculo a muitos workspaces.
- **Contador de `Layer::Unknown`** em alvo real (detector mais barato).
- **Oráculo de posição** (linha:coluna/`PARSE`) e severidade — trilha à parte.
- **Merge com o `master` público** (multi-linguagem ⊕ conserto do 0052).

## Histórico de Revisões

- 2026-06-08 — Materialização: `--emit-resolution` no linter + crate `oraculo/`
  (path-dep lente). Bite-proof (dep renomeada) morde. Corrida em lente (19 acordo,
  2 cego-linter) + bite-proof. Artefato de projeção bin→pacote corrigido. 3 cegos do
  linter listados como achados (alias, path-ref, renomeada). Self-lint 0.
