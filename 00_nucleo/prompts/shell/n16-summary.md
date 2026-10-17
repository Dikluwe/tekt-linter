# Prompt: resumo N16
Hash do Código: ac50f1ef

Owner exclusivo: `02_shell/n16_summary.rs`.

Coletar e apresentar achados N16 deterministicamente, preservando contagens e localizações.
Não reexecutar a regra nem alterar severidade.

## Critério observável

Gates preservam totais, agrupamento, ordenação e arredondamento para amostras vazias,
pequenas e extremas; a entrada não é reclassificada.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 15, 17, 19. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `N16[α]`, `N16[β]`, `N16[γ]`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
