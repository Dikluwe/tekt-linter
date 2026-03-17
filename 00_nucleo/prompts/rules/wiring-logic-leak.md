# Prompt: Rule V12 - Wiring Logic Leak (wiring-logic-leak)

**Camada**: L1 (Core — Rules)
**Regra**: V12
**Criado em**: 2026-03-16 (ADR-0007)
**Arquivos gerados**:
  - 01_core/rules/wiring_logic_leak.rs + test

---

## Contexto

L4 (`04_wiring/`) tem um único propósito: instanciar componentes de
L3, injetá-los onde L1 e L2 precisam, e inicializar o servidor ou
pipeline. L4 não cria tipos, não contém lógica de negócio, não
toma decisões sobre dados — apenas liga as peças que já existem.

Na prática, a IA tende a acumular lógica em `main.rs` quando:
- Precisa de uma pequena transformação antes de injetar
- Quer um struct de adapter que "não vale a pena" mover para L3
- Resolve um caso especial de formatação de saída que "pertence" ao wiring

Com o tempo, L4 vira um God Object. V3 protege a direção dos
imports mas não a densidade de declarações. V12 protege a pureza
estrutural de L4 detectando declarações de tipo que não deveriam
existir ali.

---

## Especificação

V12 opera sobre `ParsedFile` por arquivo, na fase Map.
Aplica-se apenas a arquivos com `layer == L4`.

### Nova entidade — `Declaration<'a>`

```rust
pub struct Declaration<'a> {
    pub kind: DeclarationKind,
    pub name: &'a str,
    pub line: usize,
}

pub enum DeclarationKind {
    Struct,
    Enum,
    Impl, // apenas impl sem trait (impl Type { ... })
}
```

`ParsedFile` ganha o campo:
```rust
pub declarations: Vec<Declaration<'a>>,
```

Populado por `RustParser` a partir de nós de nível superior:
- `struct_item` → `DeclarationKind::Struct`
- `enum_item` → `DeclarationKind::Enum`
- `impl_item` sem trait → `DeclarationKind::Impl`

`impl Trait for Type` **não** é proibido em L4 — é o padrão de
adapter. Apenas `impl Type { ... }` sem trait é proibido, pois
indica lógica de negócio embutida no wiring.

### Nova trait — `HasWiringPurity`

```rust
pub trait HasWiringPurity<'a> {
    fn layer(&self) -> &Layer;
    fn declarations(&self) -> &[Declaration<'a>];
    fn path(&self) -> &'a Path;
}
```

`ParsedFile` implementa `HasWiringPurity` em `parsed_file.rs`
via `contracts/rule_traits.rs`, seguindo o padrão de ADR-0002.

### Verificação

```rust
pub fn check<'a, T: HasWiringPurity<'a>>(
    file: &T,
    config: &WiringConfig,
) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L4 {
        return vec![];
    }

    file.declarations()
        .iter()
        .filter(|d| is_forbidden(d, config))
        .map(|d| Violation {
            rule_id: "V12".to_string(),
            level: ViolationLevel::Warning,
            message: format!(
                "Lógica no fio: {} '{}' declarado em L4. \
                 L4 não cria tipos — mover para L2 ou L3.",
                declaration_kind_str(&d.kind),
                d.name,
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: d.line,
                column: 0,
            },
        })
        .collect()
}

fn is_forbidden(d: &Declaration, config: &WiringConfig) -> bool {
    match d.kind {
        DeclarationKind::Enum => true, // enums nunca pertencem a L4
        DeclarationKind::Struct => !config.allow_adapter_structs,
        DeclarationKind::Impl => true, // impl sem trait = lógica de negócio
    }
}
```

### `WiringConfig` — configuração via `crystalline.toml`

```toml
[wiring_exceptions]
# Structs de adapter são comuns em fases de migração e em projetos
# onde L3 não tem camada separada para adapters de CLI.
# true = permite struct_item em L4 (ainda proíbe enum e impl sem trait)
allow_adapter_structs = true
```

Padrão: `allow_adapter_structs = true` — structs de adapter em L4
são aceitáveis em projetos em migração. `enum_item` e `impl_item`
sem trait são sempre proibidos.

---

## Restrições (L1 Pura)

- Recebe `ParsedFile` via trait `HasWiringPurity` — zero I/O
- `declarations` é populado por L3 (RustParser) — V12 nunca
  acessa a AST diretamente
- `WiringConfig` é injetado por L4 — V12 nunca lê `crystalline.toml`
- Warning por padrão — configurável para Error via `[rules] V12`
- `impl Trait for Type` não é proibido — é o padrão esperado
  de adapter em L4

---

## Critérios de Verificação
```
Dado arquivo L4 com struct_item "L3HashRewriter"
E allow_adapter_structs = false
Quando V12::check() for chamado
Então retorna Violation V12 Warning mencionando "L3HashRewriter"

Dado arquivo L4 com struct_item "L3HashRewriter"
E allow_adapter_structs = true (padrão)
Quando V12::check() for chamado
Então retorna vec![] — adapter struct permitido

Dado arquivo L4 com enum_item "OutputMode"
Quando V12::check() for chamado
Então retorna Violation V12 Warning — enum nunca permitido em L4

Dado arquivo L4 com impl_item "impl L3HashRewriter { ... }"
(impl sem trait)
Quando V12::check() for chamado
Então retorna Violation V12 Warning — impl sem trait é lógica de negócio

Dado arquivo L4 com impl_item "impl HashRewriter for L3HashRewriter { ... }"
(impl com trait)
Quando V12::check() for chamado
Então retorna vec![] — impl de trait é o padrão de adapter

Dado arquivo L3 com qualquer struct_item
Quando V12::check() for chamado
Então retorna vec![] — V12 só se aplica a L4

Dado arquivo L4 sem nenhuma declaração proibida
Quando V12::check() for chamado
Então retorna vec![]
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-16 | Criação inicial (ADR-0007) | wiring_logic_leak.rs |
| 2026-03-16 | Materialização: check() com is_forbidden/declaration_kind_str, 8 testes; módulo registado em rules/mod.rs; EnabledChecks v10/v11/v12 em cli.rs; WiringExceptionsConfig em config.rs; run_checks/run_pipeline actualizados em main.rs; V10/V11 V12 wired; crystalline.toml actualizado | wiring_logic_leak.rs, cli.rs, config.rs, main.rs, crystalline.toml |
