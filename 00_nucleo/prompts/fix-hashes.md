# Prompt: planejamento de fix-hashes
Hash do Código: e3670870

Owner exclusivo: `02_shell/fix_hashes.rs`.

Transformar violações V5 válidas em plano bijetivo de atualização, rejeitando conflito,
path hostil e ownership ambíguo antes de writes. Dry-run e execução compartilham o plano.

## Critério observável

Gates de planejamento provam filtro V5, bijeção e ordem; dry-run escreve zero bytes e
apresenta exatamente o mesmo plano aceito pela execução.

## Revisão 2026-09-26 — completude dos reparos

O planejamento completo inspeciona todos os owners produtivos parseados, mesmo sem V5, e verifica ambos os sentidos dos hashes. Metadados reversos ilegíveis ou inválidos impedem o lote antes de qualquer escrita. A API histórica de planejamento por V5 permanece compatível.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 41. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `"Hash do Código: <hash>"`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
