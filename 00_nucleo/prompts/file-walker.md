# Prompt: walker do projeto
Hash do Código: 9c48719c

Owner exclusivo: `03_infra/walker.rs`.

Enumerar arquivos analisáveis deterministicamente, classificar camada/linguagem e separar
erros de infraestrutura. Não seguir symlink nem absorver falha como inventário completo.

Para V2, classificar cada fonte aceita como `Production` ou `Test` pela convenção nominal
fechada da linguagem. Se nome ou stem não for UTF-8, preservar `Unknown` e emitir
`SourceError::Unreadable` com razão explícita, projetável como V0, em vez de assumir produção. As
convenções consideram a extensão completa; em particular, prefixo/sufixo de teste C só
vale para `.c`, nunca para `.h`. A busca de teste adjacente usa o mesmo domínio aceito pelo walker:
arquivo excluído, diretório ou symlink não cobre produção. O conteúdo não participa da
classificação e artefatos `Test` não consultam adjacência própria.

## Critério observável

O mesmo conjunto de arquivos produz ordem idêntica; symlink/inacessível vira erro separado
e nunca desaparece como sucesso parcial.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 157. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `[layers]`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
