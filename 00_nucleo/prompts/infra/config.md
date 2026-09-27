# Prompt: carregamento estrito de configuração
Hash do Código: fa13501f

Owner exclusivo: `03_infra/config.rs`.

Desserializar `crystalline.toml`, aplicar defaults explícitos e projetar configuração para
tipos consumíveis pelo wiring. Chave/tipo inválido é erro, nunca fallback silencioso.

## Critério observável

Config mínima aplica defaults documentados; chave desconhecida, colisão e tipo inválido
falham, enquanto projeções por regra preservam o valor declarado.

## Revisão 2026-09-26

Rejeitar chaves desconhecidas em tabelas tipadas, IDs de regra fora de V0–V27, níveis e linguagens inválidos. V0/V8/V10 não admitem rebaixamento nem filtro de linguagem. Regras globais V7/V11/V15/V26 não aceitam filtro de linguagem. Tabelas legadas project/languages aceitam apenas root=".", gramática embutida correspondente e enabled=true; opções não implementadas falham explicitamente. Projetar política para L1, sem executar regras.

## Revisão 2026-09-26 — ausência versus erro de configuração

Expor load_optional: somente ausência confirmada da entrada de configuração retorna None. Inspecionar com symlink_metadata para distinguir link quebrado de ausência; erros de inspeção, leitura ou parsing retornam Err. Entrada existente deve passar pelo carregamento estrito; link válido mantém o comportamento de leitura existente. Nenhum erro de filesystem vira default.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
