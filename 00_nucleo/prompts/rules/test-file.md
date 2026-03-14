# Prompt: Rule V2 - Missing Test File (test-file)

**Camada**: L1 (Core - Rules)
**Regra**: V2
**Criado em**: 2025-03-13

## Contexto
Sob a restrição da Regra Simuntânea (Testes), toda materialização de núcleo (L1) obriga o desenvolvimento conjunto de uma suíte de testes que valide as funções puras ali codificadas.

## Especificação
- A regra V2 acusa a ausência de cobertura de testes explícita num módulo em `L1` (`01_core/`).
- Ela verifica, dada uma entidade abstrata (via trait `HasCoverage`) cuja camada declarada seja `Layer::L1`, se há a cobertura de testes associada.
- **Isenções**: Arquivos que apenas declaram e exportam `pub trait`, `pub struct` ou `pub enum` _sem possuírem implementações de lógica (blocos `impl` que contenham funções/métodos com corpo lógico)_ são isentos dessa regra. O construtor (L3) deduz essa isenção inspecionando o AST no momento do parse.

## Estrutura da Violação Gerada
- Rule ID: `V2`
- Level: `Error` (Bloqueante)
- Contexto da Mensagem: "Módulo do núcleo carece de verificação simultânea (test file ou bloco cfg(test))".

## Restrições (L1 Pura)
- O processamento em L1 não olhará em disco para o test_file — o L3 (`FileWalker`/`LanguageParser`) injeta o metadado sobre se a cobertura existe na instância final inspecionada pela Trait.
