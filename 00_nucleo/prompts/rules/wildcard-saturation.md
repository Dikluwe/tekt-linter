# Prompt L0 — Regra V16 `WildcardSaturation` (e família V17–V20: decisões mecânicas)
Hash do Código: cdc0e043

> Causalidade: este prompt é a origem de `01_core/rules/wildcard_saturation.rs`,
> do trait `HasDecisionArms` em `01_core/entities/rule_traits.rs` e da extracção
> de braços de decisão em `03_infra/rs_parser.rs`. Alterações ao comportamento da
> regra começam aqui, não no código.

**Decisão-mãe:** ADR-0016 (2026-08-14).
**Idioma desta fase:** Rust (`languages = ["rust"]`). O desenho é neutro de
linguagem; preencher outro parser estende a regra sem a reescrever.

---

## 1. Pergunta que a regra responde

«Este braço-curinga descarta informação de um enum fechado de domínio, de forma
que uma variante futura seja adoptada silenciosamente?» — e, para as regras
irmãs: «esta decisão está escrita no subconjunto mecânico enumerável?»

Modo de falha visado (medido no typst-crystalline, 2026-08): 26 ocorrências de
`_ => <valor arbitrário>` em enums fechados (`Unit`, `BinOp`, `FontStretch`,
`MathStyle`, `Color`, `Func`) — uma variante nova é saturada para um default
semanticamente errado sem erro de compilação.

## 2. Contrato de IR — `HasDecisionArms`

Por ficheiro analisado, o parser preenche (colecção vazia se a linguagem não
tiver extractor):

```
DecisionExpr {
    snippet_scrutinee: String,      // verbatim: "unit_kind", "self.kind()", …
    scrutinee_form: Path | FieldAccess | MethodCall | Index | Literal | Tuple,
    arms: Vec<DecisionArm>,
    span: Span,
}
DecisionArm {
    pattern_snippet: String,        // verbatim: "_", "other", "Unit::Pt", "0..=9"
    is_catchall: bool,              // wildcard ou binding que captura tudo
    bound_ident_used_in_body: bool, // Filtro 2 (reincorporação)
    qualified_prefixes: Vec<String>,// ["Unit"] em braços Unit::X — enum candidato
    has_guard: bool,
    guard_is_compound: bool,        // contém && ou ||
    pattern_is_range: bool,         // 0..=9, 'a'..='z'
    pattern_depth: u8,              // níveis de destruturação
    or_alternatives: u16,           // nº de alternativas em or-pattern (1 = não-or)
    body_form: ErrorBarrier | MessageProducer | EnumPath | LiteralNeutral
             | LiteralOther | Call | EmptyBlock | Continue | Other,
    body_snippet: String,           // verbatim, truncado a 80 cols p/ mensagem
    span: Span,
}
```

Invariantes: (i) parsers sem extractor devolvem `vec![]` — a regra produz zero
violações; (ii) todos os snippets são verbatim do ficheiro — as mensagens citam
código real, nunca síntese abstracta.

## 3. Pipeline de classificação de V16 (ordem estrita)

```
catchall detectado (is_catchall)
 ├─ bound_ident_used_in_body ............ ISENTO (reincorporação: `other => f(other)`)
 ├─ scrutinee_form ∈ {MethodCall, Index, Literal} . ISENTO (scrutinee aberto)
 ├─ body_form = ErrorBarrier ............ ISENTO (barreira de erro em compile/runtime)
 ├─ body_form = MessageProducer ......... ISENTO (falha ruidosa: format!, write!, error/cannot/expected)
 └─ enum candidato (≥2 braços partilham qualified_prefixes no mesmo match)
     ├─ body_form = EnumPath | LiteralOther ... VIOLAÇÃO DENY-class (saturação arbitrária)
     ├─ body_form = LiteralNeutral ............ VIOLAÇÃO WARN-class (default neutro)
     ├─ body_form = Call ...................... INFO (delegação a outro despachante)
     └─ body_form = EmptyBlock | Continue ..... WARN-class walker parcial

Tabela de neutros (forma final calibrada):
`false`, `true`, `0`, `0.0`, `""`, `()`, tuplas de neutros `(0, 0)`, `(None, None)`,
`Default::default()`, `None`, `Option::None`, `Value::None`, `String::new()`, `Vec::new()`,
`vec![]` (macro construtora vazia).
```

