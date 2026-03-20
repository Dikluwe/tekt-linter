# CLAUDE.md — crystalline-lint

Este ficheiro guia o Claude Code neste repositório.
Para referência completa de arquitectura, tabelas de dependências
e exemplos: **ler README.md primeiro**.

---

## O que é este projecto

`crystalline-lint` é um linter arquitectural em Rust que enforça
o padrão Arquitetura Cristalina. Valida o seu próprio código
contra as suas próprias regras — `cargo run -- .` com zero
violations é o critério de correcção primário.

---

## Comandos

```bash
cargo build                            # Debug build
cargo build --release                  # Release build
cargo test                             # Todos os testes
cargo test <módulo>                    # Módulo específico
cargo test -- --nocapture              # Com stdout
cargo run -- .                         # Lint directório actual
cargo run -- --fix-hashes .            # Corrigir @prompt-hash
cargo run -- --fix-hashes --dry-run .  # Preview sem escrever
cargo run -- --format sarif .          # Saída SARIF
```

**Critério de correcção primário:**
```bash
cargo run -- .
# ✓ No violations found
```

---

## Protocolo de Nucleação (obrigatório antes de qualquer código)

Antes de escrever **qualquer linha de código** em L1–L4:

1. Inspeccionar `00_nucleo/prompts/` para o prompt correspondente
2. **Prompt existe** → ler completamente (contexto, restrições,
   critérios, histórico)
3. **Prompt não existe** → PARAR. Propor prompt estruturado ao
   developer. Não escrever código sem nucleação.
4. Materializar na ordem: **testes primeiro, código depois**
   (ver secção seguinte)
5. Registar a revisão no histórico do prompt (data, motivo,
   ficheiros afectados)
6. Executar `--fix-hashes` após editar qualquer prompt em
   `00_nucleo/`

Uma nucleação sem testes é incompleta. Um componente sem prompt
em L0 é estruturalmente ilegítimo mesmo que seja funcionalmente
correcto.

---

## Ordem de materialização — Testes Primeiro

**Esta é a regra mais importante desta secção.**

A IA tem tendência natural a escrever o código e depois os testes
que descrevem o que o código faz. Isso produz testes que são uma
sombra da implementação — não detectam bugs introduzidos durante
a materialização.

A ordem obrigatória é:

### Fase 1 — Testes (a partir dos Critérios de Verificação do prompt)

```
1. Ler a secção "Critérios de Verificação" do prompt L0
2. Escrever os testes correspondentes no ficheiro _test ou #[cfg(test)]
3. Executar: cargo test <módulo>
4. VERIFICAR QUE OS NOVOS TESTES FALHAM
```

**Se um teste passar sem código de produção existir, o teste
está errado.** Um teste que passa imediatamente não fornece
nenhuma garantia — é apenas documentação executável do
comportamento actual, que pode ser o comportamento errado.

### Fase 2 — Implementação

```
5. Escrever o código de produção para os testes passarem
6. Executar: cargo test <módulo>
7. VERIFICAR QUE TODOS OS TESTES PASSAM
8. Executar: cargo run -- .
9. VERIFICAR ZERO VIOLATIONS
```

### Cobertura obrigatória dos critérios

Cada cenário `Dado/Quando/Então` no prompt deve ter um teste
correspondente. Em particular, os **caminhos negativos** são
obrigatórios — se o prompt especifica que algo não deve acontecer,
deve existir um teste que confirma que não acontece.

Exemplos de caminhos negativos frequentemente omitidos:
- `--checks v11` não deve activar `v1` nem `v2`
- `class FooTest: pass` sem herança de `TestCase` não é cobertura de teste
- `export * from` deve ser `Glob`, não `Direct`
- Ficheiro `foo.test.ts` já é ficheiro de teste — `has_adjacent_test = false`

---

## Header de linhagem obrigatório

Todo ficheiro criado ou editado em L1–L4 deve começar com:

```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```

---

## Quando um prompt está errado vs quando o código está errado

Antes de corrigir qualquer problema, determinar a origem:

**O prompt está errado quando:**
- O prompt especifica explicitamente um algoritmo ou padrão
  incorreto (ex: `Box::leak`, `lower.contains("v1")`)
- O prompt omite um caso que deveria cobrir (ex: padrões TypeScript
  em `file-walker.md`)
- O prompt contradiz um ADR existente

**Neste caso:** corrigir o prompt L0 primeiro. Só depois
rematerializar. Materializar código a partir de um prompt errado
reproduz o bug.

