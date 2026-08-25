# Prompt: escrita transacional de snapshots
Hash do Código: 5a59e0d5

Owner exclusivo: `03_infra/snapshot_writer.rs`.

Substituir atomicamente apenas o marcador canônico de snapshot, preservando conteúdo
humano e permissões. Falha, corrida ou destino hostil não deixa publicação parcial.

## Critério observável

Gates comparam bytes antes/depois e mostram que somente o marcador canônico muda; falha
mantém original e diretório sem temporários residuais.
