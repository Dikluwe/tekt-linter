# Assessment 0002 — entidades puras de baixo risco presumido

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `layer.rs`, `violation.rs`, `project_index.rs`

## Hipótese

Este lote deve produzir zero achados: contém tipos de domínio simples e agregação em
memória, sem parser, filesystem, Git, rede ou serialização externa. Um RED legítimo
reclassifica entidades aparentemente mecânicas e aumenta a prioridade de módulos que
fazem redução paralela ou ordenação.

## Alegações sob teste

1. Variantes de camada, linguagem e severidade são distintas e completas para os
   valores públicos atualmente suportados.
2. A ordenação de severidade preserva `Info < Warning < Error < Fatal`.
3. Clone e igualdade preservam integralmente violações e locations borrowed/owned.
4. `ProjectIndex::merge_local` e `ProjectIndex::merge` não perdem nem inventam dados.
5. A redução é associativa, comutativa e possui identidade vazia, inclusive para
   prompts, aliens e as três famílias de traits.
6. Duplicatas não alteram os conjuntos; ordem e particionamento da redução paralela
   não alteram o significado observável do índice.

## Gate curto

O adversário pode propor no máximo quatro propriedades/mutações. O verificador escreve
testes separados sem ler a implementação. Durante a triagem, produção não é alterada.
Resultados possíveis: `PASS`, `RED` ou `SPEC-GAP`.

## Resultado da triagem

O gate independente terminou com três propriedades verdes e uma vermelha:

- variantes, severidades, igualdade e clone integral passaram;
- transporte dos cinco campos e identidades vazias passaram;
- duplicatas não alteraram os quatro conjuntos públicos;
- permutar as mesmas contribuições alterou a ordem de `alien_files`.

O último caso contradiz a documentação pública de `ProjectIndex::merge` como operação
comutativa: os conjuntos mantêm o mesmo significado, mas o `Vec` de aliens expõe
`[a,b,c]` ou `[c,b,a]` conforme a ordem da redução. Em execução paralela, isso permite
um índice observavelmente não determinístico antes da ordenação posterior.

Um segundo RED inicial exigia idempotência também para aliens. Ele foi descartado como
erro do teste: o contrato congelado prometia idempotência somente para conjuntos. O
verificador corrigiu sua propriedade sem consultar produção, e o gate convergiu para
`3 PASS / 1 RED`.

O achado reclassifica reduções “mecânicas” que misturam sets e sequências. A família
deve receber triagem gradual, mas este lote não autoriza ainda alterar produção.
