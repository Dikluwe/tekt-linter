# Prompt: C++ Parser (parsers/cpp)
Hash do Código: 8d7c3716

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-cpp`
**Criado em**: 2026-03-30
**Revisado em**: 2026-04-03
**Arquivos gerados**:
  - `03_infra/cpp_parser.rs`

---

## Contexto

Parser de C++ para o `crystalline-lint`. Analisa arquivos `.cpp`, `.cc`,
`.cxx`, `.hpp` e `.hxx` usando a gramática `tree-sitter-cpp`. Detecta
`#include` como imports, extrai a interface pública (funções, structs,
classes), identifica chamadas de teste via macros (`TEST`, `TEST_F`, `TEST_P`,
`EXPECT_*`, `ASSERT_*`) e produz `ParsedFile<'a>` para as regras V1–V14.

Arquivos de header (`.h`, `.hpp`, `.hxx`) são tratados como
_declaration-only_ — isentos de V2.

Dependências injetadas no construtor: `PromptReader`, `PromptSnapshotReader`,
`CrystallineConfig`, `project_root: PathBuf`.

Resolução de camadas: física via `normalize()` + `resolve_file_layer()`
(ADR-0009, I2). Imports não-relativos (`<vector>`, `<string>`, etc.) são
tratados como `Layer::Unknown` (header de sistema ou biblioteca externa).

---

## Header Cristalino

Arquivos C++ usam comentários de linha `//` contíguos no topo do arquivo:

```cpp
// Crystalline Lineage
// @prompt 00_nucleo/prompts/parsers/cpp.md
// @prompt-hash <sha256[0..8]>
// @layer L3
// @updated YYYY-MM-DD
```

O bloco termina na primeira linha que não começa com `//`.

---

## Resolução de Camadas — LayerResolver

Resolução física (I2):
1. Se o import não começa com `./` ou `../` → `Layer::Unknown`.
2. `base = file.path.parent()` + `import_path`.
3. `normalize(base.join(import_path), project_root)` → remove `..`, verifica fuga.
4. `resolve_file_layer(&normalized, project_root, config)`.

---

## Extracção de Imports (V3, V9, V10)

| Nó AST              | ImportKind | Notas                               |
|---------------------|------------|-------------------------------------|
| `preproc_include`   | `Direct`   | `string_literal` ou `system_lib_string` |

`target_subdir` é sempre `None` para C++ (sem equivalente a subdirs de L1).

---

## Extracção de Tokens — Símbolos Proibidos (V4)

V4 usa `file.language()` para seleccionar a lista de símbolos proibidos — não usa `ImportKind`.

Tokens extraídos: `call_expression` → nome da função/método chamado.

---

## Test Coverage (V2)

- `has_test_ast`: macros `TEST`, `TEST_F`, `TEST_P`, ou chamada começando com `EXPECT_` / `ASSERT_` / `assert_`.
- `has_adjacent_test`: arquivo sibling `<stem>_test.cpp`, `test_<stem>.cpp`, `<stem>_test.cc`, `test_<stem>.cc`.
- `is_decl_only`: extensão `.h`, `.hpp`, ou `.hxx`.
- `has_test_coverage = has_test_ast || has_adjacent_test || is_decl_only`.

---

## Interface Pública (V6)

| Nó AST                      | Campo        | Notas                                              |
|-----------------------------|--------------|---------------------------------------------------|
| `function_definition`       | `functions`  | Excluídas funções `static` em arquivos de impl    |
| `declaration` (struct/enum/class) | `types` | `TypeKind::Struct`, `TypeKind::Enum`, ou `TypeKind::Class` |
| `class_specifier`           | `types`      | `TypeKind::Class`                                 |

`reexports` é sempre `[]`.

---

## Traits Declaradas (V11) — `declared_traits`

Não aplicável. C++ tem herança e interfaces virtuais, mas o linter não as
modela como `declared_traits`. `declared_traits` é sempre `[]` para C++.

---

## Traits Implementadas (V11) — `implemented_traits`

Não aplicável. `implemented_traits` é sempre `[]` para C++.

---

## Declarações de Tipo (V12) — `declarations`

| Nó AST           | `DeclarationKind` |
|------------------|-------------------|
| struct em `declaration` | `Struct` |
| enum em `declaration`   | `Enum`   |
| class em `declaration` ou `class_specifier` | `Class` |

Implementações de métodos (`function_definition` com body) não são capturadas em `declarations`.

---

## Assinatura do Construtor

```rust
pub struct CppParser<R: PromptReader, S: PromptSnapshotReader> {
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R, S> CppParser<R, S> { pub fn new(...) -> Self }
impl<R, S> LanguageParser for CppParser<R, S> { fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> }
```

---

## Restrições

- `parse()` recebe `&'a SourceFile` e retorna `ParsedFile<'a>`. Zero-Copy (I1).
- Proibido `.to_string()` para conteúdo do buffer.
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para `Layer::Unknown`.
- `std::fs::canonicalize` proibido.
- `ImportKind` nunca contém variantes específicas de C++.
- V4 usa `file.language()`, não `ImportKind`.
- `std::io::Error` nunca atravessa para L1.

---

## Critérios de Verificação

- Header completo `// Crystalline Lineage` → todos os campos populados.
- `#include "./foo.cpp"` (relativo) → camada correta.
- `#include <vector>` (sistema STL) → `Layer::Unknown`.
- `#include "../../../escape.hpp"` (escapa raiz) → `Layer::Unknown`.
- `TEST(Suite, Case)` ou `EXPECT_EQ(...)` → `has_test_coverage = true`.
- Arquivo `.hpp` → `has_test_coverage = true` (declaration-only).
- Source vazio → `Err(ParseError::EmptySource)`.
- Source inválido → `Err(ParseError::SyntaxError)`.
- Linguagem errada → `Err(ParseError::UnsupportedLanguage)`.

---

## Histórico de Revisões

| Data       | Motivo                                       | Arquivos afetados              |
|------------|----------------------------------------------|--------------------------------|
| 2026-03-30 | Criação inicial — suporte a C++ no linter    | `03_infra/cpp_parser.rs`       |
| 2026-04-03 | Criação deste prompt (faltava em L0)         | `00_nucleo/prompts/parsers/cpp.md` |
