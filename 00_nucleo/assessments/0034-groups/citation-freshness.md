# Classificador — citation-freshness

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `01_core/contracts/citation_freshness.rs` | tipos de resultado, razões e porta `CitationFreshnessResolver` |
| `03_infra/citation_freshness.rs` | resolução filesystem/Git, confinamento, budgets e mapeamento de falhas |

`CONTEXT`: ambos participam do mesmo fluxo vertical. `SHARED-CLAIM`: nenhuma necessária;
a direção port→adapter já pertence à arquitetura Tekt. Sem contradição.
