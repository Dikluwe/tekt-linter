# Prompt: Python Parser (parsers/python)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-python`
**Criado em**: 2026-03-19
**Arquivos gerados**:
  - 03_infra/py_parser.rs + test

---

## Contexto

O núcleo L1 aguarda um `ParsedFile<'a>` completo e agnóstico.
Esta camada L3 faz o trabalho impuro para Python: recebe
referência de `SourceFile`, aciona `tree-sitter-python`, e
traduz a AST nos campos que as regras V1–V12 consomem.

Implementa a trait `LanguageParser` declarada em
`01_core/contracts/language_parser.rs` — a mesma trait do
`RustParser` e do `TsParser`. As regras de L1 não sabem que
linguagem estão a analisar.

Recebe quatro dependências injetadas via construtor:
- `PromptReader` — para V1 e V5
- `PromptSnapshotReader` — para V6
- `CrystallineConfig` — para resolução de camadas, subdirs,
  aliases Python e configuração de wiring
- `project_root: PathBuf` — raiz do projecto para resolução física

**Diretiva Zero-Copy (ADR-0004):** `parse()` recebe `&'a SourceFile`
e retorna `ParsedFile<'a>` com referências ao buffer do fonte.

**Resolução física (ADR-0009):** A camada de um import é determinada
pelo caminho físico no disco após normalização algébrica, não pelo
texto do import. Imports relativos (começam com `.`) são resolvidos
fisicamente; imports sem ponto inicial são packages externos →
`Layer::Unknown` directamente ou aliases se configurados em
`[py_aliases]`.

---

## 1. Header cristalino

Ficheiros `.py` usam comentários de linha `#` em bloco contíguo
no topo:

```python
# Crystalline Lineage
# @prompt 00_nucleo/prompts/<nome>.md
# @prompt-hash <sha256[0..8]>
# @layer L<n>
# @updated YYYY-MM-DD
```

O bloco termina na primeira linha que não começa com `#` —
mesma semântica do `//!` em Rust e `//` em TypeScript.

**Justificativa:** Python tem docstrings (`"""..."""`) e comentários
`#`. Docstrings são associadas à entidade que documentam (módulo,
função, classe) e ferramentas como Sphinx e pydoc as processam.
`#` é inerte para todas as ferramentas do ecossistema Python e
semanticamente equivalente ao `//` de TypeScript para este propósito.

**Extracção:** varrer `file.content` linha a linha enquanto a linha
começa com `#`. Parar na primeira que não começa. Field matching
sobre `@prompt`, `@prompt-hash`, `@layer`, `@updated` —
fatias `&'a str` do buffer (I1 Zero-Copy).

---

## 2. Resolução de camadas — PyLayerResolver

A resolução é física, não léxica (invariante I2 do `_template.md`).

### Passo 1 — Detecção de package externo

Se o import não começa com `.` E não começa com uma chave de
`[py_aliases]` no `crystalline.toml`, é um package Python externo
→ `Layer::Unknown` directamente, sem álgebra de paths:

```
"os"                →  Layer::Unknown  (stdlib externo)
"typing"            →  Layer::Unknown  (stdlib externo)
"pathlib"           →  Layer::Unknown  (stdlib externo)
"requests"          →  Layer::Unknown  (package pip)
"."                 →  continua para passo 2 (relativo)
".utils"            →  continua para passo 2 (relativo)
"..core"            →  continua para passo 2 (relativo)
"core.contracts"    →  continua para passo 2 se "core" é alias
```

**`import_statement` (`import X`):** o módulo `X` nunca começa
com `.` — é sempre externo, ou alias se `X` começa com chave de
`[py_aliases]`. Mas `import os`, `import pathlib`, `import requests`
são sempre externos. Apenas aliases explícitos como `import core`
(com `core = "01_core"` em `[py_aliases]`) passam para passo 2.

**`import_from_statement` (`from X import Y`):**
- `from .utils import Y` → `.utils` começa com `.` → relativo
- `from os import path` → `os` não começa com `.` e sem alias → externo

### Passo 2 — Resolução de alias

Se o módulo começa com uma chave de `[py_aliases]`, substituir
pelo valor correspondente:

```toml
# crystalline.toml
[py_aliases]
"core"  = "01_core"
"shell" = "02_shell"
"infra" = "03_infra"
```

