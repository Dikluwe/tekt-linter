# Prompt: consertar cego #2 — referência de caminho cross-crate fora do `use`

> Numere em sequência (provável 0060) e salve em `00_nucleo/prompts/` do **linter**.
> Cruza linter (clone do 0052) e lente. Continuação de 0058–0059.

## Contexto e princípio

O oráculo (0058) achou três cegos; 0059 fechou #1 (alias) e #3 (dep renomeada) e
**verificou** o #2 como aberto, com reprodutor `oraculo/biteproof_pathref`. O #2:
`collect_imports` só visita `use_declaration`/`extern_crate_declaration`; qualquer
referência cross-crate por **caminho** fora do `use` — `crate_x::ITEM` em expressão,
tipo, atributo ou macro — é invisível. A aresta, e qualquer V3/V9/V14 que ela
implique, somem.

Por que consertar agora, sem dado de prevalência: o #2 é **conhecido** e a forma é
**possível na linguagem** (caminho qualificado inline é Rust cotidiano). Para um
verificador, isso basta — possibilidade, não frequência, justifica fechar um
falso-negativo. (Prevalência governa a busca por cegos **desconhecidos**, trilha à
parte; não esta decisão.)

## A armadilha: não trocar falso-negativo por falso-positivo

O 0058 mostrou **0 só-linter** — o resolvedor é correto (não inventa aresta que o
compilador não vê). Alargar a extração **tem de preservar isso**. Coletar caminhos
demais — tratar um caminho local (`crate::`/`self::`/`super::`/módulo local) ou `std`
como cross-crate — criaria aresta falsa e V3 espúrio. Regra: todo caminho coletado
resolve o 1º segmento pelo **mesmo `classify_import`**, e só vira aresta se resolver a
um membro/externo de verdade. Local e `std` continuam fora.

## Conserto

### A. Posições estruturadas (a parte tratável)
Estender a coleta, além de `use`/`extern crate`, para os nós que a grammar
`tree-sitter-rust 0.23` estrutura: `scoped_identifier` (expressão),
`scoped_type_identifier` (tipo), caminhos em `generic_type`/argumentos de tipo, e
qualificações de chamada. Para cada um, resolver o 1º segmento por `classify_import`;
emitir aresta/`Import` só se cross-crate first-party ou externo. Reusar a resolução
existente — não duplicar lógica de camada.

### B. Atributos e macros (a parte frágil — escopo honesto)
A referência da lente estava em `#[arg(... lente_catalogo::HELP_X)]` (atributo clap).
Conteúdo de atributo/macro vem como `token_tree` — pode não expor `scoped_identifier`.
Tratar: varrer `token_tree` de `attribute_item` e `macro_invocation` por sequências
`ident :: ident` e resolver o 1º segmento como em A. **Limite honesto**: caminhos
gerados **dentro** do corpo de uma macro (que a grammar não estrutura) podem
permanecer invisíveis — é um cego **residual mais estreito**. Se sobrar, **documentar
como residual conhecido**, não declarar #2 fechado por inteiro. (Mesma disciplina:
não afirmar mais cobertura que o modelo dá.)

### C. Deduplicação
Um crate referenciado por `use` **e** por caminho inline não pode virar duas arestas
nem duas violações. Dedup por aresta `(origem → alvo)` / "uma violação por import
proibido" (a regra já existente). Garantir que A/B não regridem isso.

## Fixtures bite-proof (V3 em direção proibida → só dispara após o conserto)

Positivas (mordem `[V3]` só depois do conserto):
- **expressão**: arquivo L2 usa `wiremod::FUNC()` (L4), sem `use`.
- **tipo**: L2 com `let x: wiremod::T` (L4), sem `use`.
- **atributo**: L2 com `#[arg(default_value_t = wiremod::N)]` (L4), sem `use`.

Negativas (guarda de robustez — **não** podem criar aresta/violação):
- caminho **local**: `crate::interno::Foo` inline → 0 violação.
- `std`/externo: `std::cmp::max(...)` inline → 0 violação.

Harness afirma o multiset de IDs. Re-rodar a **mutação** no escopo alterado
(extração em `rs_parser.rs`): 0 sobreviventes que mudam veredito; os ramos novos
mordidos pelas positivas, a guarda pelas negativas.

## Validação pelo oráculo (re-run)

- **`oraculo/biteproof_pathref`** (0059): a aresta `a→b` (só path-ref) passa a
  **resolver** — o cego #2 fecha no reprodutor.
- **Robustez**: re-run na **lente** tem de manter **0 só-linter** (o conserto não
  inventou aresta) e self-lint = 0. Se aparecer aresta nova na lente, triar — pode
  ser uma referência cross-crate real antes invisível (achado bom), não bug.
- Se houver outro workspace real à mão, rodar e triar.

## Critérios de Verificação

- [ ] Pré-condição (0059 presente; `biteproof_pathref` existe; corpus verde).
- [ ] Extração estendida a `scoped_identifier`/`scoped_type_identifier`/caminhos de
      tipo, resolvendo o 1º segmento por `classify_import`; local/`std` ficam fora.
- [ ] Atributo/macro `token_tree` varrido; residual (path em corpo de macro
      não-estruturado), se houver, **documentado como cego mais estreito**, não escondido.
- [ ] Dedup preservada (uso por `use` + path inline = uma aresta / uma violação).
- [ ] Fixtures positivas (expr/tipo/atributo) mordem `[V3]` só após o conserto;
      negativas (local/`std`) não criam violação.
- [ ] Mutação re-rodada: 0 sobreviventes que mudam veredito no escopo alterado.
- [ ] Oráculo: `biteproof_pathref` fecha; lente mantém **0 só-linter** e self-lint 0.
- [ ] Laudo ao fim; o estatuto do #2 dito com precisão (fechado para posições
      estruturadas + atributo; residual de macro, se houver, nomeado); nada mascarado.

## Fora de escopo (trilhas seguintes)

- **Corpus de projetos reais variados** — a trilha de **descoberta** dos cegos ainda
  desconhecidos (rodar o oráculo em N workspaces; ranquear o que aparecer). Ortogonal
  a este conserto.
- **Contador de `Layer::Unknown`**; **oráculo de posição/severidade**; **merge com o
  `master` público**.

## Disciplina

Completar a extração sem quebrar a robustez (0 só-linter é invariante); fixtures
positivas e **negativas** (a guarda contra falso-positivo); mutação re-rodada;
oráculo re-run para provar que a aresta fecha e nenhuma falsa nasce; residual nomeado,
não mascarado; laudo ao fim.
