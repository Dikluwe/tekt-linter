# Passo operacional 0076 — saneamento da ordem pública de V11

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0008, commits `86a7a15` e `04d3017`

## Objetivo

Fechar o RED de V11 sem alterar sua semântica de pertinência. A mesma combinação de
traits declaradas, implementadas e satisfeitas por blanket impl deve produzir o mesmo
vetor completo de violações, independentemente de inserção ou semente do `HashSet`.

## Decisões congeladas

1. O conjunto pendente permanece exatamente
   `all_declared_traits - (all_implemented_traits ∪ all_blanket_impl_traits)`.
2. Materializar as referências pendentes, ordená-las textualmente por `&str` e somente
   então criar as violações. Não trocar os conjuntos globais nem ordenar por mensagem.
3. A comparação é a ordem total nativa de bytes/Unicode de Rust para `str`; não aplicar
   locale, lowercase, normalização NFC/NFD ou equivalência visual.
4. Duplicatas continuam neutralizadas pelos conjuntos de entrada. Uma trait presente
   simultaneamente em implementação concreta e blanket continua satisfeita uma vez.
5. `rule_id`, nível injetado, mensagem, location e cardinalidade de V11 permanecem
   inalterados. V2, V8 e V10 não serão modificadas.
6. O prompt causal `prompts/rules/dangling-contract.md` deve absorver a ordem canônica e
   os hashes devem ser atualizados exclusivamente pelo fluxo oficial `--fix-hashes`.

## Segregação e execução

- A implementa a partir deste passo e do prompt causal, sem ler assessment 0008, gate
  ou artefatos adversariais.
- B endurece e ativa o gate congelado sem ler a produção modificada.
- C revisa após o primeiro gate verde sem ler testes de B.
- O orquestrador executa suíte completa, assessments, auto-lint e `git diff --check`.

## Critérios de fechamento

- assessment 0008: 6/6, zero ignorados;
- múltiplas construções do mesmo conjunto produzem vetor byte a byte idêntico;
- NFC/NFD, caixa e prefixos próximos continuam identidades distintas e ordenadas;
- adversário declara **NÃO REABRIR** ou apresenta novo RED reproduzível;
- testes anteriores e fixtures permanecem verdes.

## Parada

Registrar relatório final. Não fazer merge, instalação ou release. Qualquer mudança de
semântica além da ordenação exige novo contrato e interrompe este passo.
