# Prompt: Parser Template (parsers/_template)

> **Este ficheiro é um contrato editorial, não um prompt de materialização.**
> Não gera código directamente. Define as secções obrigatórias e as
> invariantes que todo prompt em `parsers/` deve satisfazer.
> Ao criar `parsers/<lang>.md`, copiar esta estrutura e preencher
> cada secção para a gramática da linguagem alvo.

**Camada**: L0 (Documentação — contrato de parser)
**Criado em**: 2026-03-18 (ADR-0009)
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
                // → caminho inválido → Layer::Unknown
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
    // Garantia adicional: resultado deve estar dentro da raiz
    if result.starts_with(project_root) {
        Some(result)
    } else {
        None
    }
}
```

`Option<PathBuf>` — `None` propaga para `Layer::Unknown`.
`std::fs::canonicalize` é **proibido** — requer que o ficheiro
exista no disco, falha em CI com repos parciais.

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
para `Layer::Unknown` — o import é tratado como externo e ignorado
pelas regras de topologia. Nunca silenciar a fuga, nunca panicar.

### I4 — Filtros no parser, não nas regras
- `declared_traits`: apenas em L1/contracts/
- `implemented_traits`: apenas em L2|L3
- `declarations`: para todos os arquivos (V12 filtra por layer)

As regras de L1 não filtram por layer ou subdir ao ler estes campos
— confiam que o parser já fez a filtragem correcta.

### I5 — `impl Trait for Type` nunca em `declarations`
Apenas `impl Type { ... }` sem trait é capturado como
`DeclarationKind::Impl`. O equivalente em outras linguagens
(class que implementa interface, struct que implementa trait)
também não deve ser capturado em `declarations` — é o padrão
de adapter permitido em L4.

### I6 — UnsupportedLanguage para ficheiros de outra linguagem
Se `file.language` não corresponde à linguagem do parser,
retornar `Err(ParseError::UnsupportedLanguage { .. })`.

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
é populado. Incluir:
- Qual `ImportKind` é usado para esta linguagem
- Como `target_layer` é resolvido
- Como `target_subdir` é extraído
- Como `Layer::Lab` é detectado

### 6. Extracção de tokens — símbolos proibidos (V4)

Lista completa dos símbolos proibidos em L1 para esta linguagem.
Para cada símbolo: qual nó AST o detecta e qual é o FQN resolvido.
Documentar se a linguagem tem sistema de aliases que requer
resolução de FQN (como o Motor de Duas Fases do Rust) ou se
os imports directos são suficientes.

### 7. Test coverage (V2)

Documentar como `has_test_coverage` é determinado:
- Qual construto de teste é detectado no AST
- Qual convenção de nome de ficheiro de teste é reconhecida
- O que constitui um ficheiro "declaration-only" isento

### 8. Interface pública (V6)

Tabela com os nós AST que geram cada campo de `PublicInterface`:
- `functions` → qual nó, qual modificador de visibilidade
- `types` → qual nó, qual `TypeKind` (Struct/Enum/Trait/Class/Interface/TypeAlias)
- `reexports` → qual nó
- `prompt_snapshot` → via `PromptSnapshotReader` (idêntico para todas as linguagens)

Documentar normalização de tipos se aplicável.

### 9. Traits/interfaces declaradas (V11) — `declared_traits`

Documentar:
- Qual construto é equivalente a `trait` nesta linguagem
- Qual modificador de visibilidade é equivalente a `pub`
- Filtro: apenas em L1/contracts/
- Exemplo com nó AST → entrada em `declared_traits`

Se a linguagem não tem interfaces formais (ex: Go usa duck typing),
documentar explicitamente como "não aplicável" e que
`declared_traits` é sempre `[]` para esta linguagem.

### 10. Traits/interfaces implementadas (V11) — `implemented_traits`

Documentar:
- Qual construto é equivalente a `impl Trait for Type`
- Como extrair o nome simples da trait/interface
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

impl<R, S> LanguageParser for <Lang>Parser<R, S> { ... }
```

Documentar a ordem de extracção dentro de `parse()` como
comentários no bloco do método.

### 13. Restrições

Lista de restrições específicas desta linguagem, além das
invariantes I1–I6 que se aplicam a todos os parsers. Incluir
sempre:
- Restrição de `parse()` receber `&'a SourceFile`
- Proibição de `.to_string()` para conteúdo do buffer
- `PromptReader` e `PromptSnapshotReader` são injetados
- `std::io::Error` nunca atravessa para L1

### 14. Critérios de Verificação

Conjunto de cenários dado/quando/então cobrindo obrigatoriamente:
- Header completo → todos os campos populados
- Import relativo normal → camada correcta
- Import com `../` excessivos → `Layer::Unknown`
- Import com alias → camada correcta após resolução
- Package externo → `Layer::Unknown`
- Import para `lab/` → `Layer::Lab`
- Símbolo proibido em L1 → token capturado
- Test coverage detectado → `has_test_coverage = true`
- Ficheiro declaration-only → `has_test_coverage = true`
- Interface/trait pública em L1/contracts/ → `declared_traits`
- Implementação em L3 → `implemented_traits`
- Declaração de tipo em L4 → `declarations`
- `impl`/equivalente com trait → NÃO em `declarations`
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
[ ] Critérios cobrem import com ../ excessivos → Layer::Unknown
[ ] Critérios cobrem import para lab/ → Layer::Lab
[ ] @prompt header em 03_infra/<lang>_parser.rs aponta para parsers/<lang>.md
[ ] orphan_exceptions em crystalline.toml NÃO inclui parsers/<lang>.md
    (este prompt TEM materialização — rs_parser.rs / ts_parser.rs / etc.)
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-18 | Criação inicial (ADR-0009): contrato editorial para parsers isolados por linguagem; invariantes I1–I6; 15 secções obrigatórias; checklist | — |
