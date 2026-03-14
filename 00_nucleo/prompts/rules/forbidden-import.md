# Prompt: Rule V3 - Forbidden Import (forbidden-import)

**Camada**: L1 (Core - Rules)
**Regra**: V3
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

O fluxo de dependência causal flui unidirecionalmente através dos
estratos L0→L4. Importações que violam essa direção introduzem
acoplamento invertido — o defeito estrutural mais destrutivo em
sistemas cristalinos.

V3 detecta essas inversões comparando a camada do arquivo com a
camada destino de cada import. Ambas já estão resolvidas e expostas
por uma abstração (via trait `HasImports`) — L1 apenas compara, nunca deriva.

---

## Especificação

A regra recebe uma entidade (via trait `HasImports`) e varre a lista de imports exposta.
Para cada `Import`, compara a camada declarada do arquivo com `import.target_layer`
usando a matriz de permissões abaixo.

`Import.target_layer` é populado por L3 (RustParser) no momento
do parse, baseado no prefix do path e no mapeamento de camadas
declarado em `crystalline.toml`. V3 nunca inspeciona strings
de path diretamente.

**Matriz de Permissões:**

| Camada do arquivo | target_layer proibido |
|-------------------|-----------------------|
| L1 | L2, L3, L4, Lab |
| L2 | L3, L4, Lab |
| L3 | L2, L4, Lab |
| L4 | Lab |
| L0, Lab | — (sem restrições de import) |

`Layer::Unknown` não gera violação V3 — imports de crates
externas não mapeiam para camadas cristalinas e são válidos
em qualquer estrato.

---

## Estrutura da Violação Gerada
```rust
Violation {
    rule_id: "V3",
    level: ViolationLevel::Error,
    message: format!(
        "Inversão de gravidade: {:?} não pode importar de {:?} ('{}')",
        file.layer,
        import.target_layer,
        import.path
    ),
    location: Location {
        path: file.path.clone(),
        line: import.line,
        column: 0,
    },
}
```

---

## Restrições (L1 Pura)

- Nenhuma inspeção de string de path — apenas comparação de `Layer`
- Nenhum acesso a `crystalline.toml` — mapeamento já resolvido por L3
- `Layer::Unknown` é explicitamente ignorado — não é violação
- Uma violação por import proibido — não agrega

---

## Critérios de Verificação
```
Dado arquivo em L2 com Import { target_layer: Layer::L3, line: 4, .. }
Quando V3::check() for chamado
Então retorna vec![Violation { rule_id: "V3", location.line: 4 }]

Dado arquivo em L1 com Import { target_layer: Layer::Unknown, .. }
Quando V3::check() for chamado
Então retorna vec![] — crate externa não é violação

Dado arquivo em L4 com Import { target_layer: Layer::L1, .. }
Quando V3::check() for chamado
Então retorna vec![] — L4 pode importar de todos exceto Lab

Dado arquivo em L3 com dois imports:
  Import { target_layer: Layer::L2, line: 3 }
  Import { target_layer: Layer::L1, line: 7 }
Quando V3::check() for chamado
Então retorna vec! com uma violação (line 3) e não a outra (line 7)

Dado arquivo em Lab
Quando V3::check() for chamado
Então retorna vec![] — Lab não tem restrições de import
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | forbidden_import.rs |
| 2025-03-13 | Gap 3: removida menção a regex, alinhado com Import.target_layer | forbidden_import.rs |
