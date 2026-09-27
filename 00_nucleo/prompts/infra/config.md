# Prompt: carregamento estrito de configuração
Hash do Código: 34fcf88d

Owner exclusivo: `03_infra/config.rs`.

Desserializar `crystalline.toml`, aplicar defaults explícitos e projetar configuração para
tipos consumíveis pelo wiring. Chave/tipo inválido é erro, nunca fallback silencioso.

## Critério observável

Config mínima aplica defaults documentados; chave desconhecida, colisão e tipo inválido
falham, enquanto projeções por regra preservam o valor declarado.

## Revisão 2026-09-26

Rejeitar chaves desconhecidas em tabelas tipadas, IDs de regra fora de V0–V27, níveis e linguagens inválidos. V0/V8/V10 não admitem rebaixamento nem filtro de linguagem. Regras globais V7/V11/V15/V26 não aceitam filtro de linguagem. Tabelas legadas project/languages aceitam apenas root=".", gramática embutida correspondente e enabled=true; opções não implementadas falham explicitamente. Projetar política para L1, sem executar regras.