```
"core.contracts"    →  "01_core/contracts"    (alias + dotted→slash)
"infra.walker"      →  "03_infra/walker"
```

Imports relativos (com `.`) passam directamente ao passo 3 sem
substituição de alias — não têm prefixo de alias.

### Passo 3 — Álgebra de paths com verificação de fuga

Para imports relativos, calcular o nível a partir do número de
pontos iniciais (`import_prefix` no nó `relative_import`):

```
"."     → level=1 → base = file.parent()
".."    → level=2 → base = file.parent()/.."  → um nível acima
".utils"  → level=1, module="utils" → base = file.parent() + "utils"
"..core"  → level=2, module="core"  → base = file.parent() + "../core"
```

Algoritmo:
```rust
let n_dots: usize = prefix_text.len(); // "." = 1, ".." = 2
let ups = "../".repeat(n_dots.saturating_sub(1));
let module_part = dotted_name.replace('.', "/"); // "utils.sub" → "utils/sub"
let rel = if module_part.is_empty() {
    // from . import X → "." (current dir)
    if n_dots == 1 { ".".to_string() } else { ups }
} else {
    format!("{}{}", ups, module_part)
};
```

```rust
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if components.is_empty() { return None; } // fuga detectada
                components.pop();
            }
            Component::CurDir => {}
            c => { components.push(c); }
        }
    }
    let result: PathBuf = components.iter().collect();
    if project_root != Path::new(".") && !project_root.as_os_str().is_empty() {
        if !result.starts_with(project_root) { return None; }
    }
    Some(result)
}
```

`std::fs::canonicalize` é proibido — normalização algébrica apenas.

### Passo 4 — Reutilização de `resolve_file_layer`

```rust
let base = file.path.parent().unwrap_or(Path::new("."));
let joined = base.join(&rel_path_str);
let target_layer = match normalize(&joined, &project_root) {
    Some(normalized) => resolve_file_layer(&normalized, &project_root, &config),
    None             => Layer::Unknown,
};
```

A mesma função do `FileWalker` é a fonte de verdade.

### `target_subdir` para V9

Extraído do caminho normalizado via `strip_prefix` contra o valor
da camada em `[layers]`:

```rust
fn resolve_py_subdir(normalized: &Path, target_layer: &Layer, project_root: &Path,
                     config: &CrystallineConfig) -> Option<&'static str> {
    if *target_layer != Layer::L1 { return None; }
    let layer_dir = config.layers.get("L1")?;
    let base_l1 = project_root.join(layer_dir);
    let relative = normalized.strip_prefix(&base_l1)
        .or_else(|_| normalized.strip_prefix(layer_dir.as_str())).ok()?;
    let subdir = relative.components().next()
        .and_then(|c| c.as_os_str().to_str())?;
    Some(Box::leak(subdir.to_string().into_boxed_str()))
}
```

`target_subdir` é `Some(subdir)` para imports de L1.
V9 decide se o subdir é porta válida.

---

## 3. Extracção de imports (V3, V9, V10)

Nós AST relevantes: `import_statement`, `import_from_statement`.

| Campo | Como extrair |
|-------|--------------|
| `path` | Texto do módulo (após processamento de dots): `&'a str` do buffer |
| `line` | `node.start_position().row + 1` |
| `kind` | Ver tabela de mapeamento abaixo — `Direct/Glob/Alias/Named` |
| `target_layer` | PyLayerResolver — 4 passos descritos acima |
| `target_subdir` | `resolve_py_subdir` após resolução física — apenas para L1 |

**Mapeamento `Nó AST → ImportKind` (semântico, nunca sintáctico):**

| Forma de import Python | `ImportKind` | Exemplo |
|------------------------|-------------|---------|
| `import os` | `Direct` | módulo único |
| `import os, sys` | `Direct` | múltiplos (um `Import` por módulo) |
| `import numpy as np` | `Alias` | import com renomeação |
| `from os import path` | `Named` | símbolo nomeado |
| `from os import path, getcwd` | `Named` | múltiplos nomeados |
| `from os import *` | `Glob` | import de todos |
| `from . import utils` | `Named` | relativo nomeado |
| `from .contracts import FileProvider` | `Named` | relativo com símbolo |
| `from .. import core` | `Named` | relativo dois níveis |

