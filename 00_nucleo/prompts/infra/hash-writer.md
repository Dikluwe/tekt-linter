# Prompt: escrita transacional de hashes
Hash do Código: PENDING_P0106

Owner exclusivo: `03_infra/hash_writer.rs`.

Aplicar plano validado a headers de código e metadados de prompt numa transação com
preflight, revalidação e rollback. Nunca alterar bytes funcionais ou deixar lote parcial.
