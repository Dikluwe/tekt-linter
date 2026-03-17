# Prompt: Rule Traits (rule-traits)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2026-03-15 (ADR-0006 refactor)
**Revisado em**: 2026-03-16 (ADR-0007: HasWiringPurity para V12)
**Arquivos gerados**:
  - 01_core/contracts/rule_traits.rs

---

## Contexto

Cada regra V1–V12 precisa de um subconjunto específico dos campos
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
    Declaration, Import, PromptHeader, PublicInterface, Token,
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

/// Para V3, V9 e V10 — verifica imports por camada e subdiretório
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

/// Para V12 — verifica declarações de tipo em L4
///
/// `declarations()` expõe struct/enum/impl-sem-trait de nível superior.
/// V12 filtra por `layer() == Layer::L4` internamente.
/// `impl Trait for Type` não aparece em `declarations()` —
/// o RustParser só captura `impl Type { ... }` sem trait.
pub trait HasWiringPurity<'a> {
    fn layer(&self) -> &Layer;
    fn declarations(&self) -> &[Declaration<'a>];
    fn path(&self) -> &'a Path;
}
```

**Nota sobre V3, V9 e V10:** as três regras consomem
`HasImports`. V3 verifica a direção do import via `target_layer`,
V9 verifica o subdiretório de L1 via `target_subdir`, V10 verifica
se `target_layer == Layer::Lab`. A trait é a mesma — as regras
diferem no predicado de filtragem.

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
    HasTokens, HasHashes, HasPublicInterface,
    HasPubLeak, HasWiringPurity,
};

impl<'a> HasPromptFilesystem<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn prompt_file_exists(&self) -> bool { self.prompt_file_exists }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasWiringPurity<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn declarations(&self) -> &[Declaration<'a>] { &self.declarations }
    fn path(&self) -> &'a Path { self.path }
}

// ... demais impls análogas
```

---

## Impacto em cada arquivo de regra

Regras existentes (V1–V9) não mudam — apenas V12 é nova:
```rust
// Em rules/wiring_logic_leak.rs:
use crate::contracts::rule_traits::HasWiringPurity;
use crate::entities::violation::WiringConfig;

pub fn check<'a, T: HasWiringPurity<'a>>(
    file: &T,
    config: &WiringConfig,
) -> Vec<Violation<'a>> { ... }
```

V10 e V11 reutilizam traits existentes:
- V10 usa `HasImports` — filtra por `import.target_layer == Layer::Lab`
- V11 opera sobre `ProjectIndex` diretamente — não usa trait de `ParsedFile`

---

## Restrições

- `rule_traits.rs` importa apenas de `entities/` — sem imports
  de `rules/` ou de qualquer outro módulo de `contracts/`
- As traits são somente leitura — nenhum método mutável
- `parsed_file.rs` importa de `contracts/rule_traits` —
  nunca de `rules/`
- Cada regra importa apenas a trait que usa — não o módulo inteiro
- `HasWiringPurity.declarations()` retorna apenas declarações
  de nível superior — não itens aninhados em funções ou blocos

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

Dado MockFile implementando HasWiringPurity com layer = L4
E declarations = [Declaration { kind: Enum, name: "Mode", line: 3 }]
Quando wiring_logic_leak::check() for chamado com WiringConfig::default()
Então retorna Violation V12 Warning mencionando "Mode"
— enum nunca permitido em L4

Dado MockFile implementando HasWiringPurity com layer = L4
E declarations = [Declaration { kind: Struct, name: "Adapter", line: 5 }]
Quando wiring_logic_leak::check() for chamado
E WiringConfig { allow_adapter_structs: true }
Então retorna vec![] — struct de adapter permitida

Dado MockFile implementando HasWiringPurity com layer = L3
E declarations = [Declaration { kind: Struct, name: "Walker", line: 1 }]
Quando wiring_logic_leak::check() for chamado
Então retorna vec![] — V12 só se aplica a L4

Dado MockFile implementando HasImports com layer = L1
E imports = [Import { target_layer: Layer::Lab, line: 7, .. }]
Quando quarantine_leak::check() for chamado
Então retorna Violation V10 Fatal com location.line: 7
— HasImports reutilizada para V10, filtragem por target_layer

Dado rule_traits.rs
Quando inspecionado
Então contém exatamente as traits:
  HasPromptFilesystem, HasCoverage, HasImports, HasTokens,
  HasHashes, HasPublicInterface, HasPubLeak, HasWiringPurity
— sem traits locais remanescentes em arquivos de regra
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-15 | Criação (ADR-0006 refactor): traits movidas de rules/ para contracts/ para corrigir inversão entities→rules em parsed_file.rs | rule_traits.rs |
| 2026-03-16 | ADR-0007: HasWiringPurity adicionada para V12 (layer, declarations, path); nota sobre V3/V9/V10 compartilhando HasImports; impacto em regras documentado; critérios de HasWiringPurity adicionados | rule_traits.rs |
| 2026-03-16 | Materialização ADR-0007: HasWiringPurity trait implementada em rule_traits.rs; Declaration importada de parsed_file; MockV12 e dois testes adicionados ao #[cfg(test)] | rule_traits.rs |