Regras irmãs sobre o mesmo IR (uma só passagem):
- **V17**: `has_guard && guard_is_compound` → warning «guard composto».
- **V18**: `pattern_is_range` fora de módulos allowlistados (`lexer`, `numbering`)
  → warning.
- **V19** (info): `or_alternatives > 1` → reporta «este braço condensa N
  alternativas — cobertura de braços subestima N×».
- **V20** (info): `pattern_depth > 2` e o match não é tabela regular (braços de
  tupla sobre os mesmos tipos) → reporta profundidade.

## 4. Severidades e promoção

| Regra | Nível inicial | Promoção |
| :---- | :---- | :---- |
| V16 | warning | error quando o worklist do typst-crystalline fechar (ADR-0016 §7) |
| V17, V18 | warning | error no mesmo ratchet |
| V19, V20 | **info** (primeiro consumidor do nível novo) | nunca sobe — são métricas |

## 5. Mensagens — nomenclatura da linguagem (requisito vinculativo)

A mensagem usa o **termo nativo da linguagem do ficheiro** e cita o snippet
verbatim. Tabela de termos (semente; cresce com os parsers):

| Linguagem | Termo para catch-all | Exemplo de mensagem V16 DENY-class |
| :---- | :---- | :---- |
| Rust | wildcard `_ =>` | `wildcard `_ =>` satura variantes futuras para `Unit::Percent` — exige exaustividade nominal` |
| Python | `case _` | `` `case _` satura variantes futuras para `Percent` — exigir braços nomeados `` |
| TypeScript | cláusula `default:` | `` `default:` satura casos futuros para `Percent` — exigir cases explícitos `` |
| Go | cláusula `default:` | idem TS |
| Zig | — (switch exaustivo; regra inaplicável) | — |

Implementação: `decision_arm_term_for(language)` em `rule_traits.rs`, ao lado de
`forbidden_symbols_for(language)`; a mensagem é montada na regra pura com
`arm.pattern_snippet` e `arm.body_snippet` — nunca hardcoded a Rust.

## 6. Excepções

`crystalline.toml`:

```toml
[wildcard_exceptions]
"01_core/src/entities/gradient.rs:221" = "hub intencional: fallback lossy para sRGB documentado no ADR-0109"
```

Formato obrigatório da justificativa: frase com a razão (não «ok»). Excepção sem
justificativa ou com span obsoleto (linha deixou de ser catch-all) é ela própria
uma violação `warning` — as excepções apodrecem se ninguém as regar.

## 7. Critérios de aceitação

1. **Teste de mutação (fixture `tests/fixtures/ghost_variant.rs`)**: enum com
   `_ => Enum::Default`; adicionar variante fantasma mantém a violação no mesmo
   braço — prova que a regra aponta ao mecanismo, não ao texto.
2. **Concordância com o ground truth syn** (ADR-0016, questão 1): ≥ 95% por
   categoria sobre o typst-crystalline (26 DENY / ~18 neutros / 131 walkers /
   1 delegação).
3. **Auto-validação**: `crystalline-lint .` verde no repo do linter.
4. **Não-regressão**: zero violações V16–V20 em projectos TS e Python de
   referência.
5. **Mensagens**: snapshot tests verificam que cada mensagem contém o snippet
   verbatim e o termo da linguagem do ficheiro.

## 8. Validação

```bash
cargo build --workspace --release
cargo test -p crystalline-lint --lib
crystalline-lint .                                   # auto-validação, 0 violações
crystalline-lint --checks v16,v17,v18,v19,v20 /caminho/typst-crystalline
```
