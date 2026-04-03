# Prompt: C Parser (parsers/c)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-c`
**Criado em**: 2026-03-30
**Revisado em**: 2026-04-03
**Arquivos gerados**:
  - `03_infra/c_parser.rs`

---

## Contexto

Parser de C para o `crystalline-lint`. Analisa arquivos `.c` e `.h` usando
a gramática `tree-sitter-c`. Detecta `#include` como imports, extrai a
interface pública (funções e structs), identifica chamadas de teste via
macros (`TEST`, `RUN_TEST`, `assert_*`) e produz `ParsedFile<'a>` para
as regras V1–V14.

Arquivos `.h` são tratados como _declaration-only_ — isentos de V2.

Dependências injetadas no construtor: `PromptReader`, `PromptSnapshotReader`,
`CrystallineConfig`, `project_root: PathBuf`.

Resolução de camadas: física via `normalize()` + `resolve_file_layer()`
(ADR-0009, I2). Imports com `<stdio.h>` ou outros não-relativos são tratados
como `Layer::Unknown` (pacote externo / header de sistema).

---

## Header Cristalino

Arquivos C usam comentários de linha `//` contíguos no topo do arquivo:

```c
// Crystalline Lineage
// @prompt 00_nucleo/prompts/parsers/c.md
// @prompt-hash <sha256[0..8]>
// @layer L3
// @updated YYYY-MM-DD
```

O bloco termina na primeira linha que não começa com `//`.

---

## Resolução de Camadas — LayerResolver

Resolução física (I2):
1. Se o import não começa com `./` ou `../` → `Layer::Unknown` (header de sistema).
2. `base = file.path.parent()` + `import_path`.
3. `normalize(base.join(import_path), project_root)` → remove `..`, verifica fuga.
4. `resolve_file_layer(&normalized, project_root, config)`.

---

## Extracção de Imports (V3, V9, V10)

| Nó AST              | ImportKind | Notas                              |
|---------------------|------------|------------------------------------|
| `preproc_include`   | `Direct`   | `string_literal` ou `system_lib_string` |

`target_subdir` é sempre `None` para C (sem equivalente a subdirs de L1 em C puro).

---

## Extracção de Tokens — Símbolos Proibidos (V4)

V4 usa `file.language()` para seleccionar a lista de símbolos proibidos — não usa `ImportKind`.

Tokens extraídos: `call_expression` → nome da função chamada.

---

## Test Coverage (V2)

- `has_test_ast`: macros `TEST`, `RUN_TEST`, ou chamada começando com `assert_`.
- `has_adjacent_test`: arquivo sibling `<stem>_test.c` ou `test_<stem>.c`.
- `is_decl_only`: extensão `.h`.
- `has_test_coverage = has_test_ast || has_adjacent_test || is_decl_only`.

---

## Interface Pública (V6)

| Nó AST              | Campo        | Notas                                    |
|---------------------|--------------|------------------------------------------|
| `function_definition` | `functions` | Excluídas funções `static` em `.c`      |
| `declaration` (struct/enum) | `types` | `TypeKind::Struct` ou `TypeKind::Enum` |

`reexports` é sempre `[]` (C não tem reexports).

---

## Traits Declaradas (V11) — `declared_traits`

Não aplicável. C não tem interfaces formais. `declared_traits` é sempre `[]`.

---

## Traits Implementadas (V11) — `implemented_traits`

Não aplicável. `implemented_traits` é sempre `[]`.

---

## Declarações de Tipo (V12) — `declarations`

| Nó AST        | `DeclarationKind` |
|---------------|-------------------|
| struct em `declaration` | `Struct` |
| enum em `declaration`   | `Enum`   |

`impl`-com-trait não existe em C — não há risco de captura incorreta.

---

## Assinatura do Construtor

```rust
pub struct CParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R, S> CParser<R, S> { pub fn new(...) -> Self }
impl<R, S> LanguageParser for CParser<R, S> { fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> }
```

---

## Restrições

- `parse()` recebe `&'a SourceFile` e retorna `ParsedFile<'a>`. Zero-Copy (I1).
- Proibido `.to_string()` para conteúdo do buffer.
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para `Layer::Unknown`.
- `std::fs::canonicalize` proibido.
- `ImportKind` nunca contém variantes específicas de C.
- V4 usa `file.language()`, não `ImportKind`.
- `std::io::Error` nunca atravessa para L1.

---

## Critérios de Verificação

- Header completo `// Crystalline Lineage` → todos os campos populados.
- `#include "./foo.c"` (relativo) → camada correta.
- `#include <stdio.h>` (sistema) → `Layer::Unknown`.
- `#include "../../../escape.h"` (escapa raiz) → `Layer::Unknown`.
- `TEST(...)` ou `assert_eq(...)` → `has_test_coverage = true`.
- Arquivo `.h` → `has_test_coverage = true` (declaration-only).
- Source vazio → `Err(ParseError::EmptySource)`.
- Source inválido → `Err(ParseError::SyntaxError)`.
- Linguagem errada → `Err(ParseError::UnsupportedLanguage)`.

---

## Histórico de Revisões

| Data       | Motivo                                      | Arquivos afetados         |
|------------|---------------------------------------------|---------------------------|
| 2026-03-30 | Criação inicial — suporte a C no linter     | `03_infra/c_parser.rs`    |
| 2026-04-03 | Criação deste prompt (faltava em L0)        | `00_nucleo/prompts/parsers/c.md` |
