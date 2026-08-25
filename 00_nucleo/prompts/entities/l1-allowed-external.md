# Prompt: entidade L1AllowedExternal
Hash do Código: PENDING_P0106

## Owner

`01_core/entities/l1_allowed_external.rs`, exclusivamente.

## Instrução

Modelar allowlists externas por linguagem e prefixos de stdlib isentos. A entidade
responde se pacote/item é permitido e seleciona a política pela linguagem analisada.

## Restrições

- entidade pura, sem TOML e sem filesystem;
- matching por segmentos/prefixos definidos, não substring acidental;
- fallback de linguagem desconhecida permanece conservador.

## Critérios

Construtores preservam isenções próprias de cada linguagem; pacote, item e lookalikes
têm resultados determinísticos.
