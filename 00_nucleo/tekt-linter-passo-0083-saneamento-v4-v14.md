# Passo operacional 0083 — saneamento de V4 e V14

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; saneamento verde
> **Branch:** `codex/segregated-materialization`
> **Base:** Assessment 0012 / P0082

## Objetivo

Eliminar os RED/SPEC-GAP encontrados na análise segregada de V4/V14 e devolver o branch
a um estado integralmente verde antes do merge.

## Decisões congeladas

1. O gate V4 deve calcular near misses contra a união da tabela da linguagem, não contra
   uma entrada isolada.
2. As tabelas já materializadas para C, C++, Zig, Go, Java e Elixir passam a integrar o
   L0 de V4; o gate deve cobri-las nominalmente e provar isolamento entre linguagens.
3. V14 mantém a evolução de whitelist por pacote e opcionalmente por item: conjunto vazio
   autoriza todo o pacote; conjunto não vazio autoriza somente os itens declarados.
4. `check_test_imports`, `is_test_origin`, `crate` e `super` passam ao corpo normativo.
5. A mensagem V14 usa o nome completo do pacote extraído, não o path do item.
6. Pacotes npm scoped preservam `@scope/pkg`; subpaths posteriores não integram o nome do
   pacote.

## Execução

1. Atualizar os L0 de V4/V14 e seus critérios verificáveis.
2. Corrigir o gate V4 e ampliar os gates com os gaps congelados.
3. Alterar somente o mínimo necessário na produção V14.
4. Reparar hashes de linhagem pelo linter oficial.
5. Rodar gates dirigidos, suíte global, auto-lint, hashes e verificação de diff.
6. Registrar relatório e estado final do assessment 0012.

## Critério de fechamento

Todos os gates e a suíte global devem passar, L0 e produção devem concordar nos pontos
congelados e não pode restar RED/SPEC-GAP do assessment 0012. Não fazer merge, instalação
ou release.

## Resultado

- V4: gate corrigido para a união normativa, nove tabelas cobertas e isolamento entre
  linguagens provado; 7/7.
- V14: contrato por pacote/item, imports de teste, isenções intra-crate e npm scoped
  normatizados; mensagem alinhada ao pacote; 9/9.
- suíte global: PASS; 628 unitários, 83 fixtures e todos os gates de integração verdes;
- hashes: estáveis; auto-lint V1/V5/V7: zero violações; formatação e diff: PASS.

O assessment 0012 não possui RED/SPEC-GAP residual dentro do escopo congelado. Nenhum
merge, instalação ou release foi realizado.
