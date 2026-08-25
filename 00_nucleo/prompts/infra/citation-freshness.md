# Prompt: adapter de frescura de citação
Hash do Código: PENDING_P0106

## Owner

`03_infra/citation_freshness.rs`, exclusivamente.

## Instrução

Implementar `CitationFreshnessResolver` sobre filesystem/Git com raiz confinada,
orçamento de bytes, leitura por handle e classificação explícita de stale/unknown.

## Restrições

- rejeitar absoluto, `..`, symlink e raiz inválida de modo fail-closed;
- nunca transformar falha externa ou corrida em `Valid`;
- não alterar a taxonomia declarada pela porta L1.

## Critérios

Linha válida e não vazia é `Valid`; arquivo/linha ausente é `Stale`; escape, orçamento,
UTF-8, I/O e mutação concorrente ficam observáveis em `Unknown(reason)`.
