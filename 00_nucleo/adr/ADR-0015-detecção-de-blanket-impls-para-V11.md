# ⚖️ ADR-0015: Detecção de Blanket Impls para V11

**Status**: `PROPOSTO`
**Data**: 2026-03-26

---

## Contexto

V11 (Dangling Contract) verifica se toda trait declarada em
`L1/contracts/` tem pelo menos um `impl Trait for Type` em L2
ou L3. O `RustParser` extrai `implemented_traits` lendo o tipo
concreto no campo `for` de cada `impl_item`.

Blanket impls têm um parâmetro genérico no campo `for`, não um
tipo concreto:
```rust
impl<T: World> TrackedWorld for T          // T é parâmetro
impl<T: A + B> Contract for T              // multi-bound
impl<T> Contract for T where T: A + B     // where clause
```

O parser actual ignora estes três padrões — o campo `for`
contém `T`, que não aparece em nenhum `declared_traits` de
L1/contracts/, logo V11 reporta a trait como dangling. Isto
é um falso positivo estrutural, não um problema do código.

### Regra de Pareto aplicada

Quatro padrões de blanket impl existem em Rust. Os três
padrões cobertos por este ADR representam a esmagadora maioria
dos casos reais:

| Padrão | Cobertura | Incluído |
|--------|-----------|----------|
| `impl<T: B> Trait for T` | ~60% | ✅ |
| `impl<T: B1 + B2> Trait for T` | ~25% | ✅ |
| `impl<T> Trait for T where T: B` | ~10% | ✅ |
| `impl<T: B> Trait for &T` / `Box<T>` | ~5% | ❌ |

O padrão 4 (`&T`, `Box<T>`, `Arc<T>`) é semanticamente distinto
— o tipo concreto que satisfaz o contrato não é `T` mas um
wrapper sobre `T`. Estes casos continuam disponíveis via
`[v11_blanket_exceptions]` no `crystalline.toml`.

---

## Decisão

### 1. Novo campo em `LocalIndex` e `ProjectIndex`
```rust
pub struct LocalIndex<'a> {
    // campos existentes...
    pub blanket_impl_traits: Vec<&'a str>,  // ADR-0015
}

pub struct ProjectIndex<'a> {
    // campos existentes...
    pub all_blanket_impl_traits: HashSet<&'a str>,  // ADR-0015
}
```

`blanket_impl_traits` regista os nomes de traits satisfeitas
por blanket impl no ficheiro. Populado apenas para ficheiros
em L2 ou L3 — mesma condição de `implemented_traits`.

### 2. Algoritmo de detecção no `RustParser`

Para cada `impl_item` em L2 ou L3, verificar se é blanket:

**Passo 1 — Recolher parâmetros genéricos do impl:**
Ler o nó `type_parameters` do `impl_item`. Extrair os nomes
dos parâmetros declarados (ex: `T`, `U`).

**Passo 2 — Verificar o tipo no campo `for`:**
Ler o nó `for` do `impl_item`. Se o tipo é um identificador
simples que pertence à lista de parâmetros do passo 1,
é blanket.

**Passo 3 — Extrair o nome da trait:**
Ler o campo `trait` do `impl_item` — mesmo mecanismo já
usado para `implemented_traits`.

**Passo 4 — Registar:**
Adicionar o nome da trait a `blanket_impl_traits`.

Os três padrões mapeiam para o mesmo algoritmo porque
tree-sitter resolve `where` clauses e multi-bounds no nó
`type_parameters` antes de expor o AST — o parser não precisa
de distinguir os padrões explicitamente.
```rust
fn extract_blanket_impls<'a>(
    root: Node,
    source: &'a [u8],
    layer: &Layer,
) -> Vec<&'a str> {
    if !matches!(layer, Layer::L2 | Layer::L3) {
        return vec![];
    }
    let mut result = Vec::new();
    for node in root.children_of_kind("impl_item") {
        // Passo 1: recolher parâmetros genéricos
        let type_params = node
            .child_by_field_name("type_parameters")
            .map(|n| collect_type_param_names(n, source))
            .unwrap_or_default();
        if type_params.is_empty() {
            continue; // impl concreto, já tratado por implemented_traits
        }
        // Passo 2: verificar se o tipo em `for` é parâmetro genérico
        let for_type = node
            .child_by_field_name("type")
            .and_then(|n| node_text(n, source));
        let is_blanket = for_type
            .map(|t| type_params.contains(t))
            .unwrap_or(false);
        if !is_blanket {
            continue; // impl<T> Trait for ConcreteType — não é blanket
        }
        // Passo 3: extrair nome da trait
        if let Some(trait_name) = node
            .child_by_field_name("trait")
            .and_then(|n| last_segment(n, source))
        {
            result.push(trait_name);
        }
    }
    result
}
```

