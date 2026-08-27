# Prompt: plano puro de inicialização da configuração
Hash do Código: 7720ae99

Owner exclusivo: `01_core/entities/config_init.rs`.

Representar inventário físico já observado, estado do destino, topologia detectada e plano
de inicialização sem acessar filesystem, TOML ou CLI. A detecção aceita somente os nomes
canônicos L0, L1, L2, L3, L4 e lab; L0/L1 são obrigatórios. Variante diferente em bytes
mas igual por caixa ASCII produz `Unknown` antes da ausência. Diretórios não canônicos são
irrelevantes. `Present` e `Unsafe` têm precedência e não exigem topologia.

