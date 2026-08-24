# Assessment 0012 — V4 Impure Core e V14 External Type In Contract

**Estado:** PASS após saneamento P0083
**Data:** 2026-08-24
**Passo:** P0082
**Alvos:** `impure_core.rs`, `external_type_in_contract.rs`

## Método

Dois agentes independentes receberam L0 por caminho, seção e SHA-256 e construíram gates
sem ler os alvos, testes anteriores, fixtures, lab ou histórico Git. Depois do
congelamento, um terceiro papel confrontou L0, gates e produção. Nenhum papel alterou
produção.

Insumos validados:

- V4: `impure-core.md`, SHA-256
  `efc1998d377cedd2b698b9be0ea138b52cffa7303fea7a14ba8bb67256e8e2b4`;
- V14: `external-type-in-contract.md`, SHA-256
  `b81bd7281e09851e7586d22c561d8ac0e94f738467d460c148d28fdec52b0338`.

## V4

O gate cobriu nominalmente 12 símbolos Rust, 21 TypeScript e 14 Python, igualdade,
prefixos `::`/`.`, linguagem desconhecida, todas as camadas fora de L1, ordem,
multiplicidade, mensagem, nível, localização e `Cow` borrowed/owned.

Resultado: 5/6. A falha não pertence à produção. O gerador declarou `os.path_near` como
permitido, mas a tabela normativa também contém `os`; portanto o valor continua proibido
por `os.`. O gate precisa avaliar o candidato contra a união completa da tabela.

O confronto encontrou um `SPEC-GAP`: a produção contém tabelas para C, C++, Zig, Go,
Java e Elixir, ausentes do L0 autorizado. Também falta uma propriedade explícita de
isolamento entre tabelas de linguagens.

## V14

Resultado: 5/7. Os dois REDs são a mesma divergência concreta. O L0 constrói a mensagem
com `package_name(import.path)`, enquanto a produção usa `import.path`. Assim, o contrato
espera `'comemo'` e a produção informa `'comemo::Tracked'`, por exemplo. Ordem,
multiplicidade e localização estão corretas.

O confronto encontrou ainda:

1. **Granularidade da whitelist:** L0 descreve `HashSet<String>` por pacote; entidade e
   produção usam `HashMap<String, HashSet<String>>` e autorização opcional por item.
2. **Imports de teste:** `check_test_imports`/`is_test_origin` funciona nos dois estados,
   mas está somente no histórico do L0, não na especificação normativa.
3. **Isenções:** produção isenta `super` e `crate`, além de `std`, `core` e `alloc` do L0.
   A intenção é válida, porém não está normatizada.
4. **npm scoped:** L0 e produção reduzem `@scope/pkg` a `@scope`, embora o nome do pacote
   seja `@scope/pkg`. É defeito do L0 reproduzido na produção e consolidado pelo gate.

## Contaminação observada

Os verificadores viram módulos `#[cfg(test)]` embutidos em alguns contratos de entidades
autorizados. Eles não usaram essas expectativas e não leram testes dos alvos. Isso não
explica nenhum resultado, mas mostra que execuções futuras devem recortar contratos antes
de entregá-los aos agentes, em vez de autorizar arquivos inteiros.

## Condições antes do merge

1. Corrigir o gerador de near misses do gate V4 e obter verde.
2. Decidir e alinhar a mensagem V14 entre L0 e produção.
3. Normatizar as linguagens adicionais de V4 e a granularidade por item de V14.
4. Incorporar imports de teste e isenções ao corpo normativo do V14.
5. Corrigir a semântica de pacote npm scoped e adicionar gate específico.
6. Repetir os gates e o confronto após as correções, antes do merge.

Este assessment registra diagnóstico, não saneamento. Nenhum merge, instalação ou release
foi realizado.

## Fechamento P0083

As seis condições foram executadas. O gate V4 agora descarta falsos near misses capturados
por outra entrada da mesma tabela, cobre nominalmente as nove linguagens materializadas e
prova isolamento entre elas. Resultado: 7/7.

O L0 V14 incorporou a whitelist por pacote/item, os dois estados de imports de teste, as
isenções `crate`/`super` e a preservação de pacote npm scoped. A produção passou a emitir
o pacote extraído na mensagem e a reconhecer `@scope/pkg/subpath` como pacote
`@scope/pkg`. Resultado do gate: 9/9.

A suíte global passou com 628 testes unitários, 83 fixtures e todos os gates de integração.
Hashes ficaram estáveis e o auto-lint V1/V5/V7 não encontrou violações. Assim, os
RED/SPEC-GAP deste assessment estão encerrados; a recomendação de bloqueio ao merge foi
removida.
