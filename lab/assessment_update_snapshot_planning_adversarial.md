# Parecer adversarial P0092 — update-snapshot

## Adversário A

Veredito: `SPEC-GAP / BLOCKED`. G1–G5 confirmados; B1/B2 seriam `GATE-DEFECT` antes de
API, tipos, precedência, unreadable e dry-run serem publicados em L0.

## Adversário D

Veredito de planejamento/execução e arquitetura: PASS. Veredito global: `BLOCKED`.

- ordem segregada dos commits e RED causal confirmados;
- oito hashes L0 e dois hashes de gate conferidos;
- G1–G5 fechados;
- B1 3/3 e B2 2/2;
- enums eliminam estados inválidos;
- cardinalidade, path exato, dry-run, unreadable e continuidade confirmados;
- L2 sem I/O/import L3; L3/L4 mantêm suas responsabilidades;
- delta produtivo restrito a `update_snapshot.rs`; outras sete mudanças são hashes;
- regressão, auto-lint, hashes e formatação dirigida passaram.

Bloqueio: o wiring usa `format_plan` no dry-run; esse formatter omite o snapshot exigido
pelo L0. `format_results` também rotularia `DryRun` como “Updated”. Falta gate black-box
da apresentação realmente consumida e correção posterior ao RED congelado.

Residual aceito: Git não prova sozinho a independência cognitiva de dois gates congelados
no mesmo commit por autor genérico; a segregação operacional e o conteúdo não mostram
contaminação ou oráculo comum.
