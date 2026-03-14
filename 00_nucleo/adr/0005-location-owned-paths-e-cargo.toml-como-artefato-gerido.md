# ⚖️ ADR-0005: Location Owned Paths e Cargo.toml como Artefato Gerido

**Status**: `PROPOSTO`
**Data**: 2026-03-14

---

## Contexto

Dois problemas abertos deixados pelo ADR-0004 precisam de resolução
formal antes que possam ser atacados:

### Problema 1 — `Location<'a>` com paths owned

`source_error_to_violation` e `parse_error_to_violation` em L4
produzem `Violation<'static>` usando `Box::leak()` para converter
`PathBuf` owned em `&'static Path`. Isso é necessário porque
`Location<'a>` declara `path: &'a Path` — uma referência que
pressupõe que o path vive num buffer externo com lifetime conhecido.

Violações de infraestrutura (`V0`, `PARSE`) não têm esse buffer —
seus paths vêm de `PathBuf` owned gerado durante o parse de erro,
não do conteúdo do `SourceFile`. Logo, não podem ser `&'a str`
genuínos.

`Box::leak()` é inofensivo na prática — num utilitário CLI de
curta duração, vazar uma dúzia de bytes de paths é irrelevante,
e o SO limpa tudo no `exit()`. Mas é uma dívida arquitetural:
viola o invariante de zero-copy de forma invisível e poderia
causar problemas reais num contexto de longa duração (LSP server,
watch mode).

### Problema 2 — `Cargo.toml` sem nucleação

`Cargo.toml` foi modificado pelo ADR-0004 (adição de `rayon`)
sem um prompt correspondente em L0. O arquivo existe e é
rastreado pelo git, mas sua origem causal não está registrada.
Isso viola a Trava de Nucleação — V1 dispararia se o linter
pudesse analisar TOML.

---

## Decisão

### Para Problema 1 — `Cow<'a, Path>` em `Location`

Alterar `Location<'a>` para usar `Cow<'a, Path>` à semelhança
do que foi feito com `Token.symbol`:
```rust
use std::borrow::Cow;
use std::path::Path;

pub struct Location<'a> {
    pub path: Cow<'a, Path>,
    pub line: usize,
    pub column: usize,
}
```

- `Cow::Borrowed(&'a Path)` — violações normais, path referencia
  o buffer do `SourceFile`
- `Cow::Owned(PathBuf)` — violações de infraestrutura (V0, PARSE),
  path é owned, sem leak

`Box::leak()` é removido completamente de `main.rs`.

Impacto: `Location` propaga para `Violation<'a>` e todas as
funções de regras que constroem `Location`. Refatoração
mecânica — o compilador guia cada passo.

### Para Problema 2 — `Cargo.toml` como artefato gerido

Criar `00_nucleo/prompts/cargo.md` declarando `Cargo.toml`
como artefato gerido por prompt. O prompt declara todas as
dependências, suas versões e a justificativa arquitetural para
cada uma.

`Cargo.toml` pertence a L4 — é o ponto de composição de todas
as dependências externas do sistema. Qualquer mudança em
dependências requer revisão do prompt antes de modificar o
arquivo.

---

## Prompts Afetados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `Location<'a>` com `Cow<'a, Path>` |
| `linter-core.md` | Remoção de `Box::leak()`, atualização de conversores |
| `cargo.md` | Criação — nucleação de `Cargo.toml` |

---

## Estado Provisório Aceito

Até a implementação deste ADR, `Box::leak()` permanece em
`main.rs` com o seguinte comentário obrigatório de linhagem:
```rust
// ADR-0004 trade-off aceito: path de erro de infraestrutura
// vem de PathBuf owned, não do buffer do SourceFile.
// Box::leak() é inofensivo em CLI de curta duração —
// o SO limpa no exit(). Resolução em ADR-0005: Location
// com Cow<'a, Path>.
let leaked_path: &'static Path = Box::leak(path.into_boxed_path());
```

O comentário `TODO(ADR-0005)` não é suficiente — o comentário
deve explicar a decisão, não apenas apontar para trabalho futuro.

---

## Consequências

### ✅ Positivas

- `Location` passa a ser correta para todos os casos — sem
  exceções implícitas
- `main.rs` fica livre de `Box::leak()` e `'static` desnecessário
- `Cargo.toml` entra no ciclo de nucleação — mudanças de
  dependência passam por L0
- Watch mode e LSP server futuros não herdam dívida de memória

### ❌ Negativas

- `Cow<'a, Path>` propaga por todas as regras que constroem
  `Location` — refatoração pesada mas mecânica
- `cargo.md` requer escrita retroativa do histórico de
  dependências adicionadas antes da nucleação

### ⚙️ Neutras

- `Cow<'a, Path>` é idiomático Rust — mesmo padrão já
  estabelecido por `Token.symbol`
- O comportamento externo do linter não muda — apenas a
  representação interna de `Location`

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Manter `Box::leak()` permanentemente | Zero esforço | Viola zero-copy, problemático em watch mode / LSP |
| `Arc<Path>` em `Location` | Sem lifetime em `Location` | Overhead de contagem de referências desnecessário para CLI |
| `PathBuf` owned em `Location` | Simples | Quebra zero-copy para o caso comum — regressão |

---

## Referências

- ADR-0004: Reformulação do Motor de Análise
- `violation-types.md` — Errata Cow (`Token.symbol`)
- `linter-core.md` — Pipeline concorrente, nota sobre `Box::leak()`
```

---
