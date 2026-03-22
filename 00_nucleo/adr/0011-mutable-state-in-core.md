# ⚖️ ADR-0011: V13 — Estado Global Mutável em L1 (MutableStateInCore)

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-20
**Implementado**: 2026-03-22

---

## Contexto

A pureza de L1 é garantida por três regras existentes:

- **V3** — proíbe imports de camadas de infra (L2, L3, L4)
- **V4** — proíbe símbolos de I/O conhecidos (std::fs, tokio::io, etc.)
- **V2** — exige cobertura de teste

Nenhuma destas regras detecta estado global mutável. Uma função
em L1 pode ser tecnicamente pura na assinatura — sem I/O, sem
imports proibidos — e ainda assim não ser determinística se ler
de um singleton estático, de um cache global ou de uma variável
`static mut`.

```rust
// Passa em V1–V12, mas não é pura:
static CACHE: Mutex<HashMap<String, Vec<Violation>>> = Mutex::new(HashMap::new());

pub fn check(file: &ParsedFile) -> Vec<Violation> {
    let mut cache = CACHE.lock().unwrap();
    if let Some(cached) = cache.get(file.path.to_str().unwrap()) {
        return cached.clone(); // retorno depende de estado externo invisível
    }
    // ...
}
```

O estado global mutável é o "I/O da memória": uma função que lê
de um `static mut` ou de um `Mutex` global não é pura porque o
seu retorno depende de fatores externos invisíveis na assinatura.
A IA usa estes padrões como atalho para evitar injeção de
dependências explícita.

### Constructs afectados

| Construct | Motivo de proibição |
|-----------|---------------------|
| `static mut T` | Estado global mutável sem qualquer protecção |
| `static Mutex<T>` | Estado global mutável com lock — ainda global |
| `static RwLock<T>` | Idem |
| `static OnceLock<T>` | Inicialização lazy global — singleton |
| `static LazyLock<T>` | Idem com closure de inicialização |
| `static AtomicXxx` | Estado global mutável por definição |
| `static RefCell<T>` | Não compila em posição estática sem `unsafe`, mas prevenir explicitamente |

**Não afectados** (estáticos imutáveis são aceitáveis):

| Construct | Motivo de permissão |
|-----------|---------------------|
| `static str` / `static &[u8]` | Constante imutável — determinística |
| `const T` | Constante — sem estado |
| `static [T; N]` imutável | Array de dados fixos — determinístico |

A distinção é: um `static` sem interior mutable não pode mudar
entre chamadas — é equivalente a uma constante. Um `static` com
qualquer forma de interior mutable pode mudar entre chamadas —
viola a pureza de L1.

---

## Decisão

### V13 — MutableStateInCore

Qualquer declaração de `static` com interior mutable em ficheiros
com `layer == L1` gera violação Error.

**Detecção em L3 (RustParser):**

O `RustParser` já extrai `declarations` de nível superior para V12.
V13 reusa a mesma fase de extracção — adicionar detecção de
`static_item` com tipo que contém constructs de sincronização
ou interior mutable.

Algoritmo de detecção:

```
Para cada nó static_item de nível superior em L1:
  1. Verificar se tem modificador `mut` → proibido directamente
  2. Extrair o tipo declarado como texto
  3. Verificar se o texto do tipo contém qualquer dos tokens proibidos:
     Mutex, RwLock, OnceLock, LazyLock, AtomicBool, AtomicI8,
     AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8,
     AtomicU16, AtomicU32, AtomicU64, AtomicUsize, AtomicPtr,
     RefCell, UnsafeCell
  4. Se sim → Violation V13 Error
```

A detecção por texto do tipo (não por import resolvido) é
intencional: um `static X: MyMutex<T>` onde `MyMutex` é um alias
local para `std::sync::Mutex` deve ser detectado pelo token
`MyMutex` se esse padrão for adicionado à lista — ou detectado
como suspeito via revisão humana. A lista cobre os nomes canónicos
da stdlib e do ecossistema Rust.

**Para TypeScript e Python:**

