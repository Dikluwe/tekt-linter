# ⚖️ ADR-0014: V11 Configurável via `[rules]`

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-23

---

## Contexto

V7, V11 e V12 são regras globais — operam sobre `ProjectIndex` após o
pipeline Map-Reduce, na fase sequencial de `main.rs`:

```rust
if enabled.v7  { all_violations.extend(check_orphans(&project_index, &all_prompts)); }
if enabled.v8  { all_violations.extend(check_aliens(&project_index)); }
if enabled.v11 { all_violations.extend(check_dangling_contracts(&project_index)); }
```

As regras por ficheiro (V1–V6, V9, V10, V12, V13, V14) recebem o nível
efectivo derivado de `config.rules` via `run_checks` — o mecanismo existe
e funciona. As regras globais (`check_orphans`, `check_dangling_contracts`)
não recebem esse parâmetro: produzem `Violation` com `ViolationLevel`
hardcoded, ignorando completamente a secção `[rules]` do
`crystalline.toml`.

O sintoma foi observado num projecto que usa a arquitectura cristalina:
`V11 = { level = "warning" }` no `[rules]` não tem efeito — o linter
retorna exit 1 enquanto contratos L1 não tiverem implementação em L3,
independentemente da configuração. O único workaround disponível é
`--checks v1,v2,...` excluindo V11 explicitamente, o que é frágil e
não expressa a intenção correcta.

### Causa raiz

`check_dangling_contracts` e `check_orphans` constroem `Violation` com
nível hardcoded:

```rust
// dangling_contract.rs — nivel hardcoded
Violation {
    rule_id: "V11".to_string(),
    level: ViolationLevel::Error,  // ← não lê config
    ...
}

// orphan_prompt.rs — nivel hardcoded
Violation {
    rule_id: "V7".to_string(),
    level: ViolationLevel::Warning,  // ← não lê config
    ...
}
```

O mecanismo de resolução de nível existe para as regras por ficheiro mas
nunca foi propagado para as regras globais.

### Escopo do problema

V7 tem o mesmo defeito mas o seu nível por defeito (`Warning`) coincide
com o valor mais comum na prática — o bug é menos visível. V11 tem nível
por defeito `Error`, que bloqueia CI, tornando o defeito imediatamente
observável quando o projecto tem contratos em desenvolvimento (blanket
impl em L1, implementação em L3 ainda por escrever).

V8 (`Fatal`) e V0 (`Fatal`) são intencionalmente não configuráveis —
a sua gravidade é invariante por design da arquitectura. Não são
afectados por este ADR.

---

## Decisão

As funções globais `check_dangling_contracts` e `check_orphans` passam a
receber o nível efectivo como parâmetro, derivado de `config.rules` em L4
antes de chamar as funções.

### Assinaturas alteradas

```rust
// dangling_contract.rs
pub fn check_dangling_contracts<'a>(
    index: &'a ProjectIndex<'a>,
    level: ViolationLevel,
) -> Vec<Violation<'a>>

// orphan_prompt.rs
pub fn check_orphans<'a>(
    index: &'a ProjectIndex<'a>,
    all_prompts: &'a AllPrompts,
    level: ViolationLevel,
) -> Vec<Violation<'a>>
```

O `level` substituí o literal hardcoded na construção de cada `Violation`.

### Resolução do nível em L4

```rust
// main.rs — antes das verificações globais
let v7_level  = config.rules.level_for("V7",  ViolationLevel::Warning);
let v11_level = config.rules.level_for("V11", ViolationLevel::Error);

if enabled.v7  { all_violations.extend(check_orphans(&project_index, &all_prompts, v7_level)); }
if enabled.v11 { all_violations.extend(check_dangling_contracts(&project_index, v11_level)); }
```

`level_for(rule_id, default)` já existe em `CrystallineConfig` (ou
equivalente) — é o mesmo mecanismo usado pelas regras por ficheiro em
`run_checks`. Se não existir como método público, extrair de `run_checks`
para uma função utilitária em L4 ou expor via `config.rules`.

### Defaults preservados

O comportamento observável sem `[rules]` no `crystalline.toml` é idêntico
ao estado actual:

| Regra | Nível por defeito |
|-------|------------------|
| V7    | `Warning`        |
| V11   | `Error`          |

A mudança é apenas: quando `[rules]` declara um nível diferente, esse
nível é respeitado.

### V8, V0, V10 — não afectados

Regras com nível `Fatal` por design arquitectural não são parametrizadas.
A sua invariância é intencional e documentada — configurar um `Fatal` para
`Warning` seria contradição com a semântica da regra. `check_aliens` não
recebe parâmetro de nível.

---

## Impacto nos ficheiros

| Ficheiro | Natureza da mudança |
|----------|---------------------|
| `01_core/rules/dangling_contract.rs` | Assinatura: `+ level: ViolationLevel` |
| `01_core/rules/orphan_prompt.rs` | Assinatura: `+ level: ViolationLevel` |
| `04_wiring/main.rs` | Resolução de `v7_level` e `v11_level`; chamadas actualizadas |
| `00_nucleo/prompts/rules/dangling-contract.md` | Assinatura documentada |
| `00_nucleo/prompts/rules/orphan-prompt.md` | Assinatura documentada |
| `00_nucleo/prompts/linter-core.md` | Pipeline global actualizado |

Nenhuma mudança na IR (`ParsedFile`, `ProjectIndex`, `Violation`).
Nenhuma mudança nos parsers. Nenhuma mudança no `crystalline.toml`.

---

## Consequências

### ✅ Positivas

- V11 respeita `[rules]` como todas as outras regras configuráveis
- V7 também corrigido — consistência total no mecanismo de configuração
- Projectos em desenvolvimento podem declarar `V11 = { level = "warning" }`
  durante a fase de implementação de contratos e promover para `error`
  quando L3 estiver completo
- Zero impacto na IR — mudança cirúrgica, apenas nas assinaturas e no wiring

### ❌ Negativas

- Nenhuma relevante — a mudança é conservadora: os defaults são preservados

### ⚙️ Neutras

- Testes existentes de `check_dangling_contracts` e `check_orphans` precisam
  de passar o nível explicitamente — actualização mecânica
- O número de testes não aumenta obrigatoriamente — os existentes podem
  passar `ViolationLevel::Error` / `ViolationLevel::Warning` como antes

---

## Alternativas Consideradas

| Alternativa | Decisão |
|-------------|---------|
| Passar `&CrystallineConfig` completo para as funções globais | Rejeitada — L1 não lê config directamente; viola a restrição de L1 |
| Criar `GlobalRuleConfig` struct injectada em L4 | Desnecessário — o nível é o único parâmetro relevante; scalar é suficiente |
| Resolver nível dentro de `check_dangling_contracts` via trait | Rejeitada — L1 não conhece config; a injecção de fora é a postura correcta |

---

## Referências

- ADR-0007: V10–V12, origem de `check_dangling_contracts`
- ADR-0006: V7–V9, origem de `check_orphans`
- `linter-core.md` — pipeline Map-Reduce e fase global
