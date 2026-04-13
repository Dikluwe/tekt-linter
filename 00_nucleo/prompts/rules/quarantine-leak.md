# Prompt: Rule V10 - Quarantine Leak (quarantine-leak)
Hash do Código: f828409d

**Camada**: L1 (Core — Rules)
**Regra**: V10
**Criado em**: 2026-03-16 (ADR-0007)
**Arquivos gerados**:
  - 01_core/rules/quarantine_leak.rs + test

---

## Contexto

O diretório `lab/` é um ambiente de quarentena intencional — código
experimental, algoritmos sujos, protótipos sem restrições. A IA pode
usar o lab como rascunho sem violar V1–V9.

O problema: nada impede que código de produção (L1–L4) importe de
`lab/`. Quando a IA prototipa um algoritmo no lab e precisa usá-lo
em produção, o caminho de menor resistência é `use crate::lab::...`.
O resultado é código de produção com dependência em código sem
garantias arquiteturais.

V3 já proíbe que L4 importe de Lab via matriz de permissões, mas
trata isso como Error. V10 estende a proibição a L1, L2 e L3 e
eleva o nível para Fatal em todos os casos — pelo mesmo motivo que
V8: código de produção com dependência invisível de lab não oferece
garantias reais.

A assimetria é absoluta e intencional: o lab pode importar produção
para testar, a produção nunca importa o lab.

---

## Especificação

V10 opera sobre `ParsedFile.imports` por arquivo, na fase Map.
Aplica-se a arquivos com `layer == L1 | L2 | L3 | L4`.
Arquivos em `Lab` e `L0` são isentos.

```rust
pub fn check<'a, T: HasImports<'a>>(file: &T) -> Vec<Violation<'a>> {
    if matches!(file.layer(), Layer::Lab | Layer::L0 | Layer::Unknown) {
        return vec![];
    }

    file.imports()
        .iter()
        .filter(|import| import.target_layer == Layer::Lab)
        .map(|import| Violation {
            rule_id: "V10".to_string(),
            level: ViolationLevel::Fatal,
            message: format!(
                "Quarentena violada: '{}' é código de produção e não pode \
                 importar de lab/. Migrar o símbolo para a camada apropriada \
                 antes de usar em produção.",
                import.path
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

## Restrições (L1 Pura)

- Recebe `ParsedFile` via trait `HasImports` — zero I/O
- `target_layer == Layer::Lab` já está resolvido por L3 (RustParser)
  via `LayerResolver` — V10 nunca inspeciona strings de path
- Fatal — não configurável via `crystalline.toml`
- Nenhuma exceção declarável — a assimetria lab↔produção é absoluta

---

## Relação com V3

V3 proíbe imports de Lab a partir de L4 via matriz de permissões,
com nível Error. V10 é mais restritivo:

| Aspecto | V3 | V10 |
|---------|-----|-----|
| Camadas cobertas | L4 → Lab | L1, L2, L3, L4 → Lab |
| Nível | Error | Fatal |
| Configurável | Sim | Não |

Ambas as regras podem disparar para o mesmo import em L4. Isso é
intencional — Fatal tem precedência e o output torna visível a
gravidade máxima.

---

## Critérios de Verificação
```
Dado arquivo L1 com Import { target_layer: Layer::Lab, line: 3, .. }
Quando V10::check() for chamado
Então retorna Violation { rule_id: "V10", level: Fatal, location.line: 3 }

Dado arquivo L2 com Import { target_layer: Layer::Lab, .. }
Quando V10::check() for chamado
Então retorna Violation V10 Fatal

Dado arquivo L3 com Import { target_layer: Layer::Lab, .. }
Quando V10::check() for chamado
Então retorna Violation V10 Fatal

Dado arquivo L4 com Import { target_layer: Layer::Lab, .. }
Quando V10::check() for chamado
Então retorna Violation V10 Fatal
— redundante com V3, nível Fatal tem precedência

Dado arquivo Lab com Import { target_layer: Layer::L1, .. }
Quando V10::check() for chamado
Então retorna vec![] — lab pode importar produção

Dado arquivo L0 com qualquer import
Quando V10::check() for chamado
Então retorna vec![] — L0 é isento

Dado arquivo L2 com Import { target_layer: Layer::L3, .. }
Quando V10::check() for chamado
Então retorna vec![] — V3 cobre isso, não V10

Dado arquivo L1 com Import { target_layer: Layer::Unknown, .. }
Quando V10::check() for chamado
Então retorna vec![] — crate externa, não é lab
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-16 | Criação inicial (ADR-0007) | quarantine_leak.rs |
| 2026-03-16 | Materialização: check() implementado com HasImports, isenção Lab/L0/Unknown, 11 testes; módulo registado em rules/mod.rs | quarantine_leak.rs |
