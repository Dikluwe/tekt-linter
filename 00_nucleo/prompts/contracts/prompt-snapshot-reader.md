# Prompt: contrato PromptSnapshotReader
Hash do Código: PENDING_P0106

## Owner

`01_core/contracts/prompt_snapshot_reader.rs`, exclusivamente.

## Instrução

Declarar a porta pura que lê e serializa `PublicInterface` para V6. A fronteira recebe
e devolve entidades L1, sem conhecer JSON, arquivos ou marcadores concretos.

## Restrições

- nenhuma dependência de serde ou filesystem em L1;
- entrada inválida/ausente é representada por `None`;
- a serialização deve ser acessível por injeção, não por importação de L3.

## Critérios

Mocks exercitam snapshot presente e ausente e permitem testar V6 sem acesso externo.
