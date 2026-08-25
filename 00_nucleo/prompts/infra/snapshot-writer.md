# Prompt: escrita transacional de snapshots
Hash do Código: PENDING_P0106

Owner exclusivo: `03_infra/snapshot_writer.rs`.

Substituir atomicamente apenas o marcador canônico de snapshot, preservando conteúdo
humano e permissões. Falha, corrida ou destino hostil não deixa publicação parcial.
