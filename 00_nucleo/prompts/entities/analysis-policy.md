# Prompt: política e alcance da análise
Hash do Código: c400a853

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
