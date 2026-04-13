# Prompt: Rule Traits (rule-traits)
Hash do Código: 5f315695

**Camada**: L1 (Core — Contracts)
**Criado em**: 2026-03-15 (ADR-0006 refactor)
**Revisado em**: 2026-03-18 (ADR-0009 correcção: HasTokens ganha language() para V4 multi-linguagem)
**Arquivos gerados**:
  - 01_core/entities/rule_traits.rs

---

## Contexto

Cada regra V1–V12 precisa de um subconjunto específico dos campos
de `ParsedFile<'a>` para operar. O Claude Code implementou esse
acesso via traits locais em cada arquivo de regra, com
`ParsedFile` implementando todas em `parsed_file.rs`.

O problema: `parsed_file.rs` pertence a `entities/` e passou a
importar de `rules/` — inversão de dependência dentro de L1.
Entities não deve conhecer rules.

A correção move todas as traits de acesso para `entities/` —
o lugar correcto para contratos que definem como entidades expõem
os seus dados. `parsed_file.rs` implementa de `entities/rule_traits`,
cada regra importa de `entities/rule_traits`. A direção é correcta:
```
rules/ → entities/rule_traits → entities/parsed_file    ✅ correcto
entities/parsed_file → rules/                           ❌ inversão
```

**Nota sobre localização:** Este ficheiro vive em
`01_core/entities/rule_traits.rs` desde o ADR-0007, que moveu
as traits de acesso de `contracts/` para `entities/`. `contracts/`
contém exclusivamente portas de infraestrutura — traits cujo
`impl` pertence a L2 ou L3.

---

## Traits

```rust
use std::path::Path;
use crate::entities::layer::{Language, Layer};
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
///
/// V3 usa target_layer para detectar inversão de dependência.
/// V9 usa target_subdir para detectar imports fora das portas de L1.
/// V10 usa target_layer == Layer::Lab para detectar quarantine leak.
/// Nenhuma das três usa ImportKind — ImportKind descreve mecânica
/// estrutural, não linguagem. As regras são agnósticas de linguagem.
pub trait HasImports<'a> {
    fn layer(&self) -> &Layer;
    fn imports(&self) -> &[Import<'a>];
    fn path(&self) -> &'a Path;
}

/// Para V4 — verifica tokens de I/O em L1
///
/// language() é necessário para que V4 seleccione a lista de símbolos
/// proibidos correspondente à linguagem do arquivo.
/// V4 nunca usa ImportKind para distinguir linguagens — usa language().
pub trait HasTokens<'a> {
    fn layer(&self) -> &Layer;
    fn language(&self) -> &Language;
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
/// `declarations()` expõe struct/enum/impl-sem-trait/class/interface/
/// type-alias de nível superior.
/// V12 filtra por `layer() == Layer::L4` internamente.
/// `impl Trait for Type` e `class implements` não aparecem em
/// `declarations()` — são adapters permitidos em L4.
pub trait HasWiringPurity<'a> {
    fn layer(&self) -> &Layer;
    fn declarations(&self) -> &[Declaration<'a>];
    fn path(&self) -> &'a Path;
}
```

**Nota sobre V3, V9 e V10:** as três regras consomem `HasImports`.
V3 verifica `target_layer`, V9 verifica `target_subdir`, V10
verifica `target_layer == Layer::Lab`. A trait é a mesma — as
regras diferem no predicado de filtragem.

**Nota sobre V4 e `language()`:** `HasTokens` expõe `language()`
para que V4 possa seleccionar a lista de símbolos proibidos
correspondente sem conhecer a sintaxe de nenhuma linguagem.
`forbidden_symbols_for(language)` vive em `impure_core.rs` —
não na IR.

---

## Implementações em `parsed_file.rs`

`ParsedFile<'a>` implementa todas as traits acima.
As implementações vivem em `parsed_file.rs` — não num arquivo
separado — porque são triviais (um campo por método).

```rust
// Em 01_core/entities/parsed_file.rs
use crate::entities::rule_traits::{
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

impl<'a> HasCoverage<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn has_test_coverage(&self) -> bool { self.has_test_coverage }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasImports<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn imports(&self) -> &[Import<'a>] { &self.imports }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasTokens<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn language(&self) -> &Language { &self.language }  // novo — para V4
    fn tokens(&self) -> &[Token<'a>] { &self.tokens }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasHashes<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasPublicInterface<'a> for ParsedFile<'a> {
    fn prompt_header(&self) -> Option<&PromptHeader<'a>> {
        self.prompt_header.as_ref()
    }
    fn public_interface(&self) -> &PublicInterface<'a> { &self.public_interface }
    fn prompt_snapshot(&self) -> Option<&PublicInterface<'a>> {
        self.prompt_snapshot.as_ref()
    }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasPubLeak<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn imports(&self) -> &[Import<'a>] { &self.imports }
    fn path(&self) -> &'a Path { self.path }
}

impl<'a> HasWiringPurity<'a> for ParsedFile<'a> {
    fn layer(&self) -> &Layer { &self.layer }
    fn declarations(&self) -> &[Declaration<'a>] { &self.declarations }
    fn path(&self) -> &'a Path { self.path }
}
```

