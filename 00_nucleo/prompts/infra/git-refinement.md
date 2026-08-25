# Prompt: sessão Git confinada para refinamento
Hash do Código: 63849783

Owner exclusivo: `03_infra/git_refinement.rs`.

Resolver refs uma vez, operar por OID completo e ler objetos sob budgets/timeout, com
contenção de processo e estados hostis fail-closed. Nunca interpretar semântica L1.

## Critério observável

Testes hostis cobrem timeout, escape, symlink, objetos transitórios e lifecycle; refs são
resolvidas uma vez e erros externos retornam causa tipada sem checkout.
