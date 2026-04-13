# Prompt: Zig Parser (parsers/zig)
Hash do Código: 4ca315f8

**Camada**: L3 (Infra)
**Padrão**: Adapter over `tree-sitter-zig`
**Criado em**: 2026-04-03
**Revisado em**: 2026-04-03
**Arquivos gerados**:
  - 03_infra/zig_parser.rs

---

## Contexto

O parser para a linguagem Zig no linter `crystalline-lint`. Esta ferramenta analisa arquivos `.zig` e extrai as informações estruturais necessárias para validar as regras arquiteturais da Tekt (como isolamento de camadas e pureza de L1).

Este parser deve ser implementado de forma **Zero-Copy** (I1), e realizar a **resolução física de camadas** (I2) para imports, garantindo que não há vazamentos estruturais.

---

## Header Cristalino

O Header Cristalino para o Zig deve seguir os comentários de linha padrão (`//` ou `///`).

```zig
// Crystalline Lineage
// @prompt 00_nucleo/prompts/parsers/zig.md
// @prompt-hash <sha256[0..8]>
// @layer L3
// @updated YYYY-MM-DD
```

O bloco deve estar no topo do arquivo para que a regra `V1` seja satisfeita.

---

## Resolução de Camadas — LayerResolver

Seguindo a invariante **I2** (resolução física), o algoritmo deve ser:

1. Se o import não começa com `./` ou `../` e não é a stdlib (excluindo `@import("std")`) nem um package externo declarado, é resolvido como `Layer::Unknown`.
2. Para imports locais (ex: `@import("./core/entity.zig")`), realizar álgebra de paths algébrica (ADR-0009).
3. Utilizar `resolve_file_layer` para mapear o path normalizado para uma camada da topologia.

---

## Estrutura de Imports (V3, V9, V10)

O Zig utiliza a função embutida `@import("...")` para declarar dependências.

| Nó AST Zig | `ImportKind` | Notas |
|------------|--------------|-------|
| chamadas de `@import` | `Direct` | O path fornecido no argumento deve ser extraído. |

O parser deve mapear corretamente cada import para a camada alvo.

---

## Visibilidade e Interface Pública (V6)

A visibilidade no Zig é controlada pelo modificador `pub`. 

| Nó AST | `TypeKind` | Notas |
|--------|------------|-------|
| `struct` com `pub` | `Struct` | Capturado se o modificador de visibilidade `pub` estiver presente. |
| `enum` com `pub` | `Enum` | Capturado se o modificador de visibilidade `pub` estiver presente. |
| `fn` com `pub` | Function | Capturado se o modificador de visibilidade `pub` estiver presente. |

---

## Test Coverage (V2)

A cobertura de testes no Zig é declarada e executada diretamente no código-fonte via blocos `test`.

```zig
test "descrição do teste" {
    // corpo do teste
}
```

O parser deve procurar pelo nó `test_declaration` (ou equivalente na gramática tree-sitter) para marcar `has_test_coverage = true`.

---

## Traits/Interfases (V11)

Zig não possui suporte formal a traits/interfaces via palavras-chave exclusivas como em Rust ou TS. Ele utiliza interfaces dinâmicas via `@ptrCast` ou duck-typing durante compile-time.

Nesta implementação inicial, `declared_traits` e `implemented_traits` serão sempre `[]` no Zig.

---

## Declarações (V12)

O campo `declarations` deve capturar structs, enums e unions que não representem a implementação de um contrato (visto que o Zig não possui interfaces explícitas).

---

## Restrições

- **I1 Zero-Copy**: Não permitir conversões para string durante a análise.
- **I6 UnsupportedLanguage**: Se o arquivo não possuir a extensão `.zig`, retornar erro adequado.
- **I7 ImportKind**: Usar apenas os enums de `ImportKind` agnósticos definido no core.

---

## Histórico de Revisões

| Data | Motivo | Arquivos Afetados |
|------|--------|-------------------|
| 2026-04-03 | Criação inicial (ADR-0009): suporte ao Zig | — |
