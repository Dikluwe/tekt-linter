# Prompt: use-case e apresentação de refinamento
Hash do Código: 9db43714

Owner exclusivo: `02_shell/refinement.rs`.

Orquestrar portas de extração/snapshot e apresentar veredito com protocolo de saída
0/1/2. Shell não executa Git diretamente nem redefine a comparação L1.

## Critério observável

Gates CLI demonstram exit 0/1/2 para preserved/violated/unknown e formatos text/SARIF
preservam testemunhas sem chamar Git fora da porta injetada.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 50. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `[[observable]]`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
