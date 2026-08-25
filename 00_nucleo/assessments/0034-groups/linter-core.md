# Classificador — linter-core

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `01_core/contracts/mod.rs` | fachada nominal dos contratos L1 |
| `01_core/rules/mod.rs` | fachada nominal das regras L1 |
| `02_shell/mod.rs` | fachada nominal dos use-cases L2 |
| `02_shell/n16_summary.rs` | coleta e apresentação do resumo N16 |
| `03_infra/config.rs` | schema/configuração estrita e projeções por regra |
| `03_infra/elixir_parser.rs` | parser Elixir para IR canônica |
| `03_infra/go_parser.rs` | parser Go para IR canônica |
| `03_infra/java_parser.rs` | parser Java para IR canônica |
| `03_infra/mod.rs` | fachada nominal da infraestrutura L3 |
| `04_wiring/main.rs` | composição, pipeline paralelo e dispatch CLI |

`linter-core.md` é contexto arquitetural amplo, não contrato compartilhável atômico. ADRs e
contratos de portas já governam relações comuns. Reescrever todos os owners nominalmente;
não criar “núcleo universal”. Sem contradição.
