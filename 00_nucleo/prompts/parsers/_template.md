# Prompt: Parser Template (parsers/_template)

> **Este ficheiro é um contrato editorial, não um prompt de materialização.**
> Não gera código directamente. Define as secções obrigatórias e as
> invariantes que todo prompt em `parsers/` deve satisfazer.
> Ao criar `parsers/<lang>.md`, copiar esta estrutura e preencher
> cada secção para a gramática da linguagem alvo.

**Camada**: L0 (Documentação — contrato de parser)
**Criado em**: 2026-03-18 (ADR-0009)
**Revisado em**: 2026-03-18 (ADR-0009 correcção: ImportKind semântico, V4 via language())
**Arquivos gerados**: nenhum directamente

---

## Por que este template existe

Cada linguagem suportada pelo linter tem um parser em L3
(`03_infra/<lang>_parser.rs`) que implementa a trait `LanguageParser`.
Todos os parsers produzem o mesmo `ParsedFile<'a>` — a IR agnóstica
que as regras V1–V12 consomem.

Sem um contrato editorial, parsers de linguagens diferentes tendem
a cobrir campos diferentes, a tratar casos de borda de forma
inconsistente, e a omitir secções que parecem irrelevantes para a
linguagem mas que bloqueiam regras downstream. Este template previne
isso impondo a mesma estrutura para todos.

**O `parsers/rust.md` é a implementação de referência.** Quando
uma secção deste template parecer ambígua, o comportamento do
`RustParser` é a fonte de verdade.

---

## Invariantes obrigatórias

Antes de preencher as secções, garantir que o parser satisfaz:

### I1 — Zero-Copy (ADR-0004)
`parse()` recebe `&'a SourceFile` e retorna `ParsedFile<'a>`.
Todas as strings extraídas do AST são `&'a str` apontando para
o buffer do `SourceFile`. Proibido `.to_string()` para conteúdo
do buffer. Única excepção documentada: `PromptHeader.current_hash`
é `Option<String>` porque é calculado do disco, não do buffer.

### I2 — Resolução física de camadas (ADR-0009)
A camada de um import é determinada pelo caminho físico no disco,
não pelo texto do import. O algoritmo obrigatório tem quatro passos:

**Passo 1 — Detecção de package externo:**
Se o import não começa com `./`, `../` ou uma chave de
`[<lang>_aliases]` no `crystalline.toml`, é um package externo
→ `Layer::Unknown` directamente, sem álgebra de paths.

**Passo 2 — Resolução de alias** (se aplicável):
Se o import começa com uma chave de `[<lang>_aliases]`, substituir
pelo valor correspondente antes de qualquer álgebra.

**Passo 3 — Álgebra de paths com verificação de fuga:**
```rust
fn normalize(path: &Path, project_root: &Path) -> Option<PathBuf> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // pop() em vec vazio = tentativa de sair da raiz
                // → caminho inválido → caller retorna Layer::Unknown
                if components.is_empty() {
                    return None;
                }
                components.pop();
            }
            Component::CurDir => {}
            c => { components.push(c); }
        }
    }
    let result: PathBuf = components.iter().collect();
    // Garantia adicional: resultado dentro da raiz do projecto
    if result.starts_with(project_root) {
        Some(result)
    } else {
        None
    }
}
```

`std::fs::canonicalize` é proibido — normalização algébrica apenas.

**Passo 4 — Reutilização de `resolve_file_layer`:**
```rust
let base = file.path.parent().unwrap_or(Path::new("."));
let normalized = normalize(&base.join(import_str_after_alias), &project_root);
let target_layer = normalized
    .map(|p| resolve_file_layer(&p, &project_root, &config))
    .unwrap_or(Layer::Unknown);
```

A mesma função do `FileWalker` é a fonte de verdade para todos
os parsers. Zero duplicação de lógica de resolução.

**Excepção — Rust:**
`crate::` é absoluto por construção — não tem o vector de fuga
de paths relativos. O `RustParser` usa `LayerResolver` baseado
em `crate::` sem normalização física. Esta é a única excepção
permitida ao I2.

