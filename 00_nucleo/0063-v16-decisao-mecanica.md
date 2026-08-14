# Passo NNN — Implementar V16–V20 (decisões mecânicas) no crystalline-lint

> Número de passo: **a atribuir** pelo mantenedor (série tekt-linter).
> Executor-ready: spec completa; nenhuma decisão de desenho fica em aberto —
> o que não está aqui está no ADR-0016 e no prompt L0 `wildcard-saturation.md`.

**Precede este passo:** ADR-0016 (2026-08-14, PROPOSTO) — regras de decisão
mecânica no tronco, nível `info`, proxies sintácticos, diagnósticos na
nomenclatura da linguagem.
**Objetivo deste passo:** (A.1) V16–V20 implementadas reproduzem o ground truth
syn com concordância ≥ 95% por categoria? (A.2) o linter continua verde sobre si
próprio? (A.3) projectos TS/Python produzem zero violações novas?

**Data:** a preencher pelo executor.
**Commit base:** `<sha>` (a preencher; incluir ADR-0016 commitado em `00_nucleo/adr/`).

---

## Fase A — Nível `info` no núcleo (independente, pode commitar sozinha)

- `01_core/entities/violation.rs`: `Level::Info`; ordenação fatal > error >
  warning > info.
- `02_shell`/`sarif_formatter`: `Info → note`; `--fail-on` aceita apenas
  error|warning e **nunca** falha por info (teste: projecto com só infos → exit 0).
- `03_infra/config.rs`: parsing de `level = "info"` em `[rules]`.
- `readme_prompt.md` + tabela de níveis: actualizar (Prompt L0 em simultâneo —
  código sem prompt actualizado é dívida V5/V6 imediata).

**Conclusão esperada:** núcleo estendido, zero regras a usar info ainda,
`crystalline-lint .` verde, testes do formatter com snapshot `note`.

## Fase B — IR `HasDecisionArms` + extractor tree-sitter-rust

- `01_core/entities/rule_traits.rs`: trait + structs conforme §2 do prompt L0
  (`DecisionExpr`, `DecisionArm`, enums de forma). Puro, sem I/O.
- `03_infra/rs_parser.rs`: query tree-sitter sobre `match_expression`,
  `match_arm`, `match_pattern`, `match_block`; classificação de `body_form` por
  forma do nó (call_expression → Call; block vazio → EmptyBlock; literal →
  Neutral/Other conforme tabela de neutros `false|true|0|0.0|Default::default()`);
  `qualified_prefixes` extraídos de padrões `scoped_identifier`.
- Parsers TS/PY: implementação default vazia (null por omissão).
- `decision_arm_term_for(language)` ao lado de `forbidden_symbols_for(language)`.

**Conclusão esperada:** trait populado só em Rust; testes unitários do parser
com snippets mínimos por forma de corpo (8 formas × pelo menos 1 caso).

## Fase C — Regras puras V16–V20

- `01_core/rules/wildcard_saturation.rs` (V16): pipeline §3 do prompt L0.
- `compound_guard.rs` (V17), `range_pattern.rs` (V18),
  `or_pattern_alternatives.rs` (V19, info), `deep_pattern_nesting.rs` (V20, info).
- Mensagens montadas com termo da linguagem + snippet verbatim (§5 do prompt).
- `crystalline.toml`: entradas `[rules]` V16–V20 com `languages = ["rust"]`,
  e tabela `[wildcard_exceptions]` com validação de justificativa não-vazia e
  detecção de span obsoleto (aviso próprio).
- README: tabela de verificações actualizada.

## Fase D — Validação cruzada (o gate do ADR)

1. Fixture de mutação: `tests/fixtures/ghost_variant.rs` (aceitação §7.1).
2. Corrida de referência sobre typst-crystalline:
   `crystalline-lint --checks v16,v17,v18,v19,v20 .` — comparar com a varredura
   syn (26 DENY / ~18 neutros / 131 walkers / 1 delegação). Tabela de
   concordância por categoria no relatório; cada divergência classificada
   (proxy ajustado **ou** excepção justificada — nunca ignorada).
3. Auto-validação: `crystalline-lint .` no repo do linter — verde, ou
   excepções do próprio linter documentadas em `[wildcard_exceptions]`.
4. Não-regressão: dois projectos de referência (1 TS, 1 PY) → zero V16–V20.

## Fase E — Auditoria documental (última fase, bloqueia o fecho do passo)

O README estava desactualizado (esta ADR é a 0016 e o README só listava até
0009) — ou seja, a deriva documental já aconteceu uma vez. No fim do passo:

1. **README.md**: tabela de verificações com V16–V20 (ID, nome, nível,
   descrição); secção de níveis com `info`; `[rules]` com `languages`;
   `[wildcard_exceptions]`; lista de ADRs completa até 0016; lista de prompts
   em `00_nucleo/prompts/rules/` com as novas entradas.
2. **Varredura de deriva**: cruzar cada secção do README com o código —
   tabela de verificações vs `01_core/rules/`, flags CLI vs `02_shell/cli.rs`,
   estrutura do projecto vs árvore real. Divergências pré-existentes (não
   causadas por este passo) são registadas no relatório e corrigidas no mesmo
   commit se forem mecânicas; se forem substantivas, viram débito declarado.
3. **Critério de fecho**: nenhum documento em `00_nucleo/` referencia regras,
   níveis ou flags que não existem no código, e vice-versa. Se esta auditoria
   encontrar deriva, considerar registar como questão em aberto do ADR-0016
   um «V-doc» futuro (regra do próprio linter que verifica README vs código)
   — a deriva que nos obrigou a renumerar esta ADR não deve poder repetir-se
   em silêncio.

## Critérios de aceitação (todos obrigatórios)

- [ ] (A.1) concordância ≥ 95% por categoria vs ground truth syn
- [ ] (A.2) auto-validação verde com V16–V20 activos
- [ ] (A.3) zero violações em TS/Python
- [ ] (A.4) Fase E concluída: README e `00_nucleo/` sem deriva face ao código
- [ ] fixture de mutação verde
- [ ] snapshot tests: mensagem contém snippet verbatim + termo da linguagem
- [ ] ADR-0016 promovido a ACEITO se (A.1)–(A.3) passarem; senão permanece
      PROPOSTO com as divergências registadas como questões em aberto

## Comandos de validação

```bash
cargo build --workspace --release
cargo test -p crystalline-lint --lib
crystalline-lint .
crystalline-lint --checks v16,v17,v18,v19,v20 --format sarif /caminho/typst-crystalline
```

## Esqueleto do relatório

`tekt-lint-passo-NNN-relatorio.md` no formato da série: Resumo executivo
(resposta a A.1–A.3 com os números de concordância) → Metodologia (commits,
ground truth de referência) → Fase A–D com tabelas → «estado da árvore»
(build/test/lint) → Conclusão (ADR-0016 ACEITO ou divergências pendentes;
débito seguinte: ratchet de V16 para error quando o worklist do
typst-crystalline fechar) → Proveniência.

## Notas para o executor

- Não introduzir `syn`: o IR é tree-sitter (ADR-0001). Se um proxy se revelar
  impossível em tree-sitter, regista-se a divergência no relatório e discute-se
  no ADR — não se troca o parser a meio do passo.
- As regras V16–V20 aplicadas ao **typst-crystalline** produzem o worklist de
  refactor (26 DENY + neutros + walkers). Esse refactor é **outro passo**, no
  outro repo — este passo entrega a ferramenta, não a limpeza.
