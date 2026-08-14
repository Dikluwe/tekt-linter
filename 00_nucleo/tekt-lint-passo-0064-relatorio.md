# Laudo 0064 — Calibração de V16/V20 (Pós-Validação Cruzada)

**Onde roda**: clone canónico `tekt-linter`
**Data**: 2026-08-14
**Estado**: `IMPLEMENTADO`
**Decisão-mãe**: [ADR-0016](adr/0016-regras-decisao-mecanica.md) (Status: `ACEITO`)
**Prompt L0**: [`00_nucleo/064-wildcard-calibration.md`](064-wildcard-calibration.md) e [`00_nucleo/prompts/rules/wildcard-saturation.md`](prompts/rules/wildcard-saturation.md)

---

## 1. Resumo Executivo

O passo 0064 executou a calibração de precisão das regras de decisão mecânica (**V16** e **V20**), respondendo aos dois desvios observados na corrida de referência sobre o `typst-crystalline`:

1. **Desvio 1 (V16 MessageProducer)**: braços catch-all geradores de mensagens ruidosas (e.g. `_ => format!("cannot apply {op:?} to {a} and {b}")`) foram reclassificados de DENY para a nova categoria `BodyForm::MessageProducer` (isenta de V16 por falhar ruidosamente no runtime em vez de adotar silenciosamente).
2. **Desvio 2 (V20 Auditoria de Amostra)**: auditoria obrigatória de amostra aleatória de 20 dos 515 avisos `info` confirmou 100% de aninhamento ad-hoc real (`Option<Value::Variant>`, `[Value::Variant]`), validando que a métrica reflete com fidelidade a superfície ad-hoc do repositório analisado.
3. **Desvio 0 (Tabela de Neutros Calibrada)**: `None`, `(None, None)`, `String::new()`, `Vec::new()` e `vec![]` (vazio) consolidados em `BodyForm::LiteralNeutral`.

---

## 2. Resultados da Re-Corrida sobre `typst-crystalline`

Comando:
```bash
cargo run --bin crystalline-lint -- --checks v16,v17,v18,v19,v20 --format sarif /home/dikluwe/Documentos/Antigravity/typst-crystalline
```

### Tabela Comparativa de Concordância

| Regra / Categoria | Pré-Calibração (0063) | Calibrado (0064) | Observação |
| :--- | :--- | :--- | :--- |
| **V16 DENY-class** | 16 | **8** | Casos como `format!("cannot apply...")` isentos como `MessageProducer` |
| **V16 WARN-neutro** | 126 | **132** | `None`, `(None, None)`, `vec![]` classificados como neutros |
| **V16 WARN-walker** | 43 | **43** | Inalterado (concordância total) |
| **V16 INFO-delegação** | 6 | **6** | Inalterado |
| **V16 Total** | 197 | **195** | −2 casos de mensagem de erro |
| **V17 (CompoundGuard)** | 29 | **29** | Inalterado (estável) |
| **V18 (RangePattern)** | 2 | **2** | Inalterado (estável) |
| **V19 (OrAlternatives)**| 265 | **265** | Inalterado (estável, ∈ [250, 280]) |
| **V20 (DeepNesting)** | 515 | **515** | Amostra auditada: 100% ad-hoc real |

---

## 3. Auditoria da Amostra de V20 (20/515)

Amostra aleatória com semente fixa inspecionada:
1. `01_core/src/compiler/introspect/from_tags.rs:320` -> `Some(Value::Int(n))` (ad-hoc real)
2. `01_core/src/compiler/eval/closures.rs:180` -> `Some(FlowEvent::Return(_, None, _))` (ad-hoc real)
3. `01_core/src/compiler/stdlib/label.rs:30` -> `Some(Value::Str(s))` (ad-hoc real)
4. `01_core/src/compiler/stdlib/foundations/cast.rs:278` -> `Some(Value::Int(i))` (ad-hoc real)
5. `01_core/src/compiler/stdlib/counter.rs:393` -> `[Value::Str(key)]` (ad-hoc real)
6. `01_core/src/compiler/layout/tests.rs:9443` -> `FrameItem::Group { clip_mask: Some(ShapeKind::Rect), .. }` (ad-hoc real)
7. `01_core/src/compiler/introspect/extract_payload.rs:238` -> `Some(ElementPayload::Bibliography { entries })` (ad-hoc real)
8. `01_core/src/compiler/eval/tests.rs:12937` -> `Some(Value::Array(items))` (ad-hoc real)
9. `01_core/src/compiler/stdlib/structural/outline.rs:150` -> `Some(Value::Str(s))` (ad-hoc real)
10. `01_core/src/compiler/eval/control_flow.rs:177` -> `Some(FlowEvent::Return(..))` (ad-hoc real)
11. `01_core/src/compiler/eval/control_flow.rs:172` -> `Some(FlowEvent::Break(_))` (ad-hoc real)
12. `01_core/src/compiler/eval/tests.rs:13981` -> `Some(Value::Content(Content::Shape(e)))` (ad-hoc real)
13. `01_core/src/compiler/stdlib/counter.rs:226` -> `[Value::Func(callback)]` (ad-hoc real)
14. `01_core/src/compiler/stdlib/foundations/cast.rs:95` -> `[Value::Float(f)]` (ad-hoc real)
15. `01_core/src/compiler/eval/control_flow.rs:74` -> `Some(FlowEvent::Break(_))` (ad-hoc real)
16. `01_core/src/compiler/stdlib/collections.rs:935` -> `[Value::Regex(re)]` (ad-hoc real)
17. `01_core/src/compiler/stdlib/structural/outline.rs:106` -> `Some(Value::Func(f))` (ad-hoc real)
18. `01_core/src/compiler/stdlib/counter.rs:326` -> `[Value::Str(key), Value::Func(callback)]` (ad-hoc real)
19. `01_core/src/compiler/stdlib/structural/table_lines.rs:182` -> `Some(Value::Align(Align2D { v: Some(VAlign::Bottom), .. }))` (ad-hoc real)
20. `01_core/src/compiler/stdlib/layout.rs:100` -> `Some(Value::Str(s))` (ad-hoc real)

**Conclusão da auditoria**: 20/20 (100%) são aninhamentos ad-hoc genuínos. O número 515 é honesto e aceito como a superfície real de aninhamento ad-hoc.

---

## 4. Estado da Árvore e Validação

- **`cargo test --lib`**: 534 testes unitários passando (0 falhas).
- **`cargo test --test fixtures`**: 69 testes de fixtures passando (incluindo `error_message_arm_is_exempt_from_v16`).
- **`crystalline-lint .`**: ✓ 0 erros, 0 avisos (apenas diagnósticos informativos emitidos).
