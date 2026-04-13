# Prompt: Padrões de Arquitetura e Gestão de Prompts (architecture-standards)

**Camada**: L0 (Documentação)
**Criado em**: 2026-04-12
**Revisado em**: 2026-04-12 (Definição inicial de gestão a longo prazo)

---

## Princípios Fundamentais

Este documento define as regras de ouro para a manutenção da integridade arquitetural do projeto através do uso de prompts L0 e o sistema Tekt.

### 1. Regra 1:1 Universal
A regra **1 prompt = 1 ficheiro de código** aplica-se a todas as camadas do projeto (**L1, L2, L3 e L4**).
- Cada ficheiro de implementação deve possuir a sua respectiva documentação de intenção (L0).
- Isto garante que o linter funcione de forma uniforme em todo o projeto, sem excepções.

### 2. Responsabilidade Única (Unidade de Medida)
**Mais de um prompt não pode definir o mesmo ficheiro.**
- Se um ficheiro exige dois prompts diferentes para descrever as suas responsabilidades, isso é um erro de arquitetura.
- **Solução**: Dividir o ficheiro de código em dois ficheiros distintos, cada um com o seu respectivo prompt L0. O ficheiro é a unidade de medida padrão.

### 3. Granularidade de Funções
O linter rastreia ficheiros inteiros, não funções isoladas.
- **Funções Complexas**: Se uma função exige um documento de especificação exclusivo devido à sua complexidade, ela **deve** ser extraída para o seu próprio ficheiro de código (ex: `algoritmo_complexo.rs`) com um prompt dedicado.
- **Funções Normais**: Regras de funções padrão devem ser descritas como subsecções dentro do prompt geral do ficheiro onde residem.

### 4. Gestão de Mudanças (Estado vs. Histórico)
Os prompts L0 **não são um registo cronológico** de eventos. O histórico do Git serve para esse propósito.
- **L0 é a especificação do estado atual do sistema.**
- **Adição de funcionalidade**: 
  1. Abre o prompt existente.
  2. Adiciona a nova regra.
  3. Guarda o ficheiro `.md`.
  4. Escreve o código no ficheiro de implementação.
  5. O linter reconhece a atualização e gera o novo hash.
- **Refatoração**:
  - Se um ficheiro é dividido em três, o prompt original deve ser dividido em três ficheiros `.md` correspondentes.
  - O L0 deve refletir sempre a estrutura exata do código naquele momento.

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-04-12 | Criação inicial com as definições literais de gestão de arquitetura | N/A |
