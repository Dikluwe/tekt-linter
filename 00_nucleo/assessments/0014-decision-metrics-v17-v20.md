# Assessment 0014 — classificadores mecânicos V17–V20

**Estado:** CONGELADO PARA TRIAGEM SEGREGADA  
**Data:** 2026-08-24  
**Alvos:** `compound_guard`, `range_pattern`, `or_pattern_alternatives`,
`deep_pattern_nesting`  
**Baseline:** `cea1e7013b8c6e6f61dbcef69a4d52a392e628de`  
**L0 autorizado:** `00_nucleo/prompts/rules/wildcard-saturation.md`  
**SHA-256 L0:** `66c502de44ef21880a68fe798c74ef5f3a91b9fe7dd3e925c2722d99d25f6800`

## Natureza e limite

Assessment retroativo, portanto `assessed`, não `sealed`. O lote foi escolhido antes da
leitura adversarial da produção por ter superfície L1 pura, um único contrato de entrada
(`HasDecisionArms`) e nenhuma dependência de filesystem, Git, rede ou serialização. A
triagem não autoriza mudança funcional até que RED/SPEC-GAP seja congelado.

O verificador recebe somente este assessment e o L0 acima, preso pelo hash exato. Não
pode ler produção, testes existentes, histórico ou relatórios do implementador antes de
congelar o gate. Insumo ausente ou hash divergente bloqueia como `SPEC-GAP`.

## Alegações congeladas

1. **Escopo comum:** V17–V20 retornam vazio para toda linguagem diferente de Rust e
   preservam a ordem de expressão e braço do IR para diagnósticos emitidos.
2. **V17:** em Rust, emite exatamente um `Warning` por braço quando e somente quando
   `has_guard && guard_is_compound`; a evidência contém o snippet, path, linha e coluna.
3. **V18:** em Rust, emite exatamente um `Warning` por braço com
   `pattern_is_range`, exceto em módulos de `lexer`, `numbering` ou `syntax`; a isenção
   deve reconhecer componentes de path/módulo, não substrings acidentais de nomes.
4. **V19:** em Rust, emite exatamente um `Info` quando e somente quando
   `or_alternatives > 1`; mensagem e evidência preservam contagem, snippet e location.
5. **V20:** em Rust, emite exatamente um `Info` por braço com `pattern_depth > 2`, salvo
   quando a expressão é uma tabela regular de tuplas sobre os mesmos tipos; profundidade,
   snippet e location permanecem observáveis.
6. **Isolamento:** cada regra depende somente dos campos que seu contrato enumera;
   variar campos irrelevantes não altera cardinalidade, severidade, mensagem ou location.
7. **Fronteiras:** limiares são exatos (`1/2` em V19 e `2/3` em V20), nenhum overflow,
   path Unicode ou coleção vazia causa panic ou diagnóstico espúrio.

## Gate mínimo

O gate black-box deve cobrir todas as alegações e conter, no mínimo:

- matriz Rust/não-Rust e controles positivos/negativos por regra;
- tabela-verdade completa dos dois booleanos de V17;
- ataques de componente versus substring e case em V18;
- limiares e valores máximos representáveis em V19/V20;
- tabelas de V20 com tuplas homogêneas, heterogêneas, catch-all e quase-tabelas;
- duas expressões e múltiplos braços para ordem/cardinalidade;
- mutação sistemática de campos irrelevantes e preservação integral da evidência.

O resultado de cada propriedade é `PASS`, `RED`, `SPEC-GAP` ou `GATE-DEFECT`. Qualquer
RED/SPEC-GAP é registrado antes de correção. Um gate que derive expectativas da produção
é inválido e deve ser refeito por produtor segregado.

## Fechamento exigido

Antes de qualquer merge: gate congelado, revisão adversarial independente, correções
autorizadas somente por achados congelados, reexecução do gate e suíte global, hashes em
modo seco, auto-lint, `rustfmt --check`, `git diff --check`, relatório e assessment em
estado `READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Este branch não executa merge.
