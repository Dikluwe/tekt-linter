# Prompt: Rule V9 - Pub Leak (pub-leak)
Hash do Código: a626b323

**Camada**: L1 (Core — Rules)
**Regra**: V9
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 01_core/rules/pub_leak.rs + test

---

## Contexto

V3 garante a direção do fluxo de dependência — L2 não importa
L3. Mas não garante a granularidade. Um agente pressionado pelo
compilador Rust adiciona `pub` a helpers internos de L1, e L2
passa a depender de detalhes de implementação que nunca deveriam
ser visíveis externamente.

V9 garante que imports de L2 e L3 em L1 usem apenas
subdiretórios explicitamente designados como portas públicas.

---

## Portas de L1

Declaradas em `crystalline.toml`:
```toml
[l1_ports]
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"
```

Qualquer subdiretório de L1 não listado aqui é interno —
inacessível de L2 ou L3 mesmo que seus símbolos sejam `pub`.

---

## Especificação

V9 opera sobre `ParsedFile.imports`, inspecionando
`import.target_subdir` para imports com `target_layer == L1`:
```rust
pub fn check<'a>(file: &ParsedFile<'a>, ports: &L1Ports) -> Vec<Violation<'a>> {
    if !matches!(file.layer, Layer::L2 | Layer::L3) {
        return vec![];
    }

    file.imports.iter()
        .filter(|import| import.target_layer == Layer::L1)
        .filter(|import| {
            // Se subdir é None, é crate externa — não é V9
            // Se subdir é Some mas não está nas portas — é V9
            import.target_subdir
                .map(|subdir| !ports.contains(subdir))
                .unwrap_or(false)
        })
        .map(|import| Violation {
            rule_id: "V9".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Vazamento de encapsulamento: '{}' importa '{}' \
                 de subdiretório interno de L1. \
                 Use apenas as portas declaradas em [l1_ports].",
                file.path.display(),
                import.path
            ),
            location: Location {
                path: Cow::Borrowed(file.path),
                line: import.line,
                column: 0,
            },
        })
        .collect()
}
```

`L1Ports` é injetado via L4 — lido de `crystalline.toml` por L3
e passado como parâmetro. V9 nunca lê o toml diretamente.

---

## Campo adicional em `Import`
```rust
pub struct Import<'a> {
    pub path: &'a str,
    pub line: usize,
    pub kind: ImportKind,
    pub target_layer: Layer,
    pub target_subdir: Option<&'a str>, // novo — ADR-0006
    // None para crates externas (Layer::Unknown)
    // Some("entities") para crate::entities::*
    // Some("internal") para crate::internal::*
}
```

Resolvido por L3 (RustParser) via mapeamento de `[layers]` e
`[l1_ports]` em `crystalline.toml`. V9 em L1 apenas compara.

---

## Restrições (L1 Pura)

- Recebe `ParsedFile` e `&L1Ports` — zero I/O
- `L1Ports` é um conjunto de strings injetado por L4
- Não inspeciona visibilidade Rust (`pub`/`pub(crate)`) —
  inspeciona apenas o path do import
- Aplica-se apenas a L2 e L3 importando L1

---

## Critérios de Verificação
```
Dado arquivo L2 com import crate::core::internal::helper
E "internal" não está em [l1_ports]
Quando V9::check() for chamado
Então retorna Violation V9 Error com linha do import

Dado arquivo L2 com import crate::entities::Layer
E "entities" está em [l1_ports]
Quando V9::check() for chamado
Então retorna vec![] — porta válida

Dado arquivo L3 com import crate::contracts::FileProvider
E "contracts" está em [l1_ports]
Quando V9::check() for chamado
Então retorna vec![] — porta válida

Dado arquivo L1 com qualquer import
Quando V9::check() for chamado
Então retorna vec![] — V9 não se aplica a L1

Dado arquivo L4 com import crate::core::internal::helper
Quando V9::check() for chamado
Então retorna vec![] — V9 não se aplica a L4
— L4 pode compor tudo, V3 protege os demais

Dado import de reqwest::Client em L2
Quando V9::check() for chamado
Então retorna vec![] — crate externa, target_subdir = None
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | pub_leak.rs |
