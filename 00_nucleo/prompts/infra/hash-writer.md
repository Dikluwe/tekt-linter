# Prompt: escrita transacional de hashes
Hash do Código: e13dfc60

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
