# Prompt: planejamento de fix-hashes
Hash do Código: 527fbbd1

Owner exclusivo: `02_shell/fix_hashes.rs`.

Transformar violações V5 válidas em plano bijetivo de atualização, rejeitando conflito,
path hostil e ownership ambíguo antes de writes. Dry-run e execução compartilham o plano.

## Critério observável

Gates de planejamento provam filtro V5, bijeção e ordem; dry-run escreve zero bytes e
apresenta exatamente o mesmo plano aceito pela execução.
