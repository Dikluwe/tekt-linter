# Relatório P0090 — auditoria da partição de resultados do walker

**Data:** 2026-08-24
**Branch:** `codex/audit-walker-result-partition`
**Baseline:** `master@40c374d`
**Veredito:** `READY WITH RESIDUAL AUDIT`

## Resultado

`collect_walker_results` saiu de L4 e passou para
`contracts::file_provider` em L1. A função move sucessos e erros para subsequências
estáveis, preserva multiplicidade e consome o iterador exatamente até o primeiro EOF,
sem consultar `size_hint`. L4 apenas chama a seam.

O adversário L0 congelou `SPEC-GAP` de camada, API e consumo. Após saneamento, o gate
cego congelou RED porque a API pública não existia. A correção ocorreu somente depois
desse RED.

## Rastreabilidade

| L0 | Materialização | Gate |
|---|---|---|
| `prompts/contracts/file-provider.md` | `01_core/contracts/file_provider.rs` | `tests/walker_result_partition_assessment.rs` |
| `prompts/linter-core.md` | `04_wiring/main.rs` | adversário de gravidade |
| ADR-0004 rev. P0090 | chamada L4→L1 | suíte/auto-lint |

## Evidência

- gate instrumentado: 2/2 PASS;
- vazio, Ok, Err, alternância, duplicatas e Unicode/hostil cobertos;
- `next == itens + 1`, zero pós-EOF e zero `size_hint` comprovados;
- `cargo test --all-targets`: 628 unitários, 83 fixtures e integrações PASS;
- assessments 0001–0018: regressão PASS;
- hashes: `Nothing to fix`;
- auto-lint V4/V5/V11/V12: limpo;
- `git diff --check`: PASS;
- L1 sem I/O/configuração e L4 sem loop duplicado.

## Residual

- Nove arquivos Rust receberam somente reparo oficial de header devido ao L0 sistêmico;
  os dois deltas funcionais são exclusivamente `file_provider.rs` e `main.rs`.
- Comportamento de iteradores que violam o contrato Rust após `None` não é tratado;
  a seam para no primeiro EOF conforme norma.
- Descoberta filesystem, exclusões, symlinks, adjacência, rayon e V0 não foram
  reauditados neste lote.
- O warning legado `print_tree` permanece alheio.

P0090 está fechado. Merge, push, instalação e release são ações separadas.