Nunca usar `ImportKind::PyImport` ou qualquer outra variante
específica de linguagem — apenas `Direct/Glob/Alias/Named`.

**`import_statement`** (`import os`):
- Nó filho `dotted_name` ou `aliased_import` → extrair módulo
- Se não é alias → `Layer::Unknown` directamente
- Se é alias → resolver via passo 2–4

**`import_from_statement`** (`from X import Y`):
- Nó filho `module_name`:
  - Se `relative_import` → relativo → processar via passo 3–4
  - Se `dotted_name` → absoluto; verificar aliases; ou `Layer::Unknown`
- Nós filhos após `import` → nomes importados (para `import_name_map` interno)

**`import_name_map` (mapa interno, não exposto a L1):**
Construído durante a extracção de imports para resolver bases de
classes em `implemented_traits` e `declarations`.
Mapeia nome local importado → (target_layer, target_subdir).

Para `from .contracts import FileProvider`:
→ `import_name_map["FileProvider"] = (L1, Some("contracts"))`

Para `from core.contracts import FileProvider, LanguageParser`:
(com alias `core = "01_core"`)
→ `import_name_map["FileProvider"] = (L1, Some("contracts"))`
→ `import_name_map["LanguageParser"] = (L1, Some("contracts"))`

Para `aliased_import` (`import Foo as Bar`):
→ mapear nome local `Bar`, não o original `Foo`

---

## 4. Extracção de tokens — símbolos proibidos (V4)

V4 usa `file.language()` para seleccionar a lista de símbolos
proibidos — não usa `ImportKind`. A lista Python vive em
`impure_core.rs` via `forbidden_symbols_for(Language::Python)`.
Este prompt documenta apenas como os tokens são extraídos do AST.

V4 proíbe I/O em L1. Em Python, I/O ocorre via imports de módulos
stdlib ou via chamadas directas a builtins (`open`) e funções
não-determinísticas.

**Módulos proibidos em L1:**
```
os, os.path, pathlib, shutil, subprocess, socket,
urllib, http.client, ftplib, smtplib
```

Detectados como `import_statement` ou `import_from_statement`
de nível superior cujo módulo raiz (antes de `.`) está na lista.

**Chamadas proibidas:**
```
open           (builtin — nó call com identifier "open")
random.random  (nó call com attribute "random.random")
time.time      (nó call com attribute "time.time")
datetime.now   (nó call com attribute "datetime.now" ou "datetime.datetime.now")
```

Detectadas como nós `call` cujo nó função (`attribute` ou `identifier`)
tem texto que corresponde a um símbolo proibido.

**Sem Motor de Duas Fases:** Python não tem o sistema de aliases
de importação de Rust. V4 opera directamente sobre os nós `call`
do AST e os `import_statement`/`import_from_statement` proibidos.

---

## 5. Test coverage (V2)

`has_test_coverage = true` se qualquer das condições:

**1. Construto de teste no AST:**
Nós `call` com função `identifier` ou `attribute` cujo nome é
`unittest`, `pytest`, `describe`, `it`, `test` ou `suite`.
Detecta unittest, pytest, mamba e equivalentes.

Também: `class_definition` cujo nome termina em `Test` ou
`Tests` e herda de `TestCase` (unittest).

**2. Ficheiro de teste adjacente:**
`source_file.has_adjacent_test` — `true` se existe
`<stem>_test.py` ou `test_<stem>.py` no mesmo directório
(verificado pelo walker antes de chamar `parse()`).

**3. Declaration-only (isento de V2):**
Ficheiro que contém apenas `class_definition` com base
`Protocol`/`ABC`/`ABCMeta`, `import_statement`/`import_from_statement`,
e `assignment` de `__all__`. Nenhuma `function_definition` com
corpo não-trivial (não é `...` ou `pass`).

---

## 6. Interface pública (V6)

Construtos de nível superior **sem** prefixo `_`:

| Nó | `FunctionSignature` / `TypeSignature` | `TypeKind` |
|----|--------------------------------------|------------|
| `function_definition` sem `_` | `FunctionSignature` | — |
| `decorated_definition` → `function_definition` | `FunctionSignature` | — |
| `class_definition` sem `_`, base Protocol/ABC/ABCMeta | `TypeSignature` | `Interface` |
| `class_definition` sem `_`, sem base especial | `TypeSignature` | `Class` |
| `assignment` com alvo `__all__` | `reexports` | — |