### 3. Alteração em V11

Em `01_core/rules/dangling_contract.rs`, alterar a condição
de disparo:
```rust
// Antes
let dangling: Vec<_> = index
    .all_declared_traits
    .iter()
    .filter(|t| !index.all_implemented_traits.contains(*t))
    .collect();

// Depois
let dangling: Vec<_> = index
    .all_declared_traits
    .iter()
    .filter(|t| {
        !index.all_implemented_traits.contains(*t)
            && !index.all_blanket_impl_traits.contains(*t)
    })
    .collect();
```

### 4. Escape hatch para padrão 4

Para blanket impls sobre wrappers (`&T`, `Box<T>`, `Arc<T>`),
adicionar suporte a `[v11_blanket_exceptions]` no
`crystalline.toml`:
```toml
[v11_blanket_exceptions]
# Traits satisfeitas por blanket impl sobre wrapper —
# não detectáveis estaticamente sem type checker completo.
# "TrackedWorld" = "impl<T: World> TrackedWorld for &T"
```

V11 exclui estas traits da verificação sem tentar detectá-las.

### 5. TypeScript e Python

`TsParser` e `PyParser` não têm blanket impls no sentido Rust.
O padrão equivalente em TypeScript seria um `class implements`
com generic constraint — raro em `contracts/` e não detectado
como falso positivo hoje. Nenhuma alteração necessária nestes
parsers.

---

## Impacto na IR

### `LocalIndex` — campo adicional
```rust
pub blanket_impl_traits: Vec<&'a str>,  // V11 ADR-0015
```

### `ProjectIndex` — campo adicional
```rust
pub all_blanket_impl_traits: HashSet<&'a str>,  // V11 ADR-0015
```

### `merge_local` e `merge` em `ProjectIndex`
```rust
// Em merge_local:
self.all_blanket_impl_traits
    .extend(local.blanket_impl_traits.iter().copied());

// Em merge:
self.all_blanket_impl_traits
    .extend(other.all_blanket_impl_traits.iter());
```

---

## Prompts a criar

| Prompt | Conteúdo |
|--------|----------|
| nenhum | Alterações contidas em prompts existentes |

## Prompts a revisar

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `blanket_impl_traits` em `LocalIndex` e `ProjectIndex` |
| `project-index.md` | `all_blanket_impl_traits`, `merge_local`, `merge` |
| `parsers/rust.md` | `extract_blanket_impls`, algoritmo de 4 passos, restrições |
| `rules/dangling-contract.md` | Condição de disparo com `all_blanket_impl_traits` |
| `linter-core.md` | `[v11_blanket_exceptions]` no crystalline.toml |

---

## Sequência de implementação

1. `violation-types.md` + `project-index.md` — IR primeiro
2. `parsers/rust.md` — extracção no parser
3. `rules/dangling-contract.md` — condição de V11
4. `linter-core.md` — toml e documentação

---

## Consequências

### ✅ Positivas

- Três padrões cobertos (~95% dos casos reais) sem
  reimplementar o type checker do Rust
- Zero falsos positivos para blanket impls canónicos
- `all_blanket_impl_traits` é associativa e comutativa —
  Map-Reduce não precisa de alteração
- Escape hatch declarativo para o padrão 4 raro

### ❌ Negativas

- `RustParser` ganha responsabilidade adicional de detecção
  de genericidade — aumenta a complexidade de `rs_parser.rs`
- Padrão 4 (`&T`, `Box<T>`) não coberto automaticamente —
  requer anotação manual em `crystalline.toml`
- Blanket impls condicionais complexos
  (`where <T as A>::Output: B`) continuam fora de escopo —
  limitação declarada

### ⚙️ Neutras

- `TsParser` e `PyParser` não são afectados
- `merge` e `merge_local` ganham uma linha cada — custo zero
- `[v11_blanket_exceptions]` é opcional — projectos sem
  blanket impls de padrão 4 não precisam de o configurar

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Supressão via `[orphan_exceptions]` | Disponível hoje | Suprime, não detecta; não escala |
| Anotação `// @crystalline-satisfies Trait` | Zero alterações em V11 | Disciplina editorial; não verificado pelo compilador |
| Cobertura total incluindo padrão 4 | Cobertura 100% | Requer análise de tipos que o linter não faz |
| Integração com `rust-analyzer` para type inference | Cobertura real | Dependência pesada; viola isolamento de L3 |

---

## Referências

- ADR-0007: Fechamento Comportamental (V11 — base)
- ADR-0009: Isolamento de Parsers por Linguagem
- `parsers/rust.md` — `implemented_traits`, algoritmo actual
- `project-index.md` — `LocalIndex`, `ProjectIndex`, merge
- `rules/dangling-contract.md` — condição de disparo de V11
