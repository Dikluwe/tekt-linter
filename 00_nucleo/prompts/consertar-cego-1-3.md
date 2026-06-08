# Prompt: consertar cego #1 (alias) e #3 (dep renomeada) + verificar #2

> Numere em sequência (provável 0059) e salve em `00_nucleo/prompts/` do **linter**.
> Cruza linter (clone do 0052) e lente (`tekt-cargo-dsm`). Continuação de 0058.

## Contexto

O oráculo (0058) achou três cegos de resolução do linter, todos na direção do falso
negativo. Dois são o **mesmo** problema — o primeiro segmento do import não é o crate
real — por dois caminhos:

- **#1 alias no `use`**: `use lente_catalogo as cat;` — o sufixo `" as cat"` não é
  removido antes de resolver o 1º segmento → cai em `LocalItem` → aresta invisível.
- **#3 dep renomeada**: `[dependencies] y = { package = "x" }` com `use y::Item;` — o
  1º segmento é a chave `y`, não o pacote `x` → não casa membro → `Unknown`.

São **completude do resolvedor**, não regra nova: fazer `classify_import` ver o crate
real por trás do nome de superfície. O cego **#2** (referência de caminho fora do
`use`) é de outra natureza (escopo de extração) e fica para o próprio prompt — este
termina **verificando** que o #2 segue intacto e isolado.

Nota importante: completar o resolvedor pode tornar visível uma aresta antes
silenciosa que **viola** (uma inversão que estava oculta). Isso é o comportamento
correto — o conserto é o que faz a violação oculta aparecer, não uma regressão. Na
lente, as duas arestas em questão são legais (L4→L2 e L2→L2), então o self-lint deve
seguir 0; mas em qualquer fixture/workspace, uma violação **nova** após o conserto é
triada como possivelmente-real, não como bug.

## Pré-condição

Clone do 0052; 0058 feito (o `--emit-resolution`, o crate `oraculo/` e a entrada
bite-proof `a→b` existem); corpus 0054–0057 verde; self-lint = 0.

## Conserto

### A. Cego #1 — alias no `use`
Onde o import é parseado/o 1º segmento é resolvido (`rs_parser.rs`; o 0058 localizou
o ponto — o `" as <ident>"` sobrevive e leva a `LocalItem`): **remover o sufixo
` as <ident>`** antes de resolver o 1º segmento. Depois disso, `use lente_catalogo as
cat;` resolve o crate `lente_catalogo` normalmente.

### B. Cego #3 — dependência renomeada
A renomeação é **por-crate** (o crate A renomeia `x` como `y`; outro pode não). Então
`crate_registry` (a peça do 0052) passa a registrar, **por membro**, o mapa de
renomeação lido do `[dependencies]` do `Cargo.toml` daquele membro
(`chave = { package = "real" }`). `classify_import` resolve o 1º segmento **através
desse mapa** (chave → pacote real → camada do membro) antes de cair em `Unknown`.

## Fixtures bite-proof (a entrada do oráculo vira fixture)

Cada conserto ganha uma fixture que **morde pelo veredito** — o import aliasado/
renomeado numa direção **proibida**, para que um V3 só apareça depois do conserto:

- **#1**: workspace multi-crate, arquivo **L2** com `use wiremod as w;` onde o alvo é
  **L4** → depois do conserto dispara `[V3]`; antes, sem V3 (alias invisível).
- **#3**: o workspace sintético `a→b` do 0058, ajustado para direção proibida
  (`a` em L2, `b` em L4, `b` renomeado) → `[V3]` depois do conserto; `Unknown` antes.

Estender `tests/fixtures.rs` afirmando o multiset de IDs. Re-rodar a **mutação** no
escopo alterado (`classify_import`/`crate_registry`/`rs_parser`) e zerar os
sobreviventes que mudam veredito — os ramos novos têm de ser mordidos.

## Validação pelo oráculo (re-run)

- Lente: a aresta **`lente_app → lente_catalogo`** (#1, o alias) passa a resolver →
  **acordo 19 → 20**.
- Sintético: a aresta renomeada (#3) passa a resolver.
- Self-lint da lente segue **0** (as arestas novas são legais). Qualquer violação
  nova em outro lugar → triada como possivelmente-real, não presumida bug.

## Verificação do cego #2 (handoff — NÃO consertar aqui)

Depois de #1 e #3, re-rodar o oráculo na lente e **confirmar que o #2 segue aberto e
intacto**:

- **acordo = 20, cego-linter = 1** (eram 2; o #1 fechou).
- O único cego restante é **`lente_cli → lente_catalogo`** — a referência a
  `lente_catalogo::HELP_X` dentro de `#[arg(...)]`, sem `use`.
- Confirmar que #1/#3 **não tocaram** o #2: ele é ortogonal (escopo de extração — só
  `use`/`extern crate` é coletado — não resolução de nome). Re-caracterizar o
  mecanismo da fonte, para o prompt do #2 começar fundamentado.
- Registrar explicitamente: **#2 não foi consertado**; é o próximo prompt.

(Se o acordo não der exatamente 20/1, ou o cego restante não for o `lente_cli →
lente_catalogo`, isso é sinal — investigar antes de fechar, não ajustar o número.)

## Critérios de Verificação

- [ ] Pré-condição confirmada (0058 presente; corpus verde).
- [ ] #1 conserto: sufixo ` as <ident>` removido antes da resolução; alias resolve.
- [ ] #3 conserto: `crate_registry` lê o mapa de renomeação por-membro do `Cargo.toml`;
      `classify_import` resolve a chave renomeada antes de `Unknown`.
- [ ] Fixtures bite-proof (#1 e #3) em direção proibida, mordendo `[V3]`; harness
      afirma IDs + contagem.
- [ ] Mutação re-rodada no escopo alterado: 0 sobreviventes que mudam veredito.
- [ ] Oráculo re-run: lente acordo 19→20 (#1 fecha); sintético #3 fecha; self-lint 0.
- [ ] **Verificação #2**: oráculo na lente dá acordo=20, cego=1, e o cego é
      `lente_cli → lente_catalogo`; #2 re-caracterizado e marcado **não-consertado**.
- [ ] Laudo ao fim; nada mascarado.

## Fora de escopo (prompts seguintes)

- **Cego #2** (referência fora do `use` — atributo/macro/caminho inline): próprio prompt.
- **Corpus de projetos variados** — escalar o oráculo para achar mais cegos.
- **Contador de `Layer::Unknown`**; **oráculo de posição/severidade**; **merge com o
  `master` público**.

## Disciplina

Conserto de resolvedor com fixture que morde pelo veredito; mutação re-rodada no que
mudou; oráculo re-run para provar que a aresta fecha; o #2 verificado e entregue, não
consertado; laudo ao fim.
