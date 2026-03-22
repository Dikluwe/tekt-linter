# ⚖️ ADR-0013: Separação `Import` e `ModuleDecl` na IR

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-22

---

## Contexto

A IR actual representa todas as relações entre ficheiros como
`Import<'a>` em `ParsedFile.imports`. Isso unifica dois conceitos
distintos:

- `use foo::Bar;` — **dependência**: este ficheiro usa um símbolo
  definido noutro lugar
- `mod foo;` — **declaração de módulo**: este ficheiro inclui
  `foo.rs` como submódulo do mesmo crate

A confusão tornou-se visível durante a materialização de V14:
`mod foo;` estava a resolver para `Layer::Unknown` porque o
modelo não distinguia os dois casos, e o código de resolução de
imports não tinha tratamento explícito para declarações de módulo.
A correcção na Tarefa 5 de V13/V14 resolveu o sintoma (atribuir
`file_layer` correctamente via `resolve_file_layer`), mas não o
problema conceptual.

Num projecto de arquitectura, a clareza do modelo é um requisito
de primeira classe. Um `Import` com `target_layer == L1` e um
`ModuleDecl` com `target_layer == L1` têm significados
completamente diferentes — a IR não deve colapsar os dois.

---

## Decisão

Separar as duas relações na IR:

```rust
/// Dependência: `use foo::Bar;` — este ficheiro usa um símbolo externo.
pub struct Import<'a> {
    pub path: &'a str,
    pub symbol: Cow<'a, str>,
    pub target_layer: Layer,
    pub line: usize,
}

/// Declaração de módulo: `mod foo;` — este ficheiro inclui foo.rs
/// como submódulo. Apenas Rust tem esta construção.
pub struct ModuleDecl<'a> {
    pub name: &'a str,          // "foo" de `mod foo;`
    pub resolved_path: &'a Path, // caminho para foo.rs
    pub target_layer: Layer,    // camada de foo.rs
    pub line: usize,
}
```

`ParsedFile<'a>` passa a ter dois campos separados:

```rust
pub imports: Vec<Import<'a>>,
pub module_decls: Vec<ModuleDecl<'a>>,
```

### Impacto nas regras existentes

| Regra | Opera sobre | Mudança |
|-------|-------------|---------|
| V3 | `imports` | Sem mudança — `mod foo;` nunca foi o caso de uso |
| V4 | `imports` | Sem mudança |
| V14 | `imports` | Sem mudança — `mod foo;` sai de `imports` |
| Futura regra de estrutura | `module_decls` | Disponível sem hacks |

V3 e V14 passam a operar exclusivamente sobre dependências reais.
`mod foo;` deixa de poder gerar falsos positivos em qualquer
regra de import, por construção.

### Traits afectadas

`HasImports<'a>` permanece inalterada — opera sobre `imports`.

Nova trait opcional para regras de estrutura futura:

```rust
pub trait HasModuleDecls<'a> {
    fn module_decls(&self) -> &[ModuleDecl<'a>];
}
```

Não é necessária para nenhuma regra actual — declarar agora
para fechar o modelo.

---

## Prompts afectados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `ModuleDecl<'a>`, campo `module_decls` em `ParsedFile`, `HasModuleDecls` |
| `parsers/rust.md` | Extracção separada de `use_declaration` vs `mod_item` |
| `linter-core.md` | Nota sobre separação; regras de import operam sobre `imports` |

TypeScript e Python não têm `mod` — os seus parsers produzem
`module_decls: vec![]` por construção.

---

## Consequências

### ✅ Positivas

- A IR reflecte a distinção conceptual real — dependência vs
  estrutura
- V14 não pode ter falsos positivos por `mod foo;` por
  construção, sem código de tratamento especial
- Regras futuras sobre estrutura de crate têm onde operar
- O modelo é auto-documentado: ver `imports` ou `module_decls`
  num handler comunica intenção imediatamente

### ❌ Negativas

- Refactorização da IR: `ParsedFile`, `RustParser`,
  `HasImports` (verificar se `mod_item` estava a ser filtrado),
  e todos os testes que constroem `ParsedFile` directamente
- Custo estimado: médio — mecânico, sem lógica nova

### ⚙️ Neutras

- A correcção da Tarefa 5 (V13/V14) é substituída por esta
  separação — o comportamento observável é idêntico, o modelo
  é mais correcto
- TypeScript e Python não são afectados funcionalmente

---

## Alternativas Consideradas

| Alternativa | Decisão |
|-------------|---------|
| Manter `Import` unificado com `resolve_file_layer` para `mod` | Rejeitada — mascara distinção conceptual; falsos positivos possíveis por construção |
| Enum `Relation { Import, ModuleDecl }` em vez de dois campos | Possível — mas dois campos separados são mais simples de consumir nas traits e mais explícitos |

---

## Referências

- ADR-0001: Tree-sitter IR — modelo de `Import`
- ADR-0012: V14 — motivação para esta separação
- Tarefa 5 da materialização V13/V14 — correcção que este ADR
  substitui structuralmente
