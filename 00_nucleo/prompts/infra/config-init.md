# Prompt: adapter seguro de inicialização da configuração
Hash do Código: 2b83163c

Owner exclusivo: `03_infra/config_init.rs`.

Inventariar somente entradas imediatas de uma raiz física sem seguir symlink, serializar
bytes TOML determinísticos a partir da topologia L1 e publicar `crystalline.toml` sem
substituir destino concorrente. Escrever e sincronizar temporário exclusivo, validá-lo com
`CrystallineConfig::load`, publicar o mesmo inode por operação no-clobber e limpar somente
o temporário próprio. Erros preservam as razões normativas P0119.