**`FunctionSignature`:**
- `name`: identificador após `def` — `&'a str` do buffer
- `params`: tipos dos parâmetros normalizados (whitespace colapsado),
  omitindo `self`/`cls`; tipo anotado se presente (`x: int` → `"int"`)
- `return_type`: tipo de retorno se anotado (`-> bool:`), `None` se omitido

**`TypeSignature`:**
- `name`: identificador da classe — `&'a str` do buffer
- `kind`: `Interface` se herda de `Protocol`/`ABC`/`ABCMeta`; `Class` c.c.
- `members`: nomes de métodos públicos (não `_`) definidos na classe

**`reexports`:** valor de `__all__` como texto do buffer —
`['foo', 'bar']` capturado como string literal do assignment.

**`prompt_snapshot`:** via `PromptSnapshotReader::read_snapshot` —
idêntico para todas as linguagens.

---

## 7. Interfaces declaradas (V11) — `declared_traits`

Apenas quando `file.layer == Layer::L1` e path contém `"contracts"`.

Para cada `class_definition` de nível superior cuja lista de bases
contém `Protocol`, `ABC` ou `ABCMeta`:
- Extrair `name` como `&'a str` do buffer
- Adicionar a `declared_traits`

```python
# 01_core/contracts/file_provider.py
class FileProvider(Protocol):     →  declared_traits = ["FileProvider"]
class _InternalBase(Protocol):    →  ignorado (prefixo _)
class Helper:                     →  ignorado (não é Protocol/ABC)
```

Ficheiros em L1 fora de `contracts/` não contribuem.

---

## 8. Interfaces implementadas (V11) — `implemented_traits`

Apenas quando `file.layer == Layer::L2 | Layer::L3`.

Para cada `class_definition` de nível superior com bases:
1. Para cada nome base, verificar no `import_name_map` interno
2. Se `import_name_map[base_name] == (L1, Some("contracts"))`:
   → adicionar `base_name` a `implemented_traits`

```python
# 03_infra/walker.py
# from .contracts import FileProvider  (resolves to L1/contracts via py_aliases)
class FileWalker(FileProvider):    →  implemented_traits = ["FileProvider"]
class InternalHelper:              →  ignorado (sem base de contracts/)
```

A resolução física do import (via `import_name_map`) garante que
apenas bases importadas de L1/contracts/ são capturadas.

---

## 9. Declarações de tipo (V12) — `declarations`

Para todos os arquivos, sem filtro de layer. V12 filtra por
`layer == L4` internamente.

| Nó | `DeclarationKind` | Condição |
|----|------------------|----------|
| `class_definition` | `Class` | sem base Protocol/ABC/ABCMeta E sem base em L1/contracts/ |
| `class_definition` com base Protocol/ABC | **não capturado** | é contrato — permitido em L4 |
| `class_definition` com base de contracts/ | **não capturado** | é adapter — equivalente a `impl Trait for Type` |

**Nota:** `DeclarationKind::Interface` e `DeclarationKind::TypeAlias`
não são emitidos pelo PyParser — Python não tem `interface` como
construto distinto (usa `Protocol`/`ABC`) e não tem `type X = Y`
fora de anotações. Apenas `DeclarationKind::Class` é emitido.

---

## 10. Assinatura do construtor

```rust
pub struct PyParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    pub prompt_reader: R,
    pub snapshot_reader: S,
    pub config: CrystallineConfig,
    pub project_root: PathBuf,
}

impl<R: PromptReader, S: PromptSnapshotReader> PyParser<R, S> {
    pub fn new(
        prompt_reader: R,
        snapshot_reader: S,
        config: CrystallineConfig,
        project_root: PathBuf,
    ) -> Self {
        Self { prompt_reader, snapshot_reader, config, project_root }
    }
}

impl<R, S> LanguageParser for PyParser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        // Ordem de extracção:
        // 1. header (bloco # no topo: @prompt, @prompt-hash, @layer, @updated)
        // 2. imports + import_name_map (PyLayerResolver 4 passos + resolve_py_subdir)
        // 3. tokens (imports proibidos + call nodes — sem Motor de Duas Fases)
        // 4. has_test_coverage (call pytest/unittest + adjacência + declaration-only)
        // 5. public_interface + prompt_snapshot (V6)
        // 6. declared_traits (apenas L1/contracts, apenas class com Protocol/ABC) (V11)
        // 7. implemented_traits (apenas L2|L3, import_name_map → base de contracts/) (V11)
        // 8. declarations — class sem Protocol/ABC/contracts (V12)
        //
        // Retorna ParsedFile<'a> com todas as referências apontando
        // para file.content
    }
}
```

