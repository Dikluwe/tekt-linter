# Prompt: escrita transacional de hashes
Hash do Código: 98939785

Owner exclusivo: `03_infra/hash_writer.rs`.

Aplicar plano validado a headers de código e metadados de prompt numa transação com
preflight, revalidação e rollback. Nunca alterar bytes funcionais ou deixar lote parcial.

## Critério observável

Testes provam preservação de BOM/CRLF/permissões, escrita atômica, rollback e substituição
exclusiva das linhas de metadado autorizadas.
