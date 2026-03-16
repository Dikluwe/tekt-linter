# Prompt: Rule Traits (rule-traits)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2026-03-15 (ADR-0006 refactor)
**Arquivos gerados**:
  - 01_core/contracts/rule_traits.rs

---

## Contexto

Cada regra V1–V9 precisa de um subconjunto específico dos campos
de `ParsedFile<'a>` para operar. O Claude Code implementou esse
acesso via traits locais em cada arquivo de regra, com
`ParsedFile` implementando todas em `parsed_file.rs`.

O problema: `parsed_file.rs` pertence a `entities/` e passou a
importar de `rules/` — inversão de dependência dentro de L1.
Entities não deve conhecer rules.

A correção move todas as traits de acesso para `contracts/` —
o lugar correto para contratos que definem como entidades expõem
seus dados. `parsed_file.rs` implementa de `contracts/`,
cada regra importa de `contracts/`. A direção é correta:
```
rules/ → contracts/ → entities/    ✅ correto
entities/ → rules/                 ❌ inversão
```

---

## Traits
```rust
use std::path::Path;
use crate::entities::layer::Layer;
use crate::entities::parsed_file::{
    Import, Token, PromptHeader, PublicInterface,
};

/// Para V1 — verifica presença e validade do @prompt header
pub trait HasPromptFilesystem<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn prompt_file_exists(&self) -> bool;
    fn path(&self) -> &'a Path;
}

/// Para V2 — verifica cobertura de testes em L1
pub trait HasCoverage<'a> {
    fn layer(&self) -> &Layer;
    fn has_test_coverage(&self) -> bool;
    fn path(&self) -> &'a Path;
}

/// Para V3 — verifica imports proibidos por camada
pub trait HasImports<'a> {
    fn layer(&self) -> &Layer;
    fn imports(&self) -> &[Import<'a>];
    fn path(&self) -> &'a Path;
}

/// Para V4 — verifica tokens de I/O em L1
pub trait HasTokens<'a> {
    fn layer(&self) -> &Layer;
    fn tokens(&self) -> &[Token<'a>];
    fn path(&self) -> &'a Path;
}

/// Para V5 — verifica drift de hash entre prompt e código
pub trait HasHashes<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn path(&self) -> &'a Path;
}

/// Para V6 — verifica drift de interface pública
pub trait HasPublicInterface<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>>;
    fn public_interface(&self) -> &PublicInterface<'a>;
    fn prompt_snapshot(&self) -> Option<&PublicInterface<'a>>;
    fn path(&self) -> &'a Path;
}

/// Para V9 — verifica imports de subdiretórios não-porta de L1
pub trait HasPubLeak<'a> {
    fn layer(&self) -> &Layer;
    fn imports(&self) -> &[Import<'a>];
    fn path(&self) -> &'a Path;
}
```

---

## Implementações em `parsed_file.rs`

`ParsedFile<'a>` implementa todas as traits acima.
As implementações vivem em `parsed_file.rs` — não num arquivo
separado — porque são triviais (um campo por método) e manter
junto evita fragmentação desnecessária.

Padrão de cada impl:
```rust
// Em 01_core/entities/parsed_file.rs
use crate::contracts::rule_traits::{
    HasPromptFilesystem, HasCoverage, HasImports,
    HasTokens, HasHashes, HasPublicInterface, HasPubLeak,
};

impl<'a> HasPromptFilesystem<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn prompt_file_exists(&self) -> bool { self.prompt_file_exists }
    fn path(&self) -> &'a Path { self.path }
}

// ... demais impls análogas
```

---

## Impacto em cada arquivo de regra

Cada regra remove a definição local da trait e passa a importar
de `contracts::rule_traits`:
```rust
// ANTES (em rules/pub_leak.rs):
pub trait HasPubLeak<'a> { ... }  // ← remover

// DEPOIS:
use crate::contracts::rule_traits::HasPubLeak;  // ← importar
```

A assinatura de `check()` não muda — apenas a origem da trait.

---

## Restrições

- `rule_traits.rs` importa apenas de `entities/` — sem imports
  de `rules/` ou de qualquer outro módulo de `contracts/`
- As traits são somente leitura — nenhum método mutável
- `parsed_file.rs` importa de `contracts/rule_traits` —
  nunca de `rules/`
- Cada regra importa apenas a trait que usa — não o módulo inteiro

---

## Critérios de Verificação
```
Dado parsed_file.rs
Quando inspecionado por imports
Então não contém nenhum import de crate::rules::*
— inversão de dependência eliminada

Dado rules/pub_leak.rs
Quando inspecionado
Então HasPubLeak não está definida localmente
E importa de crate::contracts::rule_traits::HasPubLeak

Dado MockFile implementando HasPubLeak em teste de pub_leak.rs
Quando check() for chamado com MockFile
Então funciona identicamente ao ParsedFile
— testabilidade preservada

Dado rule_traits.rs
Quando inspecionado por imports
Então importa apenas de crate::entities::*
— sem imports de rules/ ou outros contracts/
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-15 | Criação (ADR-0006 refactor): traits movidas de rules/ para contracts/ para corrigir inversão entities→rules em parsed_file.rs | rule_traits.rs |

---


