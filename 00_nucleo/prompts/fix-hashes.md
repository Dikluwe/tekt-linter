# Prompt: planejamento de fix-hashes
Hash do Código: 9083fe0d

Owner exclusivo: `02_shell/fix_hashes.rs`.

Transformar violações V5 válidas em plano bijetivo de atualização, rejeitando conflito,
path hostil e ownership ambíguo antes de writes. Dry-run e execução compartilham o plano.

## Critério observável

Gates de planejamento provam filtro V5, bijeção e ordem; dry-run escreve zero bytes e
apresenta exatamente o mesmo plano aceito pela execução.

## Revisão 2026-09-26 — completude dos reparos

O planejamento completo inspeciona todos os owners produtivos parseados, mesmo sem V5, e verifica ambos os sentidos dos hashes. Metadados reversos ilegíveis ou inválidos impedem o lote antes de qualquer escrita. A API histórica de planejamento por V5 permanece compatível.
