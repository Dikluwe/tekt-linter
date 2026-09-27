# Prompt: FsPromptSnapshotReader
Hash do Código: 7c26a8b7

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

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
