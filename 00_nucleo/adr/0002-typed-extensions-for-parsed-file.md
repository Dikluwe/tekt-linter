# ⚖️ ADR-0002: Atomic ParsedFile and Trait-Based Rule Contracts

**Status**: `ACEITO`
**Data**: 2026-03-14

---

## Contexto

Na Arquitetura Cristalina do `crystalline-lint`, a entidade `ParsedFile` em `01_core` atua como a Representação Intermediária (IR) principal para todas as verificações de regras (V1 a V5 e além).

O design original acoplou fortemente a estrutural essencial da AST (`path`, `layer`, `language`, `imports`, `tokens`) a variáveis de estado exclusivas de regras específicas (por exemplo, `prompt_file_exists` para V1, `has_test_coverage` para V2). À medida que novas propostas emergem (como snapshots V6), a struct `ParsedFile` se torna um Objeto Deus (God Object).

Essa abordagem viola o **Princípio Aberto/Fechado (OCP)**: a raiz do L1 requer modificação e força a quebra de todos os testes locais em cascata (erro compliador `E0063`). Inicialmente fora proposto um Typemap `Extensions`, mas este foi superado por uma abordagem ainda mais idiomática no ecossistema Rust.

---

## Decisão

Desacoplar os requisitos específicos de cada regra transformando-os em **Traits (Contratos de Interface)** em *L1*. As regras não dependem mais da estrutura concreta `ParsedFile`, dependem apenas do comportamento que necessitam.

### 1. Regras Operam em Traits
As verificações deixam de aceitar `&ParsedFile` rígido e passam a aceitar um genérico:
```rust
pub trait HasCoverage {
    fn has_test_coverage(&self) -> bool;
}

// A Regra V2 só pede o que precisa
pub fn check<T: HasCoverage>(file: &T) -> Vec<Violation> { ... }
```

### 2. A Entidade Principal Implementa
A struct `ParsedFile` central retém os campos fundamentais e as novas variáveis como dependências concretas injetadas por L3, mas ela *implementa* as Traits em arquivos separados ou no próprio bloco de entidade, servindo as regras que chamam essas funções.

### 3. Fixtures de Teste Minimalistas Isolados
Testes locais de regras L1 não instanciam mais a entidade `ParsedFile` inteira. Cada suíte local declara seu próprio Mock:
```rust
#[cfg(test)]
struct MockFile { coverage: bool }

impl HasCoverage for MockFile { 
    fn has_test_coverage(&self) -> bool { self.coverage } 
}
```

---

## Consequências

**Positivas:**
- Extrema robustez tipada em tempo de compilação sem abstração de run-time (`Extensions`).
- "Testes a Prova de Balas": a adição de 100 campos novos no IR global `ParsedFile` jamais quebrará testes passados, pois os Mocks locais isolam a injeção.
- OCP Verdadeiro: novas regras apenas declaram suas próprias Traits locais e L3 faz a fiação (wiring) com `ParsedFile`.

**Negativas:**
- Pequeno boilerplate em escrever os blocos `impl Trait for ParsedFile` e as structs de `Mock` nos testes.