### I3 — Segurança de normalização
`normalize()` deve retornar `None` para qualquer path que tente
escapar da raiz do projecto via `../` excessivos. `None` propaga
para `Layer::Unknown`. Nunca silenciar a fuga, nunca panicar.

### I4 — Filtros no parser, não nas regras
- `declared_traits`: apenas em L1/contracts/
- `implemented_traits`: apenas em L2|L3
- `declarations`: para todos os arquivos (V12 filtra por layer)

As regras de L1 não filtram por layer ou subdir ao ler estes
campos — confiam que o parser já fez a filtragem correcta.

### I5 — Equivalente de `impl Trait for Type` nunca em `declarations`
Apenas implementações sem contrato são capturadas como
`DeclarationKind::Impl` (Rust) ou equivalente. O padrão de
adapter — `impl Trait for Type` em Rust, `class implements`
em TypeScript/Python, etc. — nunca é capturado em `declarations`.

### I6 — UnsupportedLanguage para ficheiros de outra linguagem
Se `file.language` não corresponde à linguagem do parser,
retornar `Err(ParseError::UnsupportedLanguage { .. })`.

### I7 — `ImportKind` semântico, nunca sintáctico (ADR-0009 correcção)
`ImportKind` descreve a mecânica estrutural do import, não a
sintaxe da linguagem. Nunca adicionar variantes que referenciem
uma linguagem específica (`RustUse`, `EsImport`, `PyImport`
são todas proibidas).

O mapeamento obrigatório para qualquer linguagem:

| Mecânica | `ImportKind` | Exemplos por linguagem |
|----------|-------------|------------------------|
| Import de módulo ou símbolo único | `Direct` | Rust: `use X`, `extern crate X`, `mod X;` / TS: `import X from`, `import '...'` / Python: `import os` |
| Import de todos os símbolos | `Glob` | Rust: `use X::*` / TS: `import * as ns from` / Python: `from X import *` |
| Import com renomeação | `Alias` | Rust: `use X as Y` / TS: `import X as Y`, `import { A as B }` / Python: `import X as Y` |
| Import de subconjunto nomeado | `Named` | Rust: `use {A, B}` / TS: `import { A, B }` / Python: `from X import A, B` |

### I8 — V4 usa `file.language()`, não `ImportKind`
A regra V4 (Impure Core) selecciona a lista de símbolos proibidos
via `forbidden_symbols_for(file.language())` — nunca via `ImportKind`.
O parser é responsável por:
1. Extrair os tokens (call expressions, imports proibidos)
2. Garantir que `ParsedFile.language` está correctamente preenchido
3. Nunca codificar lógica de "quais símbolos são proibidos para
   esta linguagem" no parser — isso pertence a `impure_core.rs`

---

## Secções obrigatórias de cada `parsers/<lang>.md`

### 1. Cabeçalho

```markdown
# Prompt: <Linguagem> Parser (parsers/<lang>)

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-<lang>`
**Criado em**: YYYY-MM-DD
**Revisado em**: ...
**Arquivos gerados**:
  - 03_infra/<lang>_parser.rs + test
```

### 2. Contexto

Descrever brevemente o que a linguagem é e por que está a ser
adicionada. Referenciar os ADRs relevantes. Listar as dependências
injetadas no construtor. Documentar qual mecanismo de resolução
de camadas é usado (física via `normalize` ou, apenas para Rust,
`crate::` absoluto).

### 3. Header cristalino

Documentar o formato exacto do header nesta linguagem:
- Qual marcador de comentário é usado (`//`, `#`, `--`, etc.)
- Como o bloco é delimitado (primeira linha não-marcador termina)
- Justificativa para a escolha (por que não JSDoc, docstring, etc.)
- Exemplo completo copiável

### 4. Resolução de camadas — LayerResolver

Documentar o algoritmo de resolução para esta linguagem:
- Se usa resolução física: descrever os 4 passos com código
- Se usa outro mecanismo (apenas Rust): justificar a excepção ao I2
- Documentar como `target_subdir` é extraído após resolução
- Documentar como aliases são configurados no `crystalline.toml`

