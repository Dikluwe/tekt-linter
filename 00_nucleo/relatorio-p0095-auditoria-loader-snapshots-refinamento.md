# Relatório P0095 — auditoria do loader de snapshots e contratos de refinamento

**Data:** 2026-08-25
**Branch:** `codex/audit-refinement-snapshot-loader`
**Baseline:** `4408649`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Escopo

O lote auditou exclusivamente o loader L3 de snapshots JSON e relações TOML de
refinamento: schemas fechados, identidade textual, duplicatas e conflitos, limites,
classes de erro e leitura explícita de arquivos. Comparação L1, apresentação L2 e
coordenação L4 permaneceram consumidores separados.

## Matriz causal

| Papel | Evidência |
|---|---|
| A — L0 | dez hashes validados; G1–G7 classificados e saneados |
| B1 — snapshot | `583db2ae…e3a3`, RED de API → GREEN 8/8 |
| B2 — contrato | `ac2c6057…5ebb`, RED 1/10 → GREEN 10/10 |
| C — produção | `e54cc2c`, seguido das correções `3f45f22` e `3ba5f97` |
| D — fechamento | dois REDs causais corrigidos; parecer final PASS |

O RED de integração C1 mostrou que `refine-revisions` usa TOML combinado com
`[[observable]]` e `[[relation]]`. O L0 foi saneado para reconhecer a extensão sem
materializá-la no loader de relações. D encontrou depois precedência incorreta entre
`schema:` e `limit:`. A correção final aplica 64 KiB a toda chave/string JSON e TOML antes
da semântica, com regressões próprias sem modificar os gates congelados.

## Arquitetura Tekt

| Camada | Responsabilidade confirmada |
|---|---|
| L1 | tipos, relações e comparação de refinamento |
| L2 | mensagens e política de exit status |
| L3 | leitura read-only, parse, schema e limites |
| L4 | seleção de fluxo e coordenação de dependências |

O loader não emite `PRESERVED`, `VIOLATED` ou `UNKNOWN`, não executa comandos e não
escreve artefatos.

## Validação

- gates P0095: B1 8/8 e B2 10/10 PASS, hashes preservados;
- workspace: 630 unitários e todas as integrações/fixtures PASS;
- regressões D: strings excessivas em estado, razão, payload proibido, campo JSON
  desconhecido e campo TOML proibido PASS;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- reparador V5 dry-run: `Nothing to fix`;
- `rustfmt` dirigido e `git diff --check`: PASS.

Resíduos não bloqueantes: classificação JSON usa sentinel textual interno; existe janela
TOCTOU entre inspeção de componentes e `open`; tamanho/mtime não detectam toda alteração
concorrente equivalente; `cargo fmt --all --check` expõe drift histórico fora do lote.
Nenhum merge ou push foi realizado.
