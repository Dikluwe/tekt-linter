# Prompt: Rule V6 - Prompt Stale (prompt-stale)
Hash do Código: f021b8d9

**Camada**: L1 (Core — Rules)
**Regra**: V6
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13
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

**Critério de igualdade — assinatura completa, não apenas nome:**

Duas `FunctionSignature` são idênticas se e somente se `name`,
`params` e `return_type` são todos iguais. Mudança em qualquer
um dos três campos é uma quebra de contrato — equivalente a
remover a assinatura antiga e adicionar uma nova.
```
foo(a: String) -> bool   ←→   foo(a: Vec<String>) -> bool
```

Essa mudança gera duas entradas no delta (`-fn foo`, `+fn foo`)
e dispara V6. Comparação por nome apenas é insuficiente.

A implementação usa `PartialEq` derivado sobre a struct completa:
```rust
// CORRETO — compara name + params + return_type via PartialEq
.filter(|f| !snapshot.functions.contains(f))

// ERRADO — compara apenas nome, ignora mudanças de assinatura
.filter(|f| !snapshot.functions.iter().any(|g| g.name == f.name))
```

O mesmo critério se aplica a `TypeSignature`: `name`, `kind` e
`members` devem ser todos iguais para duas assinaturas de tipo
serem consideradas idênticas.

---

## Lógica da regra
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

---

## compute_delta — função pura em L1
```rust
/// Computa diferença entre interface atual e snapshot do prompt.
/// Usa PartialEq completo sobre FunctionSignature e TypeSignature —
/// name + params + return_type devem ser todos iguais.
/// Mudança de assinatura aparece como remoção + adição.
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

`InterfaceDelta.describe()` produz string legível para a mensagem
de violação. Ordem: adições antes de remoções, funções antes de
tipos. Exemplo: `+fn check, -fn validate, +struct Delta`.

---

## Como o snapshot chega ao ParsedFile

Responsabilidade de L3 — componente `PromptSnapshotReader`:

O prompt em L0 contém uma seção `## Interface Snapshot` gerada
automaticamente no momento da materialização:
```markdown
## Interface Snapshot
<!-- GENERATED — não edite manualmente -->
<!-- crystalline-snapshot: {"functions":[...],"types":[...],"reexports":[]} -->
```

L3 lê essa seção, desserializa o JSON e popula
`ParsedFile.prompt_snapshot`. L3 também extrai
`ParsedFile.public_interface` do AST via tree-sitter.
V6 em L1 compara os dois — sem I/O.

---

## Silenciamento de V6

V6 é silenciado quando:

1. O prompt é revisado e `--update-snapshot` regenera o snapshot
   para refletir a interface atual
2. O código é revertido para o estado do snapshot

Nenhum mecanismo de `// nolint` — V6 exige decisão explícita,
não supressão silenciosa.

---

## Estrutura da Violação Gerada
```rust
Violation {
    rule_id: "V6",
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
}
```

---

## Restrições (L1 Pura)

- Zero I/O — `public_interface` e `prompt_snapshot` chegam
  prontos via `ParsedFile`
- `compute_delta` é função pura sobre dois `PublicInterface`
- Comparação usa `PartialEq` derivado — inclui todos os campos
  de `FunctionSignature` e `TypeSignature`, nunca apenas `name`
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

Dado arquivo com foo(a: String) -> bool no snapshot
E foo(a: Vec<String>) -> bool na interface atual (mesmo nome, assinatura diferente)
Quando V6::check() for chamado
Então retorna Violation V6
E delta contém -fn foo(String) e +fn foo(Vec<String>)
— mudança de assinatura é quebra de contrato

Dado arquivo com foo(a: String) -> bool no snapshot
E foo(a: String) -> bool na interface atual (idênticos)
Quando V6::check() for chamado
Então retorna vec![] — assinatura completa idêntica, sem divergência

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

## Dependências

- `violation-types.md` — define `PublicInterface`, `FunctionSignature`,
  `TypeSignature`, `InterfaceDelta`, `compute_delta`
- `contracts/prompt-snapshot-reader.md` — trait L1 + impl L3
- `rs-parser.md` — extrai `PublicInterface` do AST

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | prompt_stale.rs |
| 2025-03-13 | Critério de igualdade explicitado: assinatura completa via PartialEq, não apenas nome. Casos de teste de mudança de assinatura adicionados. compute_delta completo com implementação correta | prompt_stale.rs |
