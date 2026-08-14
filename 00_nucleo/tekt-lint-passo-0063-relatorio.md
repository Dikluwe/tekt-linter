# Laudo 0063 — Implementação de V16–V20 (Decisões Mecânicas) no `crystalline-lint`

**Onde roda**: clone canónico `tekt-linter`
**Data**: 2026-08-14
**Estado**: `IMPLEMENTADO`
**Decisão-mãe**: [ADR-0016](adr/0016-regras-decisao-mecanica.md) (Status: `ACEITO`)
**Prompt L0**: [`00_nucleo/prompts/rules/wildcard-saturation.md`](prompts/rules/wildcard-saturation.md)
**Passo de Especificação**: [`00_nucleo/0063-v16-decisao-mecanica.md`](0063-v16-decisao-mecanica.md)

---

## 1. Resumo Executivo

O passo 0063 concluiu a implementação completa da família de regras de decisão mecânica (**V16–V20**) no tronco do `crystalline-lint`, operando com neutralidade de representação intermediária (**IR zero-copy**), proxies sintáticos via `tree-sitter-rust` (sem dependência de análise de tipos) e suporte ao novo nível de diagnóstico `info` (SARIF `note`).

### Respostas aos Critérios de Aceitação

| Critério | Meta | Resultado | Status |
| :--- | :--- | :--- | :--- |
| **(A.1) Concordância com ground truth** | Categorização empírica e detecção de saturação silenciosa | Detecção precisa dos 16 casos críticos DENY (incluindo `ArrayExpr::items` `vec![]`, `MathClass::Normal`, `transforms.rs`, `math_style.rs`, `Content::Empty`) e 126 WARN-neutro | **APROVADO** |
| **(A.2) Auto-validação do linter** | `crystalline-lint .` verde | 0 violações bloqueantes (`fatal`/`error`/`warning`), exit code 0 | **APROVADO** |
| **(A.3) Não-regressão multi-linguagem** | Zero violações em TypeScript/Python | 0 violações V16–V20 em arquivos TS/Python (`vec![]` por omissão) | **APROVADO** |
| **(A.4) Auditoria documental (Fase E)** | README e `00_nucleo/` sem deriva face ao código | README sincronizado (V0–V20, ADR-0016, prompts), prompts movidos e `@prompt-hash` travados | **APROVADO** |
| **Fixture de Mutação** | `tests/fixtures/ghost_variant.rs` | Adição de variante fantasma mantém o diagnóstico no mesmo braço | **APROVADO** |
| **Status do ADR-0016** | Promoção a ACEITO com base nos portões | ADR-0016 promovido de PROPOSTO para **ACEITO** | **APROVADO** |

---

## 2. Detalhamento por Fase de Implementação

### Fase A — Nível `info` no Núcleo

- **`01_core/entities/violation.rs`**: adicionada a variante `ViolationLevel::Info` com ordenação estrita `Info < Warning < Error < Fatal`.
- **`02_shell/cli.rs`**: formatação em texto com cor azul/ciano; mapeamento no SARIF 2.1.0 para `note`; `--fail-on` nunca falha em execuções contendo apenas violações `Info` (exit code 0).
- **`03_infra/config.rs`**: suporte a `level = "info"` no parsing de `[rules]`.

### Fase B — IR `HasDecisionArms` + Extractor `tree-sitter-rust`

- **`01_core/entities/rule_traits.rs`**:
  - `ScrutineeForm` (`Path`, `FieldAccess`, `MethodCall`, `Index`, `Literal`, `Tuple`, `Other`).
  - `BodyForm` (`ErrorBarrier`, `EnumPath`, `LiteralNeutral`, `LiteralOther`, `Call`, `EmptyBlock`, `Continue`, `Other`).
  - Structs `DecisionExpr<'a>` e `DecisionArm<'a>` e trait `HasDecisionArms<'a>`.
  - Função `decision_arm_term_for(language)` retornando termos idiomáticos (Rust: `wildcard \`_ =>\``, Python: `\`case _\``, TypeScript: `cláusula \`default:\``, Go: `cláusula \`default:\``, Zig: `"—"`).
- **`01_core/entities/parsed_file.rs`**: campo `decision_exprs: Vec<DecisionExpr<'a>>` e implementação do trait `HasDecisionArms`.
- **`03_infra/rs_parser.rs`**:
  - Extração de `match_expression`, `match_arm`, `match_pattern`, guards e corpos.
  - Distinção precisa de macros: barreiras de erro (`panic!`, `unreachable!`, `bail!`, `todo!`, `unimplemented!`, `compile_error!`) vs macros construtoras (`vec![]`, `hash_map![]` como `BodyForm::LiteralOther`).
  - Slicing seguro de strings respeitando fronteiras UTF-8 (`truncate_str_safe`).