---

## 11. Restrições

- `parse()` recebe `&'a SourceFile` — proibido consumir ownership
- Proibido `.to_string()` para strings do buffer — apenas
  `PromptHeader.current_hash` é `String` (calculado do disco)
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para
  `Layer::Unknown`, nunca silenciado, nunca panic
- `std::fs::canonicalize` proibido — normalização algébrica apenas
- `import_statement` sem prefixo de alias → `Layer::Unknown` directamente
- `import_from_statement` sem `.` inicial e sem alias → `Layer::Unknown`
- `class com base Protocol/ABC/ABCMeta` não é capturado em `declarations`
- `class com base de L1/contracts/` não é capturado em `declarations`
- `declared_traits` apenas em L1/contracts/ — filtragem no parser
- `implemented_traits` apenas em L2|L3, apenas bases de L1/contracts/
- `declarations` para todos os arquivos — V12 filtra por layer
- `PromptReader` e `PromptSnapshotReader` são injetados —
  `PyParser` nunca os instancia directamente
- `std::io::Error` nunca atravessa para L1 — convertido em
  `ParseError` antes de retornar
- `import_name_map` é mapa interno de L3 — não exposto a L1
- **`ImportKind` nunca contém variantes específicas de linguagem**:
  imports Python mapeiam para `Direct/Glob/Alias/Named` —
  nunca para `PyImport` ou outra variante Python
- **V4 usa `file.language()`, não `ImportKind`**, para seleccionar
  a lista de símbolos proibidos em `forbidden_symbols_for()`

---

## 12. Critérios de Verificação