TypeScript não tem `static` no sentido de Rust. O equivalente
são variáveis de módulo mutáveis:

```typescript
// Proibido em L1 TypeScript:
let globalCache: Map<string, Violation[]> = new Map(); // mutable module-level
export let mutableState = 0;
```

Python:

```python
# Proibido em L1 Python:
_cache: dict = {}  # mutable module-level mutable dict
```

Para TypeScript e Python, a detecção é mais simples: variáveis
de módulo `let` (TS) ou atribuições de nível de topo a tipos
mutáveis (Python) em L1. Os parsers correspondentes serão
actualizados nos prompts `parsers/typescript.md` e
`parsers/python.md` quando V13 for materializado para essas
linguagens. Na fase inicial, V13 é implementado apenas para Rust.

**Nível**: Error — bloqueia CI por padrão.
**Não configurável** para ser suprimido individualmente — não há
caso legítimo de estado global mutável em L1.

**Excepção declarável em `crystalline.toml`:**
Nenhuma. Se um `static Mutex<T>` é genuinamente necessário em L1,
a decisão arquitectural está errada — o estado deve ser injetado
via parâmetro ou movido para L3.

---

## Impacto na IR e nos ficheiros

### Nova entidade — `StaticDeclaration<'a>`

```rust
pub struct StaticDeclaration<'a> {
    pub name: &'a str,
    pub type_text: &'a str, // texto bruto do tipo para detecção
    pub is_mut: bool,
    pub line: usize,
}
```

### `ParsedFile<'a>` — novo campo

```rust
pub static_declarations: Vec<StaticDeclaration<'a>>,
```

Populado pelo `RustParser` para ficheiros de qualquer camada —
V13 filtra por `layer == L1` internamente.

### Nova trait — `HasStaticDeclarations<'a>`

```rust
pub trait HasStaticDeclarations<'a> {
    fn layer(&self) -> &Layer;
    fn static_declarations(&self) -> &[StaticDeclaration<'a>];
    fn path(&self) -> &'a Path;
}
```

---

## Prompts afectados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `StaticDeclaration`, `HasStaticDeclarations`, campo em `ParsedFile` |
| `parsers/rust.md` | Extracção de `static_item` com detecção de interior mutable |
| `linter-core.md` | V13 nas verificações, lista de tokens proibidos |
| `sarif-formatter.md` | V13 na tabela SARIF e `EnabledChecks` |
| `rules/mutable-state-core.md` | Novo — V13 |

---

## Consequências

### ✅ Positivas

- Fecha o último vector de impureza em L1 não coberto por V4
- Força injeção explícita de estado — a IA não pode usar
  singletons como atalho
- Detecção baseada em tokens do tipo é robusta contra aliases
  de módulo simples

### ❌ Negativas

- `static` imutáveis legítimos (constantes de string, tabelas
  de dados fixos) não são afectados — mas a lista de tokens
  proibidos deve ser mantida actualizada se novos tipos de
  sincronização aparecerem no ecossistema
- Detecção por texto do tipo pode ter falsos negativos para
  aliases profundos (`type MyLock<T> = Mutex<T>`) — limitação
  declarada

### ⚙️ Neutras

- O projecto actual `crystalline-lint` não tem `static mut`
  em L1 — V13 não dispararia no estado actual
- A lista de tokens proibidos vive em `mutable-state-core.rs`
  em L1, não no toml — não é configurável por projecto

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Verificar via imports (std::sync::Mutex) | Consistente com V4 | Não detecta `static mut` puro; aliases de módulo burlam |
| Proibir todos os `static` em L1 | Zero falsos negativos | Proíbe constantes legítimas (tabelas, strings estáticas) |
| Detectar via análise de fluxo | Zero falsos negativos e positivos | Complexidade fora do âmbito de um linter arquitectural |

---

## Referências

- ADR-0004: Reformulação do Motor de Análise — pureza de L1
- ADR-0002: Trait-Based Testing — mocks isolam estado, nunca globais
- `rules/impure-core.md` — V4, precedente para lista de tokens proibidos
