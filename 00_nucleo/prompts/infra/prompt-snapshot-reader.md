# Prompt: FsPromptSnapshotReader
Hash do Código: PENDING_P0106

## Owner

`03_infra/prompt_snapshot_reader.rs`, exclusivamente.

## Instrução

Implementar a porta de snapshot: ler prompt confinado, extrair o marcador canônico,
desserializar/serializar `PublicInterface` em JSON determinístico.

## Restrições

- leitura ausente, hostil ou JSON inválido retorna `None`;
- não alterar seções humanas do prompt;
- round-trip preserva funções, tipos e reexports.

## Critérios

Snapshots válidos fazem round-trip; marcador malformado e path fora da raiz não vazam
I/O nem produzem interface parcial.
