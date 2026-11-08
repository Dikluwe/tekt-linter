# Prompt: carregamento estrito de configuração
Hash do Código: b3962033

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

## Revisão 2026-11-08 — seleção por projeto

Aceitar `enabled = true/false` opcional para cada regra V0–V27 em `[rules]`.
Ausência preserva a seleção atual. A seleção parte de --checks (ou seus defaults),
a configuração por regra sobrescreve essa seleção e --no-drift/--no-stale têm
precedência final. true também ativa regras opt-in. false desativa a regra mesmo
se sua severidade for Fatal; severidade e seleção são conceitos distintos.
IDs, tipos, níveis e filtros continuam validados mesmo em regras desativadas.
V0/V8/V10 mantêm nível Fatal quando ativas, mas podem ser desativadas individualmente.

O alcance informa regras efetivas, desativadas e configuração usada. Quando houver
`enabled` explícito, incluir esse alcance automaticamente no lint text/SARIF;
quiet permanece silencioso. Desativar V0 suprime seu diagnóstico de regra, mas
leitura/resolução incompleta continua erro operacional (saída 2), sem declaração
de análise completa. Raiz/configuração inválida não é uma propriedade opcional.
Reparos continuam obedecendo bloqueios intrínsecos de integridade antes da política.
A seleção não comprova supervisão humana, nem altera restrições Rust do consumidor.