### 5. Extracção de imports (V3, V9, V10)

Tabela com os nós AST que geram `Import<'a>` e como cada campo
é populado. Incluir obrigatoriamente:
- Tabela de mapeamento `Nó AST → ImportKind` usando apenas
  `Direct/Glob/Alias/Named` — nunca variantes específicas de linguagem
- Como `target_layer` é resolvido
- Como `target_subdir` é extraído
- Como `Layer::Lab` é detectado

### 6. Extracção de tokens — símbolos proibidos (V4)

Documentar como os tokens são extraídos do AST para que V4 possa
verificá-los. Incluir obrigatoriamente:
- Nota explícita: **"V4 usa `file.language()` para seleccionar
  a lista de símbolos proibidos — não usa `ImportKind`"**
- Quais nós AST geram tokens (call expressions, macro invocations,
  import statements proibidos, etc.)
- Se a linguagem requer Motor de Duas Fases (resolução de aliases
  como Rust) ou extracção directa (TypeScript, Python)
- A lista de símbolos proibidos **NÃO vive aqui** — vive em
  `impure_core.rs` via `forbidden_symbols_for(language)`.
  O prompt apenas documenta como os tokens são extraídos.

### 7. Test coverage (V2)

Documentar como `has_test_coverage` é determinado:
- Qual construto de teste é detectado no AST
- Qual convenção de nome de ficheiro de teste é reconhecida
- O que constitui um ficheiro "declaration-only" isento

### 8. Interface pública (V6)

Tabela com os nós AST que geram cada campo de `PublicInterface`:
- `functions` → qual nó, qual modificador de visibilidade,
  como `name`/`params`/`return_type` são extraídos
- `types` → qual nó, qual `TypeKind` (Struct/Enum/Trait/Class/Interface/TypeAlias)
  com descrição de `members` por kind
- `reexports` → qual nó
- `prompt_snapshot` → via `PromptSnapshotReader` (idêntico para todas)
- Regras de normalização de tipos (whitespace colapsado, etc.)

### 9. Traits/interfaces declaradas (V11) — `declared_traits`

Documentar:
- Qual construto é equivalente a `trait`/`interface` nesta linguagem
- Qual modificador de visibilidade é equivalente a `pub`/`export`
- Filtro: apenas em L1/contracts/
- Exemplo com nó AST → entrada em `declared_traits`

Se a linguagem não tem interfaces formais (ex: Go usa duck typing),
documentar explicitamente como "não aplicável" e que
`declared_traits` é sempre `[]` para esta linguagem.

### 10. Traits/interfaces implementadas (V11) — `implemented_traits`

Documentar:
- Qual construto é equivalente a `impl Trait for Type`
- Como extrair o nome simples da trait/interface
- Como tratar múltiplas implementações numa só declaração
- Filtro: apenas em L2|L3
- Exemplo com nó AST → entrada em `implemented_traits`

### 11. Declarações de tipo (V12) — `declarations`

Tabela com os nós AST que geram `Declaration<'a>` e qual
`DeclarationKind` cada um produz. Incluir explicitamente:
- O que NÃO é capturado (equivalente a `impl Trait for Type`)
- Para todos os arquivos, sem filtro de layer

### 12. Assinatura do construtor

```rust
pub struct <Lang>Parser<R, S>
where
    R: PromptReader,
    S: PromptSnapshotReader,
{ ... }

impl<R: PromptReader, S: PromptSnapshotReader> <Lang>Parser<R, S> {
    pub fn new(prompt_reader: R, snapshot_reader: S, config: CrystallineConfig) -> Self { ... }
}

impl<R, S> LanguageParser for <Lang>Parser<R, S> { ... }
```

Documentar a ordem de extracção dentro de `parse()` como
comentários no bloco do método. Incluir o mapeamento
`Nó AST → ImportKind` nos comentários.

### 13. Restrições

