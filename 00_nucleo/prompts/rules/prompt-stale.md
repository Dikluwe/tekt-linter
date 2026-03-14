# Prompt: Rule V6 - Prompt Stale (prompt-stale)

**Camada**: L1 (Core — Rules)
**Regra**: V6
**Criado em**: 2025-03-13
**Arquivos gerados**:
  - 01_core/rules/prompt_stale.rs + test

---

## Contexto

V5 detecta quando o prompt mudou depois do código.
V6 detecta o inverso: quando o código mudou depois do prompt.

Juntas, V5 e V6 fecham o ciclo de integridade bidirecional entre
L0 e os estratos de implementação. Sem V6, modificações diretas
no código — por humanos ou agentes — acumulam divergência silenciosa
até o prompt perder sua função de origem causal.

V6 é especialmente crítico em ambientes com múltiplos agentes:
dois agentes podem criar contradições entre arquivos que
individualmente passam no linter mas coletivamente representam
um estado impossível.

---

## Especificação

A regra recebe `ParsedFile` e verifica se a interface pública do
arquivo mudou estruturalmente desde a última revisão registrada
no prompt de origem.

**Campos necessários em `ParsedFile`** (novos, a adicionar em
`violation-types.md`):
```rust
pub struct ParsedFile {
    // ... campos existentes ...

    /// Para V6: snapshot da interface pública extraída do AST.
    /// Populado por L3 (RustParser) no momento do parse.
    pub public_interface: PublicInterface,

    /// Para V6: snapshot da interface pública registrada no prompt.
    /// Populado por L3 via PromptSnapshotReader.
    /// None se o prompt não tem snapshot registrado ainda.
    pub prompt_snapshot: Option<PublicInterface>,
}

/// Interface pública extraída do AST — agnóstica de linguagem.
/// Não inclui implementação, apenas contratos visíveis externamente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicInterface {
    /// Funções e métodos públicos com suas assinaturas.
    pub functions: Vec<FunctionSignature>,
    /// Structs, enums e traits públicos com seus campos/variantes.
    pub types: Vec<TypeSignature>,
    /// Imports públicos (re-exports).
    pub reexports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSignature {
    pub name: String,
    pub kind: TypeKind,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Struct,
    Enum,
    Trait,
}
```

**Lógica da regra:**
```rust
pub fn check(file: &ParsedFile) -> Vec<Violation> {
    // V6 só se aplica a arquivos com prompt registrado
    let header = match &file.prompt_header {
        Some(h) => h,
        None => return vec![], // V1 já cobre ausência de header
    };

    // Se não há snapshot no prompt, não há baseline para comparar
    let snapshot = match &file.prompt_snapshot {
        Some(s) => s,
        None => return vec![], // primeira geração — sem histórico
    };

    // Interface atual idêntica ao snapshot → sem divergência
    if &file.public_interface == snapshot {
        return vec![];
    }

    // Interface mudou — prompt pode estar desatualizado
    let delta = compute_delta(&file.public_interface, snapshot);

    vec![Violation {
        rule_id: "V6".to_string(),
        level: ViolationLevel::Warning,
        message: format!(
            "Prompt potencialmente desatualizado: interface pública mudou \
             desde a última revisão de '{}'. Delta: {}",
            header.prompt_path,
            delta.describe()
        ),
        location: Location {
            path: file.path.clone(),
            line: 1,
            column: 0,
        },
    }]
}
```

**`compute_delta`** — função pura em L1:
```rust
pub struct InterfaceDelta {
    pub added_functions: Vec<FunctionSignature>,
    pub removed_functions: Vec<FunctionSignature>,
    pub added_types: Vec<TypeSignature>,
    pub removed_types: Vec<TypeSignature>,
    pub added_reexports: Vec<String>,
    pub removed_reexports: Vec<String>,
}

impl InterfaceDelta {
    pub fn describe(&self) -> String {
        // Produz string legível:
        // "+fn check, -fn validate, +struct Delta"
    }
    pub fn is_empty(&self) -> bool { ... }
}
```

---

## Como o snapshot chega ao ParsedFile

Responsabilidade de L3 — novo componente `PromptSnapshotReader`:

O prompt em L0 passa a conter uma seção `## Interface Snapshot`
gerada automaticamente no momento da materialização:
```markdown
## Interface Snapshot
<!-- GENERATED — não edite manualmente -->
<!-- crystalline-snapshot: {"functions":[...],"types":[...],"reexports":[]} -->
```

L3 lê essa seção, desserializa o JSON e popula `ParsedFile.prompt_snapshot`.
L3 também extrai `ParsedFile.public_interface` do AST via tree-sitter.
V6 em L1 compara os dois — sem I/O.

---

## Silenciamento de V6

V6 é silenciado quando:

1. O prompt é revisado (`@updated` atualizado) e o snapshot
   é regenerado para refletir a interface atual
2. O código é revertido para o estado do snapshot

Nenhum mecanismo de `// nolint` — V6 exige decisão explícita,
não supressão silenciosa.

---

## Estrutura da Violação Gerada
```rust
Violation {
    rule_id: "V6",
    level: ViolationLevel::Warning,
    message: "Prompt potencialmente desatualizado: interface pública \
              mudou desde a última revisão de '<prompt_path>'. \
              Delta: <delta.describe()>",
    location: Location {
        path: file.path.clone(),
        line: 1,
        column: 0,
    },
}
```

---

## Restrições (L1 Pura)

- Zero I/O — `public_interface` e `prompt_snapshot` chegam
  prontos via `ParsedFile`
- `compute_delta` é função pura sobre dois `PublicInterface`
- V6 nunca lê o prompt diretamente — L3 faz isso
- Sem supressão inline — silenciamento exige mudança no prompt

---

## Critérios de Verificação
```
Dado arquivo com public_interface idêntica ao prompt_snapshot
Quando V6::check() for chamado
Então retorna vec![]

Dado arquivo com função pública adicionada desde o snapshot
Quando V6::check() for chamado
Então retorna Violation V6 com delta descrevendo a adição

Dado arquivo com função pública removida desde o snapshot
Quando V6::check() for chamado
Então retorna Violation V6 com delta descrevendo a remoção

Dado arquivo sem prompt_header
Quando V6::check() for chamado
Então retorna vec![] — V1 já cobre este caso

Dado arquivo com prompt_snapshot = None (primeira geração)
Quando V6::check() for chamado
Então retorna vec![] — sem baseline para comparar

Dado arquivo com mudança apenas em comentário interno
Quando V6::check() for chamado
Então retorna vec![] — mudança cosmética não dispara V6

Dado dois arquivos modificados por agentes distintos
com interfaces contraditórias entre si
Quando V6::check() rodar em ambos
Então cada um reporta V6 independentemente
permitindo detecção de contradição inter-agente
```

---

## Dependências de novos prompts

Este prompt depende de dois novos prompts a criar:

- `contracts/prompt-snapshot-reader.md` — trait L1 + impl L3
  para ler e desserializar a seção `## Interface Snapshot` do prompt
- `rs-parser.md` (revisão) — extrair `PublicInterface` do AST
  além do que já extrai

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | prompt_stale.rs |
