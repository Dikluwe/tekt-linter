# Prompt: carregamento de snapshots de refinamento
Hash do Código: 11485d85

Owner exclusivo: `03_infra/refinement_snapshot.rs`.

Compor leitura por OID e extração em snapshots before/after, mantendo identidade do objeto,
budgets e causas de indisponibilidade. Não colapsar `UNKNOWN` em vazio.

## Critério observável

Gates preservam OIDs e fontes before/after, distinguem missing/unreadable/opaque e propagam
orçamento sem fabricar snapshot vazio.
