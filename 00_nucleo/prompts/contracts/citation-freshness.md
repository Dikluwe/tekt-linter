# Prompt: contrato de frescura de citação
Hash do Código: PENDING_P0106

## Owner

`01_core/contracts/citation_freshness.rs`, exclusivamente.

## Instrução

Definir em L1 os estados fechados `Valid`, `Stale(reason)` e `Unknown(reason)`, a porta
`CitationFreshnessResolver` e o fallback fail-closed. Razões devem preservar a causa
sem expor filesystem, Git ou erros concretos de L3.

## Restrições

- não realizar I/O nem importar tipos de infraestrutura;
- não reduzir estado triádico a booleano;
- `UnknownCitationFreshness` nunca autoriza silêncio de uma referência.

## Critérios

Tipos são clonáveis/comparáveis; mocks puros implementam a porta; ausência de adapter
permanece observável como `Unknown`.
