# Prompt: apresentação lossless de paths
Hash do Código: PENDING_P0106

## Owner

`02_shell/path_encoding.rs`, exclusivamente.

## Instrução

Converter paths nativos para representações textuais/machine-readable sem colisão entre
bytes distintos e sem panic em paths não UTF-8.

## Restrições

- não usar conversão lossy como identidade;
- preservar distinção e estabilidade por plataforma;
- não realizar I/O.

## Critérios

UTF-8 permanece legível; bytes inválidos são escapados deterministicamente; duas
identidades nativas distintas não geram a mesma saída.
