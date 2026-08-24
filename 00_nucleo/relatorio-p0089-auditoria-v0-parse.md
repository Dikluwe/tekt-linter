# Relatório P0089 — auditoria segregada V0/PARSE

**Data:** 2026-08-24
**Branch:** `codex/audit-v0-parse-projections`
**Baseline:** `master@cc1924b`
**Veredito:** `READY WITH RESIDUAL AUDIT`

## Resultado

A política erro→violação saiu de L4 e passou a um projetor puro L1 em
`01_core/rules/infrastructure_error.rs`. L4 apenas chama a API; L3 continua produzindo
`SourceError`/`ParseError` e L2 recebe a violação pronta para apresentação.

O gate cego revelou dois REDs: API pública ausente e `SourceError` sem os derives
normativos `Clone + PartialEq + Eq`. Ambos foram corrigidos somente após o gate ser
congelado. O confronto também corrigiu o drift histórico das três mensagens inglesas,
restaurando os textos portugueses prescritos pelo L0.

## Rastreabilidade

| L0 | Materialização | Gate |
|---|---|---|
| `prompts/rules/infrastructure-error.md` | `01_core/rules/infrastructure_error.rs` | `tests/infrastructure_error_projection_assessment.rs` |
| `prompts/contracts/file-provider.md` | `01_core/contracts/file_provider.rs` | mesmo gate |
| `prompts/linter-core.md` | `04_wiring/main.rs` | adversário de gravidade |

## Evidência

- gate V0/PARSE: 7/7 PASS;
- quatro modalidades, IDs, níveis, textos, `Cow::Owned` e posições cobertos;
- `cargo test --workspace --quiet`: 628 unitários, 83 fixtures e integrações PASS;
- `cargo run --quiet -- . --fix-hashes --dry-run`: `Nothing to fix`;
- auto-lint V4/V5/V11/V12: limpo;
- busca de I/O/config no projetor L1: limpa;
- `git diff --check` contra baseline: PASS;
- adversário final: todos os bloqueios fechados.

## Residual

- `0:0` significa posição indisponível apenas nas quatro modalidades deste contrato;
  uma convenção global de localização ausente permanece fora do lote.
- O warning legado `print_tree` em `ts_parser.rs` permanece alheio.
- Parsers, walker, SARIF, CLI, ordenação e exit code não foram reauditados.

P0089 está fechado. Merge, push, instalação e release permanecem ações separadas.
