# Prompt: primitivas confinadas de I/O L0
Hash do Código: c9a967ec

Owner exclusivo: `03_infra/prompt_io.rs`.

Fornecer leitura e validação de arquivos regulares sob `00_nucleo`, rejeitando absoluto,
traversal, symlink e escape. Normalizar apenas metadados explicitamente autorizados.

## Critério observável

Fixtures confinadas aceitam arquivo regular e rejeitam absoluto, `..`, symlink, duplicata
de metadado e escape, preservando todos os bytes não autorizados.
