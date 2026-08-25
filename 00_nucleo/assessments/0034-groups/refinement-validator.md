# Classificador — refinement-validator

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `01_core/entities/refinement.rs` | domínio, políticas e comparação pura de refinements |
| `02_shell/refinement.rs` | comandos, formatos e exit codes |
| `03_infra/git_refinement.rs` | captura confinada de revisões Git e budgets |
| `03_infra/refinement_extractor.rs` | contrato de observáveis e extração/serialização |
| `03_infra/refinement_snapshot.rs` | parsing estrito de snapshots e relações |

O fluxo vertical é contexto. Cada fronteira possui ameaças e responsabilidades próprias;
nenhum núcleo amplo nesta migração. Sem contradição.
