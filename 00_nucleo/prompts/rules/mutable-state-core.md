# Prompt: Rule V13 — Mutable State In Core (mutable-state-core)
Hash do Código: 9c9f8d7c

**Camada**: L1 (Core — Rules)
**Regra**: V13
**Criado em**: 2026-03-20 (ADR-0011)
**Arquivos gerados**:
  - 01_core/rules/mutable_state_core.rs + test

---

## Contexto

A pureza de L1 exige que as suas funções sejam determinísticas —
para a mesma entrada, sempre o mesmo retorno. Estado global
mutável viola esta propriedade: uma função que lê de um `static
Mutex<T>` ou de um `LazyLock<T>` não é pura porque o seu retorno
depende de estado externo invisível na assinatura.

V4 proíbe I/O explícito. V13 proíbe o "I/O da memória": qualquer
declaração `static` com interior mutable em L1.

---

## Tokens proibidos em posição `static`

Qualquer `static_item` em L1 cujo tipo contém um dos seguintes
tokens gera V13 Error:

```
Mutex, RwLock, OnceLock, LazyLock,
AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, AtomicPtr,
RefCell, UnsafeCell
```

`static mut T` sem nenhum destes tokens também é proibido
(modificador `mut` directo).

**Permitidos** (estáticos imutáveis):

```rust
static RULE_ID: &str = "V13";             // OK — imutável
static FORBIDDEN: &[&str] = &["Mutex"];   // OK — imutável
const MAX: usize = 100;                   // OK — constante
```

---

## Especificação

V13 opera sobre `ParsedFile.static_declarations` por arquivo,
na fase Map. Aplica-se apenas a arquivos com `layer == L1`.

### Nova entidade — `StaticDeclaration<'a>`

```rust
/// Declaração static de nível superior.
/// Populada pelo RustParser para todos os arquivos.
/// V13 filtra por layer == L1 internamente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticDeclaration<'a> {
    pub name: &'a str,
    /// Texto bruto do tipo declarado (para detecção de tokens).
    pub type_text: &'a str,
    /// true se declarado como `static mut`
    pub is_mut: bool,
    pub line: usize,
}
```

### Nova trait — `HasStaticDeclarations<'a>`

```rust
pub trait HasStaticDeclarations<'a> {
    fn layer(&self) -> &Layer;
    fn static_declarations(&self) -> &[StaticDeclaration<'a>];
    fn path(&self) -> &'a Path;
}
```

### Verificação

```rust
const MUTABLE_STATE_TOKENS: &[&str] = &[
    "Mutex", "RwLock", "OnceLock", "LazyLock",
    "AtomicBool", "AtomicI8", "AtomicI16", "AtomicI32", "AtomicI64",
    "AtomicIsize", "AtomicU8", "AtomicU16", "AtomicU32", "AtomicU64",
    "AtomicUsize", "AtomicPtr", "RefCell", "UnsafeCell",
];

pub fn check<'a, T: HasStaticDeclarations<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.static_declarations()
        .iter()
        .filter(|s| is_mutable_static(s))
        .map(|s| Violation {
            rule_id: "V13".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Estado global mutável em L1: '{}' usa '{}'. \
                 Estado deve ser injectado por parâmetro, não partilhado globalmente.",
                s.name,
                offending_token(s),
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: s.line,
                column: 0,
            },
        })
        .collect()
}

fn is_mutable_static(s: &StaticDeclaration) -> bool {
    if s.is_mut {
        return true;
    }
    MUTABLE_STATE_TOKENS.iter().any(|token| s.type_text.contains(token))
}

fn offending_token(s: &StaticDeclaration) -> &str {
    if s.is_mut {
        return "mut";
    }
    MUTABLE_STATE_TOKENS
        .iter()
        .find(|token| s.type_text.contains(*token))
        .copied()
        .unwrap_or("estado mutável")
}
```

---

## Extracção em L3 (RustParser)

Para cada `static_item` de nível superior:
- `name`: campo `name` do nó
- `type_text`: texto bruto do campo `type` — `&'a str` do buffer
- `is_mut`: presença do token `mut` entre `static` e o nome
- `line`: `node.start_position().row + 1`

```rust
// Exemplo de extracção:
// static mut COUNTER: u32 = 0;
// → StaticDeclaration { name: "COUNTER", type_text: "u32", is_mut: true, line: N }

// static CACHE: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
// → StaticDeclaration { name: "CACHE", type_text: "Mutex<HashMap<String, u32>>", is_mut: false, line: N }
```

Populado para todos os arquivos — V13 filtra por `layer == L1`
internamente, tal como V12 faz com `declarations`.

---

## Restrições (L1 Pura)

- Opera sobre `ParsedFile.static_declarations` — zero I/O
- `MUTABLE_STATE_TOKENS` é constante em L1 — não configurável
  por projecto via toml
- `static mut` é sempre Error, independentemente do tipo
- Detecção por texto do tipo pode ter falsos negativos para
  aliases profundos — limitação declarada (ADR-0011)
- Sem excepções configuráveis: não há caso legítimo de estado
  global mutável em L1

---

## Critérios de Verificação

```
Dado arquivo L1 com:
  static mut COUNTER: u32 = 0;
Quando V13::check() for chamado
Então retorna Violation { rule_id: "V13", level: Error, line: N }
— static mut é sempre proibido

Dado arquivo L1 com:
  static CACHE: Mutex<HashMap<String, Vec<Violation>>> = Mutex::new(HashMap::new());
Quando V13::check() for chamado
Então retorna Violation { rule_id: "V13", level: Error }
— Mutex em posição static é proibido

Dado arquivo L1 com:
  static INSTANCE: OnceLock<Config> = OnceLock::new();
Quando V13::check() for chamado
Então retorna Violation { rule_id: "V13", level: Error }
— OnceLock é singleton — proibido

Dado arquivo L1 com:
  static TABLE: LazyLock<HashMap<&str, Layer>> = LazyLock::new(|| HashMap::new());
Quando V13::check() for chamado
Então retorna Violation { rule_id: "V13", level: Error }
— LazyLock é inicialização lazy global — proibido

Dado arquivo L1 com:
  static RULE_ID: &str = "V13";
Quando V13::check() for chamado
Então retorna vec![]
— estático imutável de tipo primitivo — permitido

Dado arquivo L1 com:
  static FORBIDDEN_TOKENS: &[&str] = &["Mutex", "RwLock"];
Quando V13::check() for chamado
Então retorna vec![]
— slice imutável de constantes — permitido
— (o texto "Mutex" dentro de um literal de string não é um tipo Mutex)

Dado arquivo L1 com:
  static ATOMIC: AtomicUsize = AtomicUsize::new(0);
Quando V13::check() for chamado
Então retorna Violation { rule_id: "V13", level: Error }
— AtomicUsize é estado global mutável por definição

Dado arquivo L3 com:
  static CACHE: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
Quando V13::check() for chamado
Então retorna vec![]
— V13 aplica-se apenas a L1

Dado arquivo L1 com dois statics proibidos
Quando V13::check() for chamado
Então retorna duas violations — uma por static
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-20 | Criação inicial (ADR-0011) | mutable_state_core.rs |
