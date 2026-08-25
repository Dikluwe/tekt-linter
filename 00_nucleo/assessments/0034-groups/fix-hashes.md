# Classificador — fix-hashes

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `02_shell/fix_hashes.rs` | plano L2, porta transacional, resultados e apresentação de hashes |
| `02_shell/update_snapshot.rs` | plano L2 e apresentação de atualização de snapshots V6 |
| `03_infra/hash_writer.rs` | transformação byte-safe e escrita/rollback de pares hash |
| `03_infra/snapshot_writer.rs` | persistência atômica do bloco de snapshot |

`CONTEXT`: família de reparadores. Atomicidade concreta pertence aos writers; planejamento a
cada use-case. Não criar núcleo genérico que force contratos diferentes. Sem contradição.
