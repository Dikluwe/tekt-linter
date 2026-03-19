# Prompt: Rule V4 - Impure Core (impure-core)

**Camada**: L1 (Core - Rules)
**Regra**: V4
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-18 (ADR-0009 correcção: V4 usa file.language() para seleccionar lista de símbolos proibidos)
**Arquivos gerados**:
  - 01_core/rules/impure_core.rs + test

---

## Contexto

A essência do estrato L1 é ser matematicamente funcional e
previsível: entradas resultam deterministicamente num resultado
sem interferência de I/O de rede, leituras de disco ou estado
não-determinístico. V4 é o guardião desta garantia.

A detecção é feita semanticamente sobre a AST — não é um
`.contains()` por regex no texto. Os tokens chegam a L1 já
resolvidos para FQN pelo parser em L3 (aliases resolvidos,
caminhos canonizados). V4 compara `token.symbol` contra a lista
de símbolos proibidos para a linguagem do arquivo.

**Agnósticidade de linguagem:** V4 não usa `ImportKind` para
distinguir linguagens — `ImportKind` descreve mecânica estrutural,
não sintaxe de linguagem. V4 usa `file.language()` para seleccionar
a lista de símbolos proibidos correspondente. As regras de
comparação são idênticas para todas as linguagens — apenas a
lista muda.

---

## Especificação

A regra recebe uma entidade via trait `HasTokens` e verifica cada
token contra a lista de símbolos proibidos para `file.language()`.

```rust
pub fn check<'a, T: HasTokens<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    let forbidden = forbidden_symbols_for(file.language());

    file.tokens()
        .iter()
        .filter(|token| is_forbidden_symbol(&token.symbol, forbidden))
        .map(|token| make_violation(file, token))
        .collect()
}

fn is_forbidden_symbol(symbol: &str, forbidden: &[&str]) -> bool {
    forbidden
        .iter()
        .any(|&f| symbol == f || symbol.starts_with(&format!("{}::", f))
                              || symbol.starts_with(&format!("{}.", f)))
}
```

---

## Listas de símbolos proibidos por linguagem

```rust
fn forbidden_symbols_for(language: &Language) -> &'static [&'static str] {
    match language {
        Language::Rust => &[
            // Sistema de ficheiros e I/O
            "std::fs",
            "std::io",
            "std::net",
            "std::process",
            "tokio::fs",
            "tokio::io",
            "tokio::process",
            // Bases de dados e rede externa
            "reqwest",
            "sqlx",
            "diesel",
            // Estado não-determinístico
            "std::time::SystemTime::now",
            "rand::random",
        ],

        Language::TypeScript => &[
            // Módulos Node.js de I/O (com e sem prefixo node:)
            "fs", "node:fs",
            "fs/promises", "node:fs/promises",
            "child_process", "node:child_process",
            "net", "node:net",
            "http", "node:http",
            "https", "node:https",
            "dgram", "node:dgram",
            "dns", "node:dns",
            "readline", "node:readline",
            // Estado não-determinístico
            "process.env",
            "Date.now",
            "Math.random",
        ],

        Language::Python => &[
            // Sistema de ficheiros e I/O
            "os",
            "os.path",
            "pathlib",
            "shutil",
            "subprocess",
            // Rede
            "socket",
            "urllib",
            "http.client",
            "ftplib",
            "smtplib",
            // Builtins de I/O
            "open",
            // Estado não-determinístico
            "random.random",
            "time.time",
            "datetime.now",
        ],

        Language::Unknown => &[],
    }
}
```

**`Language::Unknown`** retorna lista vazia — ficheiros de
linguagem não reconhecida não disparam V4. V8 (Alien File)
cobre o caso de ficheiros fora da topologia.

---

## Trait `HasTokens` — extensão necessária

Para que V4 possa chamar `file.language()`, a trait `HasTokens`
em `contracts/rule_traits.rs` expõe o campo `language`:

```rust
pub trait HasTokens<'a> {
    fn layer(&self) -> &Layer;
    fn language(&self) -> &Language;   // necessário para V4
    fn tokens(&self) -> &[Token<'a>];
    fn path(&self) -> &'a Path;
}
```

`ParsedFile` implementa `HasTokens` trivialmente — `language()`
retorna `&self.language`.

---

## Estrutura da Violação Gerada

```rust
Violation {
    rule_id: "V4".to_string(),
    level: ViolationLevel::Error,
    message: format!(
        "Núcleo Impuro: operação proibida '{}' detectada em L1",
        token.symbol
    ),
    location: Location {
        path: Cow::Borrowed(file.path()),
        line: token.line,
        column: token.column,
    },
}
```

---

## Restrições (L1 Pura)

- Aplica-se apenas a `layer == Layer::L1`
- Usa `file.language()` para seleccionar lista — nunca `ImportKind`
- `is_forbidden_symbol` verifica prefixo com `::` (Rust) e `.`
  (Python/TypeScript) para cobrir chamadas de método e submódulos
- Arquivos `cfg(test)` não são isentos automaticamente —
  a isenção é decidida pelo parser L3 via `has_test_coverage`.
  V4 não inspeciona atributos de teste directamente
- `Language::Unknown` → lista vazia → zero violações V4
- Os tokens chegam com FQN resolvido — aliases não burlam V4

---

## Critérios de Verificação

```
Dado arquivo L1 com language = Rust
E token { symbol: "std::fs::read", line: 10 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", location.line: 10 }

Dado arquivo L1 com language = Rust
E token { symbol: Cow::Owned("std::fs::read") }  (alias resolvido)
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4" }
— Owned tratado identicamente via Deref

Dado arquivo L1 com language = TypeScript
E token { symbol: "fs.readFileSync", line: 5 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", location.line: 5 }
— lista TypeScript seleccionada por file.language()

Dado arquivo L1 com language = TypeScript
E token { symbol: "Date.now", line: 8 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4" }
— estado não-determinístico proibido em qualquer linguagem

Dado arquivo L1 com language = Python
E token { symbol: "os.path.join", line: 3 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4", location.line: 3 }
— lista Python seleccionada por file.language()

Dado arquivo L1 com language = Python
E token { symbol: "subprocess.run", line: 7 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4" }

Dado arquivo L1 com language = Python
E token { symbol: "open", line: 12 }
Quando V4::check() for chamado
Então retorna Violation { rule_id: "V4" }
— builtin de I/O proibido

Dado arquivo L1 com language = Rust
E token { symbol: "my_module::compute", line: 2 }
Quando V4::check() for chamado
Então retorna vec![] — símbolo puro, não proibido

Dado arquivo L3 com language = Rust
E token { symbol: "std::fs::read", line: 10 }
Quando V4::check() for chamado
Então retorna vec![] — V4 apenas em L1

Dado arquivo L1 com language = Unknown
E token { symbol: "anything", line: 1 }
Quando V4::check() for chamado
Então retorna vec![] — lista vazia para Unknown

Dado arquivo L1 com language = Rust
E dois tokens proibidos em linhas diferentes
Quando V4::check() for chamado
Então retorna duas violações — uma por token

Dado MockFile implementando HasTokens com language = Python
E token { symbol: "random.random" }
Quando V4::check() for chamado sem acesso a disco
Então retorna Violation — testável com mock puro
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | impure_core.rs |
| 2026-03-18 | ADR-0009 correcção: V4 multi-linguagem via file.language(); forbidden_symbols_for() com listas por Language; HasTokens ganha language(); critérios TypeScript e Python adicionados; restrição sobre ImportKind documentada | impure_core.rs, rule_traits.rs |
