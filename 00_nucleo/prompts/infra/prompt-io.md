# Prompt: primitivas confinadas de I/O L0
Hash do Código: 6557a9cc

Owner exclusivo: `03_infra/prompt_io.rs`.

Fornecer leitura e validação de arquivos regulares sob `00_nucleo`, rejeitando absoluto,
traversal, symlink e escape. Normalizar apenas metadados explicitamente autorizados.

## Critério observável

Fixtures confinadas aceitam arquivo regular e rejeitam absoluto, `..`, symlink, duplicata
de metadado e escape, preservando todos os bytes não autorizados.

## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
