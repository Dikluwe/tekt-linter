# Prompt: Types of Violations (violation-types)

**Camada**: L1 (Core - Entities)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

Este módulo define as entidades fundamentais do linter: `ParsedFile`,
`Violation`, `Layer` e tipos auxiliares. Formam a Representação
Intermediária (IR) sobre a qual todas as regras V1–V6 operam de forma
pura e agnóstica a linguagem e filesystem.

L3 é responsável por popular todos os campos — incluindo os booleanos
derivados de filesystem, os `target_layer` de cada import, e os
snapshots de interface pública para V6. L1 nunca deriva esses valores
por conta própria.

---

## Estruturas de Dados

### `ParsedFile`
```rust
pub struct ParsedFile {
    pub path: PathBuf,
    pub layer: Layer,
    pub language: Language,

    // Base IR (AST Features)
    pub prompt_header: Option<PromptHeader>,
    pub prompt_file_exists: bool,

    // Para V2
    pub has_test_coverage: bool,

    // Para V3
    pub imports: Vec<Import>,
    
    // Para V4
    pub tokens: Vec<Token>,

    // Para V6
    pub public_interface: PublicInterface,
    // Interface pública extraída do AST pelo RustParser (L3)

    pub prompt_snapshot: Option<PublicInterface>,
    // Snapshot da interface registrada no prompt em L0
    // None se o prompt não contém seção ## Interface Snapshot
    // Populado por L3 via PromptSnapshotReader
}
```

### `PublicInterface` (novo — para V6)
```rust
/// Interface pública extraída do AST — agnóstica de linguagem.
/// Não inclui implementação, apenas contratos visíveis externamente.
/// Dois ParsedFiles com PublicInterface idênticas são estruturalmente
/// equivalentes do ponto de vista do contrato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicInterface {
    pub functions: Vec<FunctionSignature>,
    pub types: Vec<TypeSignature>,
    pub reexports: Vec<String>,
}

impl PublicInterface {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.types.is_empty()
            && self.reexports.is_empty()
    }
}
```

### `FunctionSignature` (novo — para V6)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub name: String,
    /// Tipos dos parâmetros como strings normalizadas
    pub params: Vec<String>,
    /// None para funções que retornam ()
    pub return_type: Option<String>,
}
```

### `TypeSignature` (novo — para V6)
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature {
    pub name: String,
    pub kind: TypeKind,
    /// Campos de struct, variantes de enum, métodos de trait
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
}
```

### `InterfaceDelta` (novo — para V6)
```rust
/// Diferença entre interface atual e snapshot do prompt.
/// Produzida por compute_delta() em L1 — função pura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDelta {
    pub added_functions: Vec<FunctionSignature>,
    pub removed_functions: Vec<FunctionSignature>,
    pub added_types: Vec<TypeSignature>,
    pub removed_types: Vec<TypeSignature>,
    pub added_reexports: Vec<String>,
    pub removed_reexports: Vec<String>,
}

impl InterfaceDelta {
    pub fn is_empty(&self) -> bool {
        self.added_functions.is_empty()
            && self.removed_functions.is_empty()
            && self.added_types.is_empty()
            && self.removed_types.is_empty()
            && self.added_reexports.is_empty()
            && self.removed_reexports.is_empty()
    }

    pub fn describe(&self) -> String {
        // Produz string legível para mensagem de violação.
        // Exemplo: "+fn check, -fn validate, +struct Delta"
        // Ordem: adições antes de remoções, funções antes de tipos.
    }
}

/// Computa o delta entre interface atual e snapshot.
/// Função pura — zero I/O, zero tree-sitter.
pub fn compute_delta(
    current: &PublicInterface,
    snapshot: &PublicInterface,
) -> InterfaceDelta {
    InterfaceDelta {
        added_functions: current.functions.iter()
            .filter(|f| !snapshot.functions.contains(f))
            .cloned().collect(),
        removed_functions: snapshot.functions.iter()
            .filter(|f| !current.functions.contains(f))
            .cloned().collect(),
        added_types: current.types.iter()
            .filter(|t| !snapshot.types.contains(t))
            .cloned().collect(),
        removed_types: snapshot.types.iter()
            .filter(|t| !current.types.contains(t))
            .cloned().collect(),
        added_reexports: current.reexports.iter()
            .filter(|r| !snapshot.reexports.contains(r))
            .cloned().collect(),
        removed_reexports: snapshot.reexports.iter()
            .filter(|r| !current.reexports.contains(r))
            .cloned().collect(),
    }
}
```

---

## Responsabilidades de População (L3)

L3 processa e popula os campos abstratos de `ParsedFile`. As regras (L1) não consomem `ParsedFile` de forma acoplada, mas sim através de abstrações (Traits como `HasPromptFilesystem`, `HasCoverage`, etc).

| Informação Abstrata | Quem popula em ParsedFile (L3) | Regra Destino | Como |
|------------------|------------------|---------------|------|
| `prompt_file_exists` | `FsPromptReader` | V1 | O L3 checa no disco e popula o booleano diretamente. L1 consome via `HasPromptFilesystem`. |
| `has_test_coverage` | `FileWalker` + `RustParser` | V2 | Injeta ao ver adjacência em disco ou nó `#[cfg(test)]`. L1 consome via `HasCoverage`. |
| `imports` | `RustParser` | V3 | Extração de _use_ e _extern crate_. L1 consome via `HasImports`. |
| `tokens` | `RustParser` | V4 | Extração de call expressions. L1 consome via `HasTokens`. |
| `PromptHeader` | `FsPromptReader` e `RustParser` | V5 e V1 | Extração do header e injeção do hash atual do disco. V5 consome via `HasHashes`. |

| Campo Base IR | Quem popula | Como |
|---------------|-------------|------|
| `Import.target_layer` | `RustParser` | prefix matching do path contra `crystalline.toml` |
| `PromptHeader.current_hash` | `FsPromptReader` | `PromptReader::read_hash()` |
| `public_interface` | `RustParser` | extração de nós públicos do AST via tree-sitter |
| `prompt_snapshot` | `PromptSnapshotReader` | leitura e desserialização da seção `## Interface Snapshot` do prompt |

---

## Restrições (L1)

- Zero I/O — todas as structs são dados puros
- `ParsedFile` é construído inteiramente por L3 antes de chegar a L1
- `compute_delta` é função pura sobre dois `PublicInterface`
- Regras em L1 apenas leem os campos — nunca os derivam

---

## Critérios de Verificação
```
[critérios existentes de V1–V5 preservados]

Dado PublicInterface com função "check" adicionada
E snapshot sem essa função
Quando compute_delta() for chamado
Então InterfaceDelta.added_functions contém "check"

Dado PublicInterface idêntica ao snapshot
Quando compute_delta() for chamado
Então InterfaceDelta.is_empty() == true

Dado InterfaceDelta com +fn check e -fn validate
Quando describe() for chamado
Então retorna string contendo "+fn check" e "-fn validate"
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | parsed_file.rs, violation.rs, layer.rs |
| 2025-03-13 | Gap 2: adicionado prompt_file_exists, has_test_coverage, Import.target_layer, PromptHeader.current_hash | parsed_file.rs |
| 2025-03-13 | V6: adicionado PublicInterface, FunctionSignature, TypeSignature, InterfaceDelta, compute_delta | parsed_file.rs |