**O código está errado quando:**
- O prompt especifica o resultado correcto mas a implementação
  diverge (ex: prompt diz `Glob`, código retorna `Direct`)
- Um detalhe de implementação não coberto pelo prompt foi feito
  incorrectamente (ex: `find_first_error_pos` retorna `(0,0)`)

**Neste caso:** corrigir o código directamente. O prompt não
precisa de ser alterado — a implementação é que falhou em seguir
o contrato.

**Para determinar a origem:** comparar o comportamento esperado
no prompt com o comportamento actual no código. Se o prompt
especifica o comportamento correcto, a falha é de implementação.
Se o prompt especifica o comportamento errado, a falha é do prompt.

---

## Restrições Rust — Todas as camadas

### Borrowing (obrigatório)

Preferir referências a valores owned. Sem `clone()` sem comentário
justificativo. Sem `Rc<RefCell<T>>` — usar o borrow checker.

### Lifetimes (obrigatório)

Lifetimes explícitos quando o compilador os exige. Sem `'static`
para evitar pensar em lifetimes. `'static` produzido por
`Box::leak` é **proibido** (ADR-0005) — usar `Cow` ou buffer
interno com `intern()`.

### `Box::leak` — proibido (ADR-0005)

`Box::leak` foi eliminado do projecto pelo ADR-0005 que introduziu
`Cow<'a, Path>` para `Location`. Qualquer uso de `Box::leak` para
produzir `&'static str` ou `&'static Path` é uma violação do ADR.

O padrão correcto para strings que não existem no buffer do
`SourceFile` (ex: subdirs resolvidos por algebra de paths) é o
buffer interno com `intern()`:

```rust
// Padrão estabelecido em FsPromptWalker e nos parsers TS/Python
subdirs_buffer: std::sync::Mutex<Vec<Box<str>>>,

fn intern_subdir(&self, s: String) -> &str {
    let mut buf = self.subdirs_buffer.lock().unwrap();
    let boxed: Box<str> = s.into_boxed_str();
    let raw: *const str = &*boxed as *const str;
    buf.push(boxed);
    // SAFETY: raw aponta para dado heap que vive em self.subdirs_buffer.
    // Realoções do Vec movem o Box (fat pointer), não o dado heap.
    // Mutex garante exclusão mútua — sem borrows concorrentes do buffer.
    unsafe { &*raw }
}
```

Usar `Mutex` (não `RefCell`) quando o tipo precisa de ser `Sync`
para participar no pipeline paralelo do rayon.

### Enums sobre booleanos e Options parciais (obrigatório)

Estados inválidos devem ser irrepresentáveis. Substituir
`is_valid: bool` + `error: Option<String>` por enum que torna
a contradição impossível.

### Parse, não validar (obrigatório)

Funções recebem dados brutos e retornam tipos validados.
Downstream não revalida — o tipo é a prova.

### Newtype para primitivos de domínio (obrigatório)

Envolver primitivos que representam conceitos de domínio em
structs de campo único.

---

## Restrições por camada

### L1 — Core (lógica pura)

| Regra | Detalhe |
|-------|---------|
| Zero I/O | Sem `std::fs`, `std::net`, `std::process` |
| Erros | `thiserror` com enums tipados — sem `anyhow` |
| Estado | Sem `Mutex`, `Arc`, `Atomic`, `RefCell` |
| Concorrência | Nenhuma |
| Traits seladas | Para contratos não destinados a implementação externa |
| Typestate | Para operações com ordenação obrigatória |

### L2 — Shell (CLI e formatadores)

| Regra | Detalhe |
|-------|---------|
| Erros | `anyhow` permitido para propagação CLI |
| Imports | Apenas L1 — nunca L3 |
| Concorrência | Nenhuma — execução sequencial |

### L3 — Infra (implementações de I/O)

| Regra | Detalhe |
|-------|---------|
| Erros | `thiserror` — erros I/O tipados que mapeiam para erros L1 |
| Imports | Apenas L1 — nunca L2 ou L4 |
| Concorrência | `Arc<Mutex<T>>` ou canais permitidos para walking paralelo |

### L4 — Wiring (composição)

| Regra | Detalhe |
|-------|---------|
| Lógica | Zero — qualquer `if/else` de negócio é defeito estrutural |
| Erros | `anyhow` para propagação top-level |
| Concorrência | Spawning de threads apenas aqui |
| `expect()` em threads rayon | Proibido — panic numa thread rayon produz mensagem pouco informativa; usar `?` ou tratamento explícito |

