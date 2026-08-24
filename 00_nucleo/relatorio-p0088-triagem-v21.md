# Relatório P0088 — auditoria segregada V21

**Data:** 2026-08-24  
**Branch:** `codex/plan-v21-segregated-audit`  
**Veredito:** `READY WITH RESIDUAL AUDIT`

## Resultado

V21 voltou a respeitar a arquitetura Tekt. O classificador L1 é puro e recebe a porta
`CitationFreshnessResolver`; o adapter filesystem vive em L3; L4 somente instancia e
injeta `FsCitationFreshnessResolver`. `stale` e `unknown` geram Warning explícito e
nunca silenciam uma ocorrência.

O confronto inicial encontrou I/O direto em L1, matching por substring, citações vazias
silenciando e ausência de estado unknown observável. Os gates cegos congelaram esses
REDs. A primeira correção revelou ainda um RED adversarial TOCTOU; ele foi fechado após
novo L0 e gates concorrentes, por travessia Linux ancorada em handles com `O_NOFOLLOW`.

## Rastreabilidade

| L0 | Materialização | Gate |
|---|---|---|
| `prompts/unsourced-constant.md` | `01_core/rules/unsourced_constant.rs` | `tests/hardcoded_contextual_value_v21_assessment.rs` |
| `prompts/contracts/citation-freshness.md` | `01_core/contracts/citation_freshness.rs` | B1/B2 |
| mesma porta L0 | `03_infra/citation_freshness.rs` | `tests/citation_freshness_adapter_assessment.rs` |
| `prompts/linter-core.md` | `04_wiring/main.rs` | auto-lint e suíte |

## Evidência de fechamento

- gate B1: 9/9 PASS;
- gate B2: 9/9 PASS;
- ataques concorrentes: 20.000 trocas symlink e 10.000 remoções, nunca `Valid`;
- `cargo test --workspace --quiet`: 628 unitários, 83 fixtures e integrações PASS;
- regressão V22: PASS;
- `git diff --check`: PASS;
- busca mecânica: zero I/O em V21/porta L1;
- adversário final: todos os REDs e gate-defects fechados.

## Auditoria residual

- Em não-Linux o adapter retorna `Unknown(Io)`; portabilidade funcional fica em lote
  próprio, preservando fail-closed.
- Mutação no mesmo handle é observada por tamanho/mtime; reforço de fingerprint fica
  residual e não reabre confinamento.
- Direção/associação do parser, janela de citações e orçamento L4 configurável permanecem
  fora do escopo já declarado.
- O warning legado `print_tree` em `ts_parser.rs` é alheio ao lote.

P0088 está fechado. Merge, push, instalação e release continuam ações separadas.
