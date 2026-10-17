# Prompt: escrita transacional de hashes
Hash do Código: c56d3014

Owner exclusivo: `03_infra/hash_writer.rs`.

Aplicar plano validado a headers de código e metadados de prompt numa transação com
preflight, revalidação e rollback. Nunca alterar bytes funcionais ou deixar lote parcial.

## Critério observável

Testes provam preservação de BOM/CRLF/permissões, escrita atômica, rollback e substituição
exclusiva das linhas de metadado autorizadas.

## Revisão 2026-09-26 — completude dos reparos

Expor verificação do Hash do Código contra o digest atual do owner; ler prompt confinado à raiz, com limite de tamanho, e validar metadado canônico único usando o mesmo parser das escritas. Retornar erro para metadados ausentes ou inválidos.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.

## Rustdoc estrito — 2026-10-16

Medição anterior à correção: cargo doc --offline --locked --no-deps com RUSTDOCFLAGS="-D warnings" falhou neste consumer nas linhas 46. O diagnóstico tekt-v16-none-parser-2026-10-16 em typst-crystalline conserva attempt-doc-green-doc.log, hashes base e53de6f e UTC em docs-before-code.json. Corrigir somente a apresentação dos comentários Rustdoc: representar como código literal `"Hash do Código: <hash>"`. Não alterar tokens executáveis, tipos, campos, assinaturas, mensagens de execução, ordem ou comportamento. É manutenção documental interna em fluxo contínuo; o RED é a compilação Rustdoc estrita registrada e o GREEN exige cargo doc sem advertências, mais suíte, Clippy e linhagem existentes. Resselar os dois sentidos somente dos owners editados.
