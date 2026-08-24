# Changelog

Todas as mudanças notáveis deste projeto são registadas aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e o projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Unreleased]

### Added

- **Subcomando `refine` (ADR-0019)** — comparação direcional de snapshots explícitos
  por relações `preserve`, `may-normalize` e `must-not-invent`, com resultados
  `Preserved`, `Violated(Witness)` e `Unknown(reason)`, saída texto/SARIF e exit codes
  distintos. Primeira etapa sem Git, execução de comandos, SMT ou análise
  interprocedural.
- **Subcomando `snapshot` (ADR-0019, Etapa B1)** — extração Rust determinística por
  queries tree-sitter declaradas no contrato, cardinalidade e ausência explícitas,
  confinamento de paths/symlinks à raiz e escrita atômica do snapshot v1. Inclui
  autoaplicação do linter e três oráculos históricos locais.

- **V23 `ContextErasure`, V24 `SemanticFieldLoss` e V25 `DecisionOwnership`** —
  contratos declarativos de preservação semântica (ADR-0018), com fatos AST genéricos,
  seleção isolada, níveis configuráveis e catálogo SARIF. Sem contratos em
  `crystalline.toml`, as regras permanecem silenciosas. Escopo inicial Rust e fluxo
  intraprocedural limitado.

- **V15 `MultiPromptHeader`** — arquivo `.rs` em L1–L4 com 2+ linhas
  `//! @prompt` no doc-header passa a ser **erro bloqueante de lint**.
  A regra de linhagem é um ficheiro, um prompt; com multi-`@prompt` o
  `--fix-hashes` comportava-se de forma indefinida (hash certo no header
  errado). O lint agora bloqueia com mensagem que lista os prompts
  encontrados, em vez de silêncio ou correcção ambígua. `ParsedFile`
  ganha o campo `prompt_refs` (populado apenas pelo RustParser) e o
  default de `--checks` passa a incluir `v15`.

## [0.2.0] — 2026-06-09

Primeira versão versionada e taggeada. Sobre a base `0.1.0` (não taggeada —
parsing multi-linguagem para Rust, TypeScript, Python, C, C++ e Zig, mais o
travamento bidirecional de hashes de linhagem), o `0.2.0` traz uma análise de
dependências cross-crate muito mais fiel para Rust e a primeira opção de política.

### Added

- **Classificação de import ciente de dependências.** O linter passa a ler o
  workspace Cargo (membros, camadas, dependências declaradas e renomeações) para
  distinguir um crate first-party de um externo. Com isso, a **gravidade
  cross-crate** (um crate de uma camada importando outro de camada proibida) passa
  a ser **vista** — antes, todo cross-crate caía em "externo" e escapava.
- **Detecção de referência cross-crate em três formas antes invisíveis:**
  - por `use` com **alias** (`use outro_crate as x;`);
  - por **dependência renomeada** no `Cargo.toml` (`alias = { package = "real" }`);
  - por **caminho qualificado fora do `use`** — em expressão (`crate_x::FUNC()`),
    tipo (`-> crate_x::T`) e atributo (`#[arg(... crate_x::N)]`).
- **Opção `check_test_imports`** (em `crystalline.toml`, default `false`).

### Changed

- **V3 / V9 / V14 passam a excluir código `#[cfg(test)]` por padrão.** A gravidade
  é uma afirmação sobre o **grafo de produção**, e `#[cfg(test)]` é removido do
  build de release — uma aresta que cruza camadas só em teste não corrompe o que o
  artefato entrega. Para reactivar a verificação em código de teste (teste como
  canário), ligue `check_test_imports = true`.

### Qualidade interna (sem impacto de API)

- Corpo de **fixtures bite-proof** por regra — cada teste afirma o conjunto exato
  de IDs de violação, não apenas sucesso/fracasso.
- **Completude por mutação** para os **vereditos de lint de Rust** em todo o
  caminho do veredito (regras, classificação de import, registro de crates,
  configuração, walking, I/O de prompt, despacho e saída SARIF): zero mutantes
  sobreviventes que mudam um veredito.
- **Oráculo diferencial** contra a lente independente (`tekt-cargo-dsm`, grafo do
  compilador): nas arquiteturas testadas, as duas computações concordam aresta a
  aresta no modo default.

### Limitações conhecidas

- **Precisão de sub-caminho** para referências dentro de atributo/macro
  (`token_tree`): captura-se o crate (1º segmento), não o caminho completo — então
  o subdir do V9 a partir de um atributo não é resolvido. V3/V14 não são afetados.
- **Caminhos dentro de corpos de macro** que a grammar não estrutura podem
  permanecer invisíveis.
- **Posição e severidade** das violações não estão sob o oráculo de veredito.
- No modo `check_test_imports = true`, o grafo de **teste** não tem segunda
  computação independente (a lente exclui teste por construção) — esse modo é
  coberto só por fixture, não pelo diferencial.

### Declaração de escopo

O `0.2.0` é **completo para os vereditos de lint de Rust** (selado por mutação) e
**concorda com o oráculo independente da lente nas arquiteturas testadas**. Ele
**não** afirma completude contra todas as formas que a linguagem Rust permite: a
lista de pontos cegos fechados saiu da análise de arquiteturas reais, e a trilha de
descoberta sistemática (corpus de projetos variados) continua aberta.

[0.2.0]: https://github.com/Dikluwe/tekt-linter/releases/tag/v0.2.0
