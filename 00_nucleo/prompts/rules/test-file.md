# Prompt: Rule V2 - Missing Test File (test-file)

**Camada**: L1 (Core - Rules)
**Regra**: V2
**Criado em**: 2025-03-13

## Contexto
Sob a restrição da Regra Simuntânea (Testes), toda materialização de núcleo (L1) obriga o desenvolvimento conjunto de uma suíte de testes que valide as funções puras ali codificadas.

## Especificação
- A regra V2 acusa a ausência de cobertura de testes explícita num módulo em `L1` (`01_core/`).
- Ela verifica, dado um `ParsedFile` no qual `ParsedFile.layer == Layer::L1`, se há a diretiva de teste interna (`#[cfg(test)]`) dentro do mesmo arquivo `foo.rs`, ou a existência adjacente/referenciada de um módulo `foo_test.rs`.
- **Isenções**: Arquivos que apenas declaram e exportam `pub trait`, `pub struct` ou `pub enum` _sem possuírem implementações de lógica (blocos `impl` que contenham funções/métodos com corpo lógico)_ são isentos dessa regra. O linter deduz essa isenção inspecionando o AST (`ParsedFile.tokens`).

## Estrutura da Violação Gerada
- Rule ID: `V2`
- Level: `Error` (Bloqueante)
- Contexto da Mensagem: "Módulo do núcleo carece de verificação simultânea (test file ou bloco cfg(test))".

## Restrições (L1 Pura)
- O processamento em L1 não olhará em disco para o `foo_test.rs` — o L3 (`FileWalker`/`LanguageParser`) que constrói o `ParsedFile` injeta nesta entidade o metadado sobre se os testes existem anexos ou emparelhados, ou o próprio AST indica a `cfg(test)`.
