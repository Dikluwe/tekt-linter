# Prompt: adapter seguro de inicialização da configuração
Hash do Código: 07cb3035

Owner exclusivo: `03_infra/config_init.rs`.

Inventariar somente entradas imediatas de uma raiz física sem seguir symlink, serializar
bytes TOML determinísticos a partir da topologia L1 e publicar `crystalline.toml` sem
substituir destino concorrente. Escrever e sincronizar temporário exclusivo, validá-lo com
`CrystallineConfig::load`, publicar o mesmo inode por operação no-clobber e limpar somente
o temporário próprio. Erros preservam as razões normativas P0119.


## Revisão 2026-09-26 — fechamento de qualidade

Eliminar avisos do Clippy preservando resultados, ordem, localizações, interfaces públicas e invariantes acima. Manter compatibilidade das APIs utilizadas com Rust 1.75; simplificar expressões e remover parâmetros/auxiliares redundantes. Agrupar contexto interno de travessia quando necessário, sem reclassificar entradas. Validar com a suíte existente e autoverificação, e sincronizar hashes nos dois sentidos.
