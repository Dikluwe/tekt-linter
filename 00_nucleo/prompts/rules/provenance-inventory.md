# Prompt: inventário de proveniência V21
Hash do Código: PENDING_P0106

## Owner

`01_core/rules/provenance_inventory.rs`, exclusivamente.

## Instrução

Agregar, sem I/O, as constantes contextuais extraídas e suas classificações de
proveniência para produzir inventário auditável, ordenado e estável.

## Restrições

- não reimplementar o predicado decisório de V21;
- preservar multiplicidade, localização e linguagem;
- não promover inventário a violação nem resolver referências externas.

## Critérios

Entradas equivalentes em ordens distintas geram agrupamento estável; duplicatas e
estados sem fonte continuam visíveis.