Lista de restrições específicas desta linguagem, além das
invariantes I1–I8 que se aplicam a todos os parsers. Incluir
sempre:
- Restrição de `parse()` receber `&'a SourceFile`
- Proibição de `.to_string()` para conteúdo do buffer
- `normalize()` retorna `Option<PathBuf>` — `None` propaga para
  `Layer::Unknown`, nunca silenciado, nunca panic
- `std::fs::canonicalize` proibido
- **`ImportKind` nunca contém variantes específicas desta linguagem**
- **V4 usa `file.language()`, não `ImportKind`**
- `PromptReader` e `PromptSnapshotReader` são injetados
- `std::io::Error` nunca atravessa para L1

### 14. Critérios de Verificação

Conjunto de cenários dado/quando/então cobrindo obrigatoriamente:
- Header completo → todos os campos populados
- Import relativo com `../` normal → camada correcta
- Import com `../` excessivos (escapa da raiz) → `Layer::Unknown`
- Import com alias configurado → camada correcta após resolução
- Package externo (npm/pip/crate) → `Layer::Unknown`
- Import para `lab/` → `Layer::Lab`
- Cada variante de `ImportKind` (Direct, Glob, Alias, Named) →
  critério próprio confirmando o mapeamento correcto
- Símbolo proibido em L1 → token capturado
- Test coverage detectado → `has_test_coverage = true`
- Ficheiro declaration-only → `has_test_coverage = true`
- Interface/trait pública em L1/contracts/ → `declared_traits`
- Implementação em L3 → `implemented_traits`
- Múltiplas implementações numa declaração → todas capturadas
- Declaração de tipo em ficheiro → `declarations`
- Equivalente de `impl-com-trait` → NÃO em `declarations`
- Source inválido → `Err(ParseError::SyntaxError)`
- Source vazio → `Err(ParseError::EmptySource)`
- Linguagem errada → `Err(ParseError::UnsupportedLanguage)`

### 15. Histórico de Revisões

Tabela padrão com Data, Motivo, Arquivos afetados.

---

## Checklist antes de submeter `parsers/<lang>.md`

```
[ ] Todas as 15 secções presentes e preenchidas
[ ] Secções "não aplicável" documentadas explicitamente (não omitidas)
[ ] I1 Zero-Copy documentado nas Restrições
[ ] I2 Resolução física documentada no LayerResolver (ou excepção justificada)
[ ] I3 normalize() com verificação de fuga documentada
[ ] I4 Filtros de declared_traits/implemented_traits documentados
[ ] I5 Equivalente de impl-com-trait explicitamente excluído de declarations
[ ] I6 UnsupportedLanguage nos Critérios
[ ] I7 ImportKind semântico — tabela Direct/Glob/Alias/Named na secção 5
    Nenhuma variante específica de linguagem (sem EsImport, PyImport, etc.)
[ ] I8 Nota "V4 usa file.language(), não ImportKind" na secção 6
    Lista de símbolos proibidos NÃO está no prompt — está em impure_core.rs
[ ] Critérios cobrem import com ../ excessivos → Layer::Unknown
[ ] Critérios cobrem import para lab/ → Layer::Lab
[ ] Critérios cobrem cada variante de ImportKind (Direct/Glob/Alias/Named)
[ ] @prompt header em 03_infra/<lang>_parser.rs aponta para parsers/<lang>.md
[ ] orphan_exceptions em crystalline.toml NÃO inclui parsers/<lang>.md
    (este prompt TEM materialização — rs_parser.rs / ts_parser.rs / etc.)
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-18 | Criação inicial (ADR-0009): contrato editorial para parsers isolados por linguagem; invariantes I1–I6; 15 secções obrigatórias; checklist | — |
| 2026-03-18 | ADR-0009 correcção: I7 adicionado (ImportKind semântico — tabela Direct/Glob/Alias/Named, proibição de variantes de linguagem); I8 adicionado (V4 usa file.language(), não ImportKind); secção 5 actualizada com nota sobre tabela de mapeamento; secção 6 actualizada com nota obrigatória sobre file.language(); secção 12 actualizada com mapeamento nos comentários do construtor; secção 13 com duas restrições novas; checklist com itens I7 e I8 | — |
