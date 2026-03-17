# Prompt: README (readme)

**Camada**: L0 (Documentação)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-16 (ADR-0006, ADR-0007: V7–V12)
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
4. **Verificações** — tabela V0–V12 com descrição e severidade
5. **Flags CLI** — tabela completa incluindo `--checks v0,...,v12`
6. **crystalline.toml** — exemplo comentado com todas as secções:
   `[layers]`, `[excluded]`, `[module_layers]`, `[l1_ports]`,
   `[orphan_exceptions]`, `[wiring_exceptions]`, `[rules]`
7. **Header canônico** — formato `//!` obrigatório em arquivos Rust
8. **Workflows de correção** — `--fix-hashes` (V5) e
   `--update-snapshot` (V6)
9. **Auto-validação** — o linter valida seu próprio código
10. **Estrutura do projeto** — árvore derivada dos prompts
11. **Integração CI** — exemplo GitHub Actions

---

## Restrições

- Sem marketing — apenas fatos e exemplos funcionais
- Exemplos de código devem ser copiáveis e funcionais
- Tabelas para referência rápida, prosa apenas onde necessário
- Manter alinhado com `linter-core.md` como fonte de verdade
- V0, V8 e V10 são Fatal — deve ficar claro que não são
  configuráveis via `--fail-on`
- V11 opera sobre `ProjectIndex` pós-reduce — comportamento
  diferente de regras por arquivo deve ser mencionado na nota
  de V11
- A distinção `contracts/` vs `entities/rule_traits` não precisa
  aparecer no README — é detalhe interno de L1

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | README.md |
| 2026-03-16 | ADR-0006 e ADR-0007: V7–V12, crystalline.toml completo, estrutura actualizada | README.md |
