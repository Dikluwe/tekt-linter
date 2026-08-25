# Classificador — prompt-reader

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `01_core/contracts/prompt_reader.rs` | contrato de existência e leitura de hash de prompt |
| `03_infra/prompt_reader.rs` | leitura confinada, hash efetivo e cache de resultados |

`CONTEXT`: fluxo port/adaptor. Nenhuma claim nova em núcleo; limites e I/O pertencem ao
adapter, forma da porta pertence ao contrato. Sem contradição.