---

## Regras de teste

### Atomização

Um teste por comportamento. Sem setup partilhado. Sem teste que
depende de outro. Mocks implementam traits L1 — nunca I/O real.

### Localização

Testes co-localizados no mesmo ficheiro via `#[cfg(test)]`.
Nunca ficheiros `_test.rs` separados.

### Cobertura mínima obrigatória por função

Para cada função pública em L1:
- Caminho feliz (input válido → output correcto)
- Caminho negativo (input inválido → erro correcto)
- Caso limite (zero, vazio, máximo)

Para parsers L3, adicionalmente:
- Cada variante de `ImportKind` (Direct, Glob, Alias, Named)
- Path relativo com `../` normal → camada correcta
- Path com `../` excessivos → `Layer::Unknown`
- Import para `lab/` → `Layer::Lab`
- `target_subdir` para imports de L1 (não só para aliases)
- Ficheiro de teste adjacente (por linguagem)
- Ficheiro que já é teste → `has_adjacent_test = false`
- `SyntaxError` reporta linha `> 0`

### Exemplo de estrutura

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Mock mínimo — apenas os campos necessários para a trait
    struct MockFile { layer: Layer, tokens: Vec<Token<'static>> }
    impl HasTokens<'static> for MockFile { ... }

    #[test]
    fn v4_flags_forbidden_symbol_in_l1() { ... }      // caminho feliz

    #[test]
    fn v4_ignores_forbidden_symbol_outside_l1() { ... } // caminho negativo

    #[test]
    fn v4_returns_empty_for_unknown_language() { ... }  // caso limite
}
```

---

## Restrições de configuração

### `[excluded]` no `crystalline.toml`

`[excluded]` é para **directórios**, não para ficheiros individuais.
O mecanismo `is_ignored` compara componentes de path — uma entrada
como `lib_root = "lib.rs"` excluiria qualquer ficheiro chamado
`lib.rs` em qualquer subdirectório do projecto.

Para excluir `lib.rs` da raiz do projecto da análise, o correcto
é garantir que tem um header `@prompt` válido e `@layer` correcto
— ou mapeá-lo explicitamente em `[layers]` se a estrutura o exigir.

---

## ADRs e decisões de design

Antes de qualquer decisão técnica nova, verificar se já existe um
ADR que a cobre. ADRs com estado `IMPLEMENTADO` são vinculativos —
não podem ser contrariados por prompts novos sem um ADR de revisão.

ADRs relevantes para decisões frequentes:

| ADR | Decisão |
|-----|---------|
| ADR-0001 | tree-sitter como IR; agnósticidade de linguagem |
| ADR-0002 | Traits para contratos de regras; mocks locais |
| ADR-0004 | Zero-copy com lifetimes; FQN em L3; Fail-Fast V0 |
| ADR-0005 | `Cow<'a, Path>` elimina `Box::leak`; `Cargo.toml` nucleado |
| ADR-0006 | Fechamento topológico; V7/V8/V9; `ProjectIndex` |
| ADR-0007 | Fechamento comportamental; V10/V11/V12 |
| ADR-0008 | Distribuição via binários estáticos |
| ADR-0009 | Parsers por linguagem; `ImportKind` semântico; V4 multi-linguagem |

---

## Quando criar um ADR

Criar ADR para decisões que:
- Alteram um contrato entre camadas (nova entidade na IR, nova trait)
- Revogam ou corrigem uma decisão anterior registada em L0
- Introduzem um novo padrão de implementação (ex: novo mecanismo
  de interning)

Não criar ADR para:
- Bugs de implementação que não alteram contratos
- Gaps de especificação em prompts (corrigir o prompt directamente)
- Extensões que seguem padrões já estabelecidos (ex: adicionar
  um quarto parser seguindo o template `_template.md`)

---

## Workflows

Operações estruturadas em `.agents/workflows/`:

| Workflow | Propósito |
|----------|-----------|
| `init-legado.md` | Inicializar projecto legado para migração |
| `gerar-spec.md` | Gerar novo prompt L0 estruturado |
| `integrar-legado.md` | Refactorizar ficheiro legado para camada correcta |
| `auditar-spec.md` | Auditar qualidade e completude de prompt |
| `clivar-modulo.md` | Dividir módulo grande pelas camadas correctas |

---

## Referência

Para descrições completas de camadas, diagramas de dependência,
definições de violations V0–V12 e configuração de `crystalline.toml`:
**README.md**
