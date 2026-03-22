# Prompt: Rule V14 — External Type In Contract (external-type-in-contract)

**Camada**: L1 (Core — Rules)
**Regra**: V14
**Criado em**: 2026-03-20 (ADR-0012)
**Arquivos gerados**:
  - 01_core/rules/external_type_in_contract.rs + test

---

## Contexto

V3 proíbe imports de camadas internas na direcção errada. V4
proíbe símbolos de I/O conhecidos. Nenhuma das duas protege L1
de dependências externas arbitrárias que não são I/O.

Listas de bloqueio não funcionam para este problema — o
ecossistema cresce mais rápido que qualquer lista. A solução
correcta é inverter a lógica: L1 é **fechada por defeito** para
o mundo exterior. Apenas pacotes explicitamente autorizados em
`[l1_allowed_external]` são permitidos.

---

## Especificação

V14 opera sobre `ParsedFile.imports` por arquivo, na fase Map.
Aplica-se apenas a arquivos com `layer == L1`.

Para cada import em L1 com `target_layer == Layer::Unknown`:
1. Extrair o nome do pacote (primeiro segmento do path)
2. Verificar isenções de stdlib
3. Verificar contra a whitelist `L1AllowedExternal`
4. Se não autorizado → Violation V14 Error

### Nova struct — `L1AllowedExternal`

```rust
/// Whitelist de pacotes externos permitidos em L1.
/// Construída de config.l1_allowed_external em L4.
/// Injectada em V14 via parâmetro — L1 nunca lê o toml.
pub struct L1AllowedExternal {
    /// Nomes de pacotes autorizados para a linguagem em análise.
    allowed: HashSet<String>,
    /// Prefixos sempre isentos (stdlib) — nunca verificados contra whitelist.
    /// Rust: ["std", "core", "alloc"]
    /// TypeScript: [] (distinção stdlib já feita pelo parser)
    /// Python: [] (distinção stdlib já feita pelo parser)
    exempt_prefixes: Vec<String>,
}

impl L1AllowedExternal {
    pub fn for_rust(allowed: HashSet<String>) -> Self {
        Self {
            allowed,
            exempt_prefixes: vec![
                "std".to_string(),
                "core".to_string(),
                "alloc".to_string(),
            ],
        }
    }

    pub fn is_allowed(&self, package_name: &str) -> bool {
        if self.exempt_prefixes.iter().any(|p| package_name == p) {
            return true;
        }
        self.allowed.contains(package_name)
    }
}
```

### Extracção do nome do pacote

```rust
fn package_name(import_path: &str) -> &str {
    // Rust: "serde::Serialize" → "serde"
    //       "std::collections::HashMap" → "std" (isento)
    // TypeScript e Python: o path já é o nome do pacote
    import_path.split("::").next()
        .unwrap_or(import_path)
        .split('/')  // para scoped npm packages: "@scope/pkg"
        .next()
        .unwrap_or(import_path)
}
```

### Verificação

```rust
pub fn check<'a, T: HasImports<'a>>(
    file: &T,
    allowed: &L1AllowedExternal,
) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| import.target_layer == Layer::Unknown)
        .filter(|import| !allowed.is_allowed(package_name(import.path)))
        .map(|import| Violation {
            rule_id: "V14".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Dependência externa não autorizada em L1: '{}' não está em \
                 [l1_allowed_external]. Adicionar ao crystalline.toml se necessário, \
                 ou mover a dependência para L3.",
                package_name(import.path),
            ),
            location: Location {
                path: Cow::Borrowed(file.path()),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}
```

---

## Configuração em `crystalline.toml`

```toml
[l1_allowed_external]
# Pacotes externos explicitamente autorizados em L1.
# L1 é fechada por defeito — qualquer externo não listado é Error.
# Manter esta lista pequena: cada entrada é uma dependência de domínio.
rust       = ["thiserror"]
typescript = []
python     = []
```

**Semântica da lista vazia:** TypeScript e Python com lista vazia
significa que nenhum pacote externo é permitido em L1. O parser
já exclui a stdlib antes de chegar a V14 (ADR-0009).

**Projectos sem a secção:** se `[l1_allowed_external]` não existir
no toml, o comportamento padrão é lista vazia para todas as
linguagens — qualquer externo em L1 é Error. Adicionar a secção
ao toml é o gesto explícito de autorização.

---

## Restrições (L1 Pura)

- Opera sobre `HasImports` — zero I/O
- `L1AllowedExternal` é injectada por L4 — V14 nunca lê o toml
- Aplica-se apenas a `layer == L1`
- `target_layer == Layer::Unknown` é o critério — o parser já
  fez a distinção entre externo e interno
- Stdlib isenta por prefixo (`std`, `core`, `alloc` para Rust)
- A lista de isenções de stdlib é constante por linguagem —
  não configurável por projecto

---

## Critérios de Verificação

```
Dado arquivo L1 com:
  use comemo::Tracked;
E [l1_allowed_external] rust = ["thiserror"]
Quando V14::check() for chamado
Então retorna Violation { rule_id: "V14", level: Error }
— comemo não está na whitelist

Dado arquivo L1 com:
  use thiserror::Error;
E [l1_allowed_external] rust = ["thiserror"]
Quando V14::check() for chamado
Então retorna vec![]
— thiserror está na whitelist

Dado arquivo L1 com:
  use std::collections::HashMap;
E whitelist vazia
Quando V14::check() for chamado
Então retorna vec![]
— std é isento (stdlib Rust)

Dado arquivo L1 com:
  use core::fmt::Display;
E whitelist vazia
Quando V14::check() for chamado
Então retorna vec![]
— core é isento (stdlib Rust)

Dado arquivo L1 com:
  use tokio::sync::Mutex;
E [l1_allowed_external] rust = ["thiserror"]
Quando V14::check() for chamado
Então retorna Violation { rule_id: "V14", level: Error }
— tokio não está na whitelist
— (V4 também poderia disparar para tokio::sync::Mutex se for usado)

Dado arquivo L1 com:
  use serde::Serialize;
E [l1_allowed_external] rust = []  (lista vazia)
Quando V14::check() for chamado
Então retorna Violation { rule_id: "V14", level: Error }
— serde não está na whitelist vazia

Dado arquivo L1 com:
  use serde::Serialize;
E [l1_allowed_external] rust = ["serde", "thiserror"]
Quando V14::check() for chamado
Então retorna vec![]
— serde explicitamente autorizado

Dado arquivo L3 com:
  use rayon::prelude::*;
E whitelist qualquer
Quando V14::check() for chamado
Então retorna vec![]
— V14 aplica-se apenas a L1

Dado arquivo L1 sem imports externos
E whitelist qualquer
Quando V14::check() for chamado
Então retorna vec![]

Dado arquivo L1 com dois imports externos não autorizados
Quando V14::check() for chamado
Então retorna duas violations — uma por import
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-20 | Criação inicial (ADR-0012) | external_type_in_contract.rs |
