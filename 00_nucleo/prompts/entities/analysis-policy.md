# Prompt: política e alcance da análise
Hash do Código: 6e101db1

Owner exclusivo: `01_core/entities/analysis_policy.rs`.

L1 puro. Validar IDs V0–V27 e linguagens; representar níveis/filtros opcionais.
Aplicar nível configurado e filtro por linguagem aos achados normais, preservando
Fatal e V0/V8/V10. Regras globais sem linguagem não podem ser filtradas.
Resumir arquivos parseados por linguagem e imports com destino Unknown, sem afirmar
que Unknown é necessariamente erro (também inclui externos). Não executar I/O.

Testes verificam política de nível, filtro, proteção fatal e resumo com dados explícitos.
Revisão inicial: 2026-09-26, autorização do mantenedor para configuração e alcance.

## Revisão 2026-09-26 — completude dos reparos

Representar contadores tipados de SyntaxError, UnsupportedLanguage e EmptySource, com agregação associativa. A política pura bloqueia reparos quando há diagnóstico V0 ou PARSE, independentemente de severidade ou seleção de checks. Testar cada variante e agregação.

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
