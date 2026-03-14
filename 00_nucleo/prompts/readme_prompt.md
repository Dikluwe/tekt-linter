# Prompt: README (readme)

**Camada**: L0 (Documentação)
**Criado em**: 2025-03-13
**Arquivos gerados**: README.md

---

## Contexto

O `crystalline-lint` é um linter arquitetural para projetos que
seguem a Arquitetura Cristalina. O README é o ponto de entrada
para qualquer pessoa que encontre o repositório — deve comunicar
o propósito, como instalar, como usar, e como o projeto se
auto-valida.

O público é duplo: desenvolvedores adotando a Arquitetura
Cristalina que precisam do linter, e contribuidores que precisam
entender a estrutura do próprio projeto.

---

## Conteúdo obrigatório

1. **O que é** — problema que resolve em 2-3 linhas
2. **Instalação** — `cargo install` e download de binário para CI
3. **Uso rápido** — exemplo mínimo funcional
4. **Verificações** — tabela V1–V5 com descrição e severidade
5. **Flags CLI** — tabela completa
6. **crystalline.toml** — exemplo comentado
7. **Header canônico** — formato `//!` obrigatório em arquivos Rust
8. **Auto-validação** — o linter valida seu próprio código
9. **Estrutura do projeto** — árvore derivada dos prompts
10. **Integração CI** — exemplo GitHub Actions

---

## Restrições

- Sem marketing — apenas fatos e exemplos funcionais
- Exemplos de código devem ser copiáveis e funcionais
- Tabelas para referência rápida, prosa apenas onde necessário
- Manter alinhado com `linter-core.md` como fonte de verdade

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | README.md |
