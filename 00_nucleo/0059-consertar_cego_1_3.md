# Laudo 0059 — consertar cego #1 (alias) e #3 (dep renomeada); verificar #2

**Onde roda**: cruza linter (clone do 0052) e lente (`tekt-cargo-dsm`). Continuação de 0058.
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/consertar-cego-1-3.md`](prompts/consertar-cego-1-3.md)
**Camadas tocadas**: L3 (`rs_parser::classify_import`/`first_segment`,
`crate_registry::parse_manifest`+`MemberCrate`) e L2 (`cli::format_resolution`
`first_segment` do `--emit-resolution`). **Nenhuma regra mudou** — é completude do
resolvedor.

---

## O que o 0058 achou (e o que este laudo fecha)

Dois dos três cegos são o **mesmo** problema — o 1º segmento do import não é o crate
real — por dois caminhos:

- **#1 alias no `use`**: `use lente_catalogo as cat;` — o sufixo `" as cat"` não era
  removido antes de resolver o 1º segmento → `LocalItem` → aresta invisível.
- **#3 dep renomeada**: `[dependencies] y = { package = "x" }` com `use y::Item;` — o
  1º segmento é a chave `y`, não o pacote `x` → não casa membro → `Unknown`.

Ambos são **completude do resolvedor**, não regra nova. O **#2** (referência fora do
`use`) é de outra natureza (escopo de extração) — **verificado intacto**, não consertado.

## Conserto

### #1 — alias no `use`
`first_segment` (em `rs_parser.rs` **e** no emit `--emit-resolution` de `cli.rs`)
passa a cortar no primeiro de `::` **ou** `" as "`. `use lente_catalogo as cat;`
resolve o crate `lente_catalogo`. Os dois lugares: o parser corrige o `target_layer`
(veredito); o emit corrige a chave de aresta que o oráculo lê.

### #3 — dependência renomeada (por-membro)
`parse_manifest` passa a ler `chave = { package = "real" }` num mapa de renomeação;
`MemberCrate` carrega esse mapa por-membro; `classify_import` resolve o 1º segmento
**através do mapa do owner** (chave → pacote real → camada do membro) antes de cair
em `Unknown`. Sem rename, `real == seg` (comportamento inalterado).

> A renomeação é por-crate (A renomeia `x` como `y`; B pode não) — por isso o mapa
> vive no `MemberCrate` do owner, não global.

## Fixtures bite-proof (direção proibida → V3 só após o conserto)

- **`v03c_fail_alias`** (#1): workspace L2 `shell` com `use wiremod as w;`, `wiremod`
  é L4 → `[V3]`. Antes: `LocalItem`, invisível.
- **`v03d_fail_rename`** (#3): workspace L2 `a` com `alias = { package = "b" }`,
  `use alias::…`, `b` é L4 → `[V3]`. Antes: `Unknown`.

Harness (`tests/fixtures.rs`) afirma o multiset de IDs. Total: 48 fixtures.

## Mutação (escopo alterado)

`cargo mutants -j 4 --file rs_parser.rs --file crate_registry.rs`: **195 mutantes,
39 sobreviventes, 0 nos pontos do conserto.** `classify_import` (resolução de
rename), `first_segment` (strip de alias) e `parse_manifest` (leitura do rename)
estão **todos mortos** pelas fixtures `v03c`/`v03d` — os ramos novos são mordidos
pelo veredito. Os 39 restantes são o conjunto já reconciliado em 0056/0057
(`parse_layer_tag`, posição linha:coluna, `ImportKind`, `collect_type_param_names`,
`find_first_error_pos` — linhas deslocadas pelo código novo). **0 veredito-mudantes
novos.**

## Validação pelo oráculo (re-run)

- **Lente**: acordo **19 → 21**, cego **2 → 0**. O #1 fechou as arestas de alias.
- **Sintético #3** (`oraculo/biteproof`, dep renomeada): **fecha** (a→b vira acordo).
  Para isso o `--emit-resolution` passou a reportar **`target_crate`** — o crate
  resolvido pelo registry do próprio linter (pós-rename) — em vez do nome de
  superfície; o oráculo alinha por ele. (Sem rename, `target_crate == first_segment`.)
- **Self-lint da lente = 0** (as arestas novas são legais: L4→L2 e L2→L2).

> **Investigação (o prompt mandou não ajustar o número, e sim investigar).** A
> previsão era acordo=20, cego=1=`lente_cli→lente_catalogo` (suposto #2). O real é
> acordo=21, cego=0: `lente_cli→lente_catalogo` era **também #1** — há um
> `use lente_catalogo as cat;` em `02_shell/cli/src/saida.rs:19` que o triage do 0058
> **não viu** (olhou só as refs em atributo de `args.rs`). Ambas as arestas de catalogo
> eram alias; o #1 fechou as duas. O rótulo "#2" da `lente_cli→catalogo` no 0058 estava
> **errado** — corrigido aqui.

## Verificação do cego #2 (handoff — NÃO consertado)

Consequência da investigação: o **#2 não é exibido como aresta isolada na lente** —
todo uso cross-crate de `lente_catalogo` também passa por um `use ... as cat`, então
a aresta já existe pelo #1. O #2 só vira aresta-do-oráculo quando uma dependência é
usada **exclusivamente** por referência de caminho (nunca `use`/`extern crate`).

**Reprodutor concreto** (`oraculo/biteproof_pathref`): `a` usa `b` só no tipo de
retorno `b::Thing`, sem `use`. O linter **não extrai nada** (`collect_imports` só
visita `use_declaration`/`extern_crate_declaration`); o oráculo **morde** `a→b`. Isso
(i) mantém o oráculo confiável (ainda morde pós-conserto) e (ii) caracteriza o #2 da
fonte: é **escopo de extração**, ortogonal à resolução de nome — o #1/#3 não o
tocaram. **#2 segue aberto; é o próximo prompt.**

## Critérios de Verificação

- [x] Pré-condição (0058 presente; corpus verde).
- [x] #1: sufixo ` as <ident>` removido antes da resolução (parser + emit); alias resolve.
- [x] #3: `crate_registry` lê o mapa de renomeação por-membro; `classify_import`
      resolve a chave antes de `Unknown`.
- [x] Fixtures bite-proof (#1 e #3) em direção proibida, mordendo `[V3]`.
- [x] Mutação re-rodada: 0 sobreviventes que mudam veredito no escopo alterado.
- [x] Oráculo: lente acordo 19→**21** (#1 fechou as DUAS arestas de alias); sintético
      #3 fecha (via `target_crate` no emit); self-lint 0.
- [x] **#2** verificado e re-caracterizado: na lente não é aresta isolada (coberto por
      alias `use`); reprodutor `biteproof_pathref` morde; **marcado não-consertado**.
      (O número previsto 20/1 foi corrigido para 21/0 por investigação — o
      `lente_cli→catalogo` era #1, não #2; ver acima.)
- [x] Nada mascarado.

## Histórico de Revisões

- 2026-06-08 — Conserto do resolvedor: #1 (strip ` as`) e #3 (rename map por-membro).
  2 fixtures bite-proof (V3 em direção proibida). Mutação + oráculo re-run. #2
  verificado intacto e entregue como próximo prompt.
