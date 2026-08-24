# Prompt: Rule V2 - Missing Test File (test-file)
Hash do Código: f9074a5f

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

## Fundamentação Teórica

1. **Necessidade de Pareamento Obrigatório (motivada por achados de co-evolução):**
   * **Zaidman et al. (2011)** (*Studying the Co-Evolution of Production and Test Code*): Demonstram empiricamente que, quando o desenvolvimento de testes fica desacoplado no tempo do desenvolvimento da lógica de produção, a qualidade e a testabilidade do núcleo degradam. Isso motiva a política de V2: exigir cobertura de teste como pré-condição de conformidade, e não como item postergável — ainda que V2, por sua natureza estática, verifique apenas a presença da cobertura no momento da análise, e não a sincronia temporal de sua criação.
2. **Invariante de Pareamento Modular (1:1):**
   * **Miranda et al. (2025)** (*Test Co-Evolution in Software Projects: A Large-Scale Empirical Study*): Provam que a integridade e a robustez de componentes de software críticos dependem da existência de testes unitários dedicados por módulo (relação 1:1), demonstrando que depender exclusivamente de testes indiretos de integração deixa lacunas estruturais não cobertas.
3. **Escopo de Verificação: Comportamento vs. Declaração de Tipos:**
   * **Cheque Bernardo (2011)** (*Padrões de Testes Automatizados*): O esforço e o escopo de verificação unitária devem recair sobre unidades que contêm lógica executável e transformações de estado/dados. Módulos puramente declarativos (estruturas de dados e definições de traits sem métodos implementados) são isentos por não conterem fluxo de controle passível de regressão lógica.