```
Dado SourceFile .py com header cristalino completo em comentários #
Quando parse() for chamado
Então prompt_header populado com todos os campos como &'a str

Dado SourceFile com from .entities.layer import Layer
E file em 03_infra/walker.py, project_root="."
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L3, .. }
— relativo: 03_infra + entities/layer = 03_infra/entities/layer → L3

Dado py_aliases com "core" = "01_core"
E SourceFile com from core.contracts import FileProvider
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1,
      target_subdir: Some("contracts"), kind: ImportKind::Named, .. }
— alias resolvido antes da álgebra de paths

Dado SourceFile com from ..entities.layer import Layer
E file em 01_core/contracts/fp.py
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::L1,
      target_subdir: Some("entities"), .. }
— dois pontos = go up 1 from 01_core/contracts → 01_core + entities/layer

Dado SourceFile com from ../../../../../etc import passwd
(path que escapa da raiz do projecto)
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
— normalize() retorna None, Layer::Unknown propaga

Dado SourceFile com import os
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Direct, target_layer: Layer::Unknown, .. }

Dado SourceFile com import numpy as np
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Alias, .. }

Dado SourceFile com from os import path
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Named, .. }

Dado SourceFile com from os import *
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Glob, .. }

Dado SourceFile com from . import utils
Quando parse() for chamado
Então imports contém Import { kind: ImportKind::Named, .. }

Dado SourceFile com import os
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
— package externo, Layer::Unknown directamente

Dado SourceFile com from typing import Protocol
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Unknown, .. }
— "typing" não começa com . e não é alias

Dado SourceFile com from .lab.experiment import X
E lab/ mapeado em [layers]
Quando parse() for chamado
Então imports contém Import { target_layer: Layer::Lab, .. }
— import relativo de lab detectado, V10 dispara em produção

Dado SourceFile em L1 com import os
Quando parse() for chamado
Então tokens contém Token { symbol: "os", .. }
— módulo proibido em V4

Dado SourceFile em L1 com import pathlib
Quando parse() for chamado
Então tokens contém Token { symbol: "pathlib", .. }
— módulo proibido em V4

Dado SourceFile em L1 com open("file.txt")
Quando parse() for chamado
Então tokens contém Token { symbol: "open", .. }
— builtin proibido em V4

Dado SourceFile em L1 com random.random()
Quando parse() for chamado
Então tokens contém Token { symbol: "random.random", .. }
— chamada não-determinística proibida em V4

Dado SourceFile em L1 com time.time()
Quando parse() for chamado
Então tokens contém Token { symbol: "time.time", .. }

Dado SourceFile com import unittest; class FooTest(unittest.TestCase)
Quando parse() for chamado
Então has_test_coverage = true

Dado SourceFile com source_file.has_adjacent_test = true
Quando parse() for chamado
Então has_test_coverage = true

Dado SourceFile com apenas:
  from typing import Protocol
  class FileProvider(Protocol):
      def files(self) -> list: ...
Quando parse() for chamado
Então has_test_coverage = true — declaration-only, isento de V2

Dado SourceFile com:
  def check(file: ParsedFile) -> list: ...
Quando parse() for chamado
Então public_interface.functions contém FunctionSignature {
    name: "check",
    params: ["ParsedFile"],
    return_type: Some("list")
}
— self/cls omitidos dos params

Dado SourceFile com:
  class FileWalker(FileProvider): ...
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "FileWalker",
    kind: TypeKind::Class,
    ..
}

Dado SourceFile com:
  class FileProvider(Protocol): ...
Quando parse() for chamado
Então public_interface.types contém TypeSignature {
    name: "FileProvider",
    kind: TypeKind::Interface,
    ..
}

Dado SourceFile com __all__ = ['foo', 'bar']
Quando parse() for chamado
Então public_interface.reexports não está vazio

Dado SourceFile em L1/contracts/ com:
  class FileProvider(Protocol): ...
  class LanguageParser(ABC): ...
  class _InternalHelper(Protocol): ...
Quando parse() for chamado
Então declared_traits = ["FileProvider", "LanguageParser"]
E "_InternalHelper" não aparece — prefixo _

Dado SourceFile em L3 com:
  from core.contracts import FileProvider  (alias core=01_core)
  class FileWalker(FileProvider): ...
  class InternalHelper: ...
Quando parse() for chamado
Então implemented_traits = ["FileProvider"]
E "InternalHelper" não aparece — sem base de contracts/

Dado SourceFile em L4 com:
  from core.contracts import HashRewriter  (alias)
  class L3HashAdapter(HashRewriter): ...
  class OutputFormatter: ...
Quando parse() for chamado
Então declarations contém:
  Declaration { kind: Class, name: "OutputFormatter", .. }
E NÃO contém Declaration para "L3HashAdapter"
— base de contracts/ é adapter, não capturado

Dado SourceFile em L4 com:
  class Config(Protocol): ...
Quando parse() for chamado
Então declarations NÃO contém Declaration para "Config"
— herda de Protocol, é contrato

Dado SourceFile .py sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado SourceFile .py vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })

Dado SourceFile com language = Language::Rust num PyParser
Quando parse() for chamado
Então retorna Err(ParseError::UnsupportedLanguage { .. })

Dado NullPromptReader e NullSnapshotReader como mocks
Quando parse() for chamado
Então nenhum acesso a disco ocorre durante testes
```

---

## 13. Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-19 | Criação inicial: PyParser com resolução física, header #, PyImport, V4 Python, V11 via import_name_map, V12 com exclusão de Protocol/ABC/adapters | py_parser.rs |
| 2026-03-19 | Implementação completa: find_child_by_kind com lifetime explícito, return_type sem move-while-borrowed, mock NullSnapshotReader corrigido para 'static, test file_with_implementation_is_not_declaration_only reescrito; wiring em mod.rs e main.rs; 334/334 testes, zero violations | py_parser.rs, 03_infra/mod.rs, 04_wiring/main.rs |
| 2026-03-19 | ADR-0009 correcção: ImportKind::PyImport removido; tabela de mapeamento Python→Direct/Glob/Alias/Named adicionada; nota sobre V4 usar file.language() na secção de tokens; restrições de agnósticidade adicionadas; critério de lab corrigido para sintaxe Python; critérios de ImportKind adicionados | py_parser.rs |
