# Prompt: linguagem e camada arquitetural
Hash do Código: 09edb04d

Owner exclusivo: `01_core/entities/layer.rs`.

Modelar `Layer` e `Language`, incluindo parsing nominal conservador. Valores desconhecidos
permanecem explícitos; a entidade não lê paths/configuração e não decide violações.

## Critério observável

Todos os valores de camada/linguagem fazem round-trip nominal; desconhecidos continuam
distintos. Testes da entidade verificam igualdade, clone e ausência de I/O/política.