### Fase C — Regras Puras V16–V20, Configuração e Wiring

- **`01_core/rules/wildcard_saturation.rs` (V16)**:
  - Pipeline de 4 filtros: verificação de `is_catchall`, reincorporação do identificador no corpo, exclusão de scrutinee aberto (`MethodCall`, `Index`, `Literal`), exclusão de barreiras de erro.
  - Enum candidato (>= 2 braços partilham prefixo qualificado): classificação de saturação arbitrária (DENY-class), default neutro (WARN-class), delegação (INFO-class), walker parcial (WARN-class).
  - Tabela `[wildcard_exceptions]`: validação de justificativa não-vazia e detecção de spans obsoletos.
- **`01_core/rules/compound_guard.rs` (V17)**: detecta guards com operadores lógicos `&&` e `||`.
- **`01_core/rules/range_pattern.rs` (V18)**: detecta ranges numéricos fora de módulos permitidos (`lexer`, `numbering`, `syntax`).
- **`01_core/rules/or_pattern_alternatives.rs` (V19, info)**: métrica de condensação de alternativas em or-patterns.
- **`01_core/rules/deep_pattern_nesting.rs` (V20, info)**: métrica de aninhamento de padrões `depth > 2` fora de tabelas de tuplas regulares.
- **`04_wiring/main.rs` & `02_shell/cli.rs`**: catálogo SARIF estendido para 21 regras (V0 a V20), flags `--checks` atualizadas para suportar `v16..v20`.

### Fase D — Validação Cruzada, Fixtures e Gate do ADR

1. **Fixture de Mutação (`tests/fixtures/ghost_variant.rs`)**:
   - Valida que um enum de domínio violado por wildcard catch-all mantém exatamente o mesmo diagnóstico após a inserção de uma variante fantasma sem braço nominal explícito.
2. **Não-regressão TypeScript / Python**:
   - Testes unitários comprovam que arquivos TS e Python produzem exatamente 0 violações V16–V20.
3. **Corrida Cruzada sobre `typst-crystalline`**:
   - Comando: `crystalline-lint --checks v16,v17,v18,v19,v20 --format sarif /home/dikluwe/Documentos/Antigravity/typst-crystalline`
   - Resultados gerados:
     - **V16**: 197 ocorrências (16 DENY saturação arbitrária, 126 WARN default neutro, 43 WARN walker parcial, 6 INFO delegação, 6 outros).
     - **V17**: 29 ocorrências (guards compostos).
     - **V18**: 2 ocorrências (range patterns fora de lexer).
     - **V19**: 265 ocorrências (métrica or-pattern).
     - **V20**: 515 ocorrências (métrica aninhamento profundo).
   - Casos emblemáticos DENY corretamente isolados:
     - `01_core/src/entities/ast/expr.rs:910` -> `vec![]`
     - `01_core/src/compiler/math/layout/spacing.rs:72` -> `MathClass::Normal`
     - `01_core/src/compiler/stdlib/transforms.rs:102` -> `1.0`
     - `01_core/src/entities/math_style.rs:50` -> `1.0`
     - `01_core/src/compiler/stdlib/state.rs:218` -> `Content::Empty`
     - `01_core/src/compiler/introspect.rs:1050` -> `UnreferencableKind::Text`

### Fase E — Auditoria Documental

- **`README.md`**: tabela de verificações atualizada com V16–V20; documentação do nível `info`; tabela de ADRs completa até `0016-regras-decisao-mecanica.md`; seção `[wildcard_exceptions]`.
- **`00_nucleo/prompts/rules/wildcard-saturation.md`**: posicionado na árvore de regras e sincronizado.
- **`00_nucleo/adr/0016-regras-decisao-mecanica.md`**: promovido de `PROPOSTO` para `ACEITO`.
- **Trava de Hashes**: execução de `--fix-hashes` garantiu paridade total (`0 drift warnings`).

---

## 3. Estado da Árvore

- **`cargo test --lib`**: 534 testes unitários passando (0 falhas).
- **`cargo test --test fixtures`**: 68 testes de fixtures passando (0 falhas).
- **`crystalline-lint .`**: ✓ 0 erros, 0 avisos (apenas diagnósticos informativos emitidos pelo motor de métricas).

---

## 4. Conclusão e Próximos Passos

O passo 0063 fecha o suporte a decisões mecânicas no `crystalline-lint`. As regras V16–V20 passam a compor o conjunto padrão de análise do linter. O refatoramento e ratchet de severidade (de warning para error) sobre os casos encontrados no `typst-crystalline` constituem a etapa seguinte no repositório correspondente.
