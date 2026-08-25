# Prompt: carregamento estrito de configuração
Hash do Código: 6686bac6

Owner exclusivo: `03_infra/config.rs`.

Desserializar `crystalline.toml`, aplicar defaults explícitos e projetar configuração para
tipos consumíveis pelo wiring. Chave/tipo inválido é erro, nunca fallback silencioso.

## Critério observável

Config mínima aplica defaults documentados; chave desconhecida, colisão e tipo inválido
falham, enquanto projeções por regra preservam o valor declarado.