---

## Impacto em cada arquivo de regra

Regras existentes (V1–V3, V5–V12) não mudam — apenas V4 é
afectada pela adição de `language()` a `HasTokens`:

```rust
// Em rules/impure_core.rs — usa language() para seleccionar lista
pub fn check<'a, T: HasTokens<'a>>(file: &T) -> Vec<Violation<'a>> {
    if *file.layer() != Layer::L1 { return vec![]; }
    let forbidden = forbidden_symbols_for(file.language());  // novo
    file.tokens()
        .iter()
        .filter(|t| is_forbidden_symbol(&t.symbol, forbidden))
        .map(|t| make_violation(file, t))
        .collect()
}
```

Mocks de teste de V4 precisam de implementar `language()`:
```rust
// Antes — mock sem language()
struct MockFile { layer: Layer, tokens: Vec<Token<'static>>, path: &'static Path }

// Depois — mock com language()
struct MockFile {
    layer: Layer,
    language: Language,  // novo campo
    tokens: Vec<Token<'static>>,
    path: &'static Path,
}
impl HasTokens<'static> for MockFile {
    fn layer(&self) -> &Layer { &self.layer }
    fn language(&self) -> &Language { &self.language }  // novo
    fn tokens(&self) -> &[Token<'static>] { &self.tokens }
    fn path(&self) -> &'static Path { self.path }
}
```

---

## Restrições

- `rule_traits.rs` importa apenas de `entities/` — sem imports
  de `rules/` ou de `contracts/`
- As traits são somente leitura — nenhum método mutável
- `parsed_file.rs` importa de `entities::rule_traits` —
  nunca de `rules/`
- Cada regra importa apenas a trait que usa — não o módulo inteiro
- `HasTokens.language()` é o único canal pelo qual V4 conhece
  a linguagem — nunca via `ImportKind` ou condicionais na regra
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
E importa de crate::entities::rule_traits::HasPubLeak

Dado MockFile implementando HasPubLeak em teste de pub_leak.rs
Quando check() for chamado com MockFile
Então funciona identicamente ao ParsedFile
— testabilidade preservada

Dado rule_traits.rs
Quando inspecionado por imports
Então importa apenas de crate::entities::*
— sem imports de rules/ ou contracts/

Dado rule_traits.rs
Quando inspecionado
Então contém exactamente as traits:
  HasPromptFilesystem, HasCoverage, HasImports, HasTokens,
  HasHashes, HasPublicInterface, HasPubLeak, HasWiringPurity
— sem traits locais remanescentes em arquivos de regra

Dado MockFile implementando HasTokens
Com layer = L1, language = Rust
E tokens = [Token { symbol: "std::fs::read", .. }]
Quando impure_core::check() for chamado
Então retorna Violation V4
— language() exposto via HasTokens, lista Rust seleccionada

Dado MockFile implementando HasTokens
Com layer = L1, language = Python
E tokens = [Token { symbol: "os.path.join", .. }]
Quando impure_core::check() for chamado
Então retorna Violation V4
— language() exposto via HasTokens, lista Python seleccionada

Dado MockFile implementando HasTokens
Com layer = L1, language = TypeScript
E tokens = [Token { symbol: "Date.now", .. }]
Quando impure_core::check() for chamado
Então retorna Violation V4
— language() exposto via HasTokens, lista TypeScript seleccionada

Dado MockFile implementando HasTokens com language = Unknown
Quando impure_core::check() for chamado
Então retorna vec[]
— lista vazia para Unknown, zero violações V4

Dado MockFile implementando HasWiringPurity com layer = L4
E declarations = [Declaration { kind: Enum, name: "Mode", line: 3 }]
Quando wiring_logic_leak::check() for chamado com WiringConfig::default()
Então retorna Violation V12 Warning

Dado MockFile implementando HasImports com layer = L1
E imports = [Import { target_layer: Layer::Lab, line: 7, .. }]
Quando quarantine_leak::check() for chamado
Então retorna Violation V10 Fatal
— HasImports reutilizada para V10, filtragem por target_layer

Dado MockFile implementando HasImports
Com Import { kind: ImportKind::Named, target_layer: Layer::L3 }
em arquivo com layer = L2
Quando forbidden_import::check() for chamado
Então retorna Violation V3
— V3 usa target_layer, nunca ImportKind
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-15 | Criação (ADR-0006 refactor): traits movidas de rules/ para contracts/ para corrigir inversão entities→rules em parsed_file.rs | rule_traits.rs |
| 2026-03-16 | ADR-0007: HasWiringPurity adicionada para V12; nota sobre V3/V9/V10 partilhando HasImports | rule_traits.rs |
| 2026-03-16 | ADR-0007 fecho: rule_traits.rs movido de contracts/ para entities/ | rule_traits.rs |
| 2026-03-18 | ADR-0009 correcção: HasTokens ganha language() para V4 multi-linguagem; nota sobre V4 nunca usar ImportKind; mock de V4 actualizado com campo language; critérios de Rust/Python/TypeScript/Unknown adicionados; critério V3 com ImportKind::Named documenta agnósticidade | rule_traits.rs, impure_core.rs |
