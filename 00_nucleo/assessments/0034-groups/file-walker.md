# Classificador — file-walker

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `03_infra/prompt_io.rs` | confinamento, leitura limitada, substituição atômica e metadata byte-safe |
| `03_infra/walker.rs` | enumeração de fontes, exclusões, layers e testes adjacentes |

Ambos lidam com paths, mas operações e políticas são diferentes. Reutilizam contratos de
config/camada; nenhum núcleo novo é necessário. Sem contradição.
