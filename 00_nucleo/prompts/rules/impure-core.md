# Prompt: Rule V4 - Impure Core (impure-core)

**Camada**: L1 (Core - Rules)
**Regra**: V4
**Criado em**: 2025-03-13

## Contexto
A essência do estrato `L1` (`01_core/`) é ser matematicamente funcional e previsível: entradas resultam deterministicamente num resultado sem interferência de I/O de rede terrestre, leituras de CPU-clock ou bytes do disco. V4 age como guardião destas chaves.

## Especificação
A regra assinala a infração se encontrar call expressions diretas de funções atreladas à impureza num arquivo cuja camada informada seja `Layer::L1`. 
Os nós do AST abstraídos pela entidade (via trait `HasTokens`) que indicam *call expressions* ou declarações de rotas cujos caminhos globais resolvem para os seguintes símbolos proibidos acionam a V4:
- Sistema/Mundo exterior: `std::fs`, `std::io`, `std::net`, `std::process`, `tokio::fs`, `tokio::io`, `tokio::process`.
- Banco de Dados/Rede externa: `reqwest`, `sqlx`, `diesel`.
- Estado não-determinístico: `std::time::SystemTime::now()`, `rand::random()`.

A detecção é feita semanticamente baseada na árvore (AST), não é um simples `.contains()` por regex no texto do arquivo. Arquivos de teste `cfg(test)` são exceções que podem importar utilitários de I/O caso apropriado para fixtures locadas (embora o ideal seja evitar).

## Estrutura da Violação Gerada
- Rule ID: `V4`
- Level: `Error` (Bloqueante)
- Contexto da Mensagem: "Núcleo Impuro: operação proibida <símbolo_proibido> detectada em AST location X".

## Restrições (L1 Pura)
A extração dos _tokens_ do AST como nós semânticos tipados já fora parseada pela grammar do Tree-sitter instanciada no L3 e repassada via Trait para que as regras de L1 analisem determinísticamente os símbolos importados ou instanciados na linha, devolvendo as coordenadas perfeitamente traçadas na violação em SARIF.
