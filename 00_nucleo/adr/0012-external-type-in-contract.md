# ⚖️ ADR-0012: V14 — Tipo Externo em Contrato L1 (ExternalTypeInContract)

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-20
**Implementado**: 2026-03-22

---

## Contexto

V3 garante a direcção do fluxo entre camadas internas. V4 proíbe
símbolos de I/O conhecidos. Nenhuma das duas regras protege L1
de dependências externas (crates npm/pip/crates.io) que não são
I/O mas que ainda assim contaminam o domínio.

```rust
// Passa em V1–V12, mas L1 tornou-se escravo de comemo:
use comemo::Tracked;

pub fn check(file: Tracked<ParsedFile>) -> Vec<Violation> { ... }
```

### Por que listas de bloqueio não funcionam

V4 usa uma lista de tokens proibidos (`std::fs`, `tokio::io`,
etc.). Essa abordagem funciona para I/O porque os símbolos
relevantes são estáveis e em número limitado. Para dependências
externas gerais, a abordagem falha:

- Se proibir `tokio`, a IA usa `async-std`
- Se proibir `comemo`, usa `salsa`
- Se proibir `express` em TypeScript, usa `koa` ou `hono`
- Se proibir `rayon`, usa `crossbeam` ou `std::thread`

A lista de bloqueio é uma corrida armamentista que o maintainer
perde por definição — o ecossistema cresce mais rápido que a
lista.

### A inversão correcta: whitelist em vez de blacklist

A regra topológica real para L1 é:

> L1 é fechada por defeito para o mundo exterior. Apenas imports
> que resolvem para `Layer::L1`, `Layer::L0`, a biblioteca
> padrão da linguagem, e uma lista explícita de utilitários de
> domínio autorizados são permitidos em L1.

Qualquer import que o parser resolve como `Layer::Unknown`
(pacote externo) é submetido à whitelist. Se não estiver na
lista, é Error imediato — independentemente de qual pacote é.

---

## Decisão

### V14 — ExternalTypeInContract

Qualquer import em L1 com `target_layer == Layer::Unknown` que
não esteja na whitelist `[l1_allowed_external]` do
`crystalline.toml` gera violação Error.

**Configuração em `crystalline.toml`:**

```toml
[l1_allowed_external]
rust       = ["serde", "thiserror"]
typescript = []
python     = []
```

A lista é por linguagem. O valor é o nome do pacote — o primeiro
segmento do import antes de `::` (Rust) ou o nome do módulo
(TypeScript, Python).

**Defaults razoáveis para Rust:**

```toml
[l1_allowed_external]
rust = ["thiserror"]
# serde é permitido apenas se L1 precisa de Serialize/Deserialize
# para snapshots (ADR-0003). Adicionar explicitamente se necessário.
```

`thiserror` é o único externo que o projecto actual usa em L1
para definição de erros com `derive(Error)`. A sua presença é
intencionalmente visível na whitelist.

**Mecânica de verificação:**

```
Para cada import em L1 com target_layer == Layer::Unknown:
  1. Extrair o nome do pacote:
     - Rust: primeiro segmento antes de "::" (ex: "serde" de "serde::Serialize")
     - TypeScript: nome do pacote npm (ex: "zod" de "import { z } from 'zod'")
     - Python: nome do módulo raiz (ex: "pydantic" de "from pydantic import BaseModel")
  2. Verificar se está na lista [l1_allowed_external] para a linguagem
  3. Se não estiver → Violation V14 Error
  4. Se estiver → permitido
```

**Biblioteca padrão não é afectada:**

Para Rust, imports que começam com `std::`, `core::` ou `alloc::`
resolvem como `Layer::Unknown` no `LayerResolver` actual (não
começam com `crate::`). V14 deve reconhecer estes prefixos como
isentos antes de verificar a whitelist:

```
Isentos de V14 (nunca verificados contra whitelist):
  Rust:       std::*, core::*, alloc::*
  TypeScript: node:* (quando explicitamente prefixado)
  Python:     módulos detectados como stdlib pelo parser
```

Para Rust, a lista de prefixos isentos (`std`, `core`, `alloc`)
é imutável — faz parte da linguagem. Para TypeScript e Python, a
distinção stdlib vs externo já é feita pelo parser (ADR-0009).

**Nível**: Error — bloqueia CI por padrão.

**Projectos sem `[l1_allowed_external]`:**

Se a secção não existir no `crystalline.toml`, o comportamento
padrão é lista vazia — qualquer externo em L1 é Error. Projectos
existentes devem declarar explicitamente os externos que usam.
Isso é intencional: torna o custo de cada dependência externa
em L1 visível.

---

## Impacto na IR e nos ficheiros

### `CrystallineConfig` — novo campo

```rust
/// Pacotes externos permitidos em L1 por linguagem.
/// Chave: linguagem ("rust", "typescript", "python").
/// Valor: lista de nomes de pacote permitidos.
/// Se ausente, L1 não pode importar nenhum externo.
#[serde(default)]
pub l1_allowed_external: HashMap<String, Vec<String>>,
```

### `Import<'a>` — sem mudança

V14 opera sobre `Import.target_layer == Layer::Unknown` e
`Import.path` — ambos já existem.

### `L1AllowedExternal` — nova struct em L1

```rust
/// Whitelist de pacotes externos permitidos em L1.
/// Construída de config.l1_allowed_external e injectada em V14.
/// L1 nunca lê o toml directamente.
pub struct L1AllowedExternal {
    /// Nome do pacote (primeiro segmento do import).
    allowed: HashSet<String>,
    /// Prefixos sempre isentos (stdlib).
    exempt_prefixes: Vec<String>,
}

impl L1AllowedExternal {
    pub fn is_allowed(&self, package_name: &str) -> bool {
        // Verificar prefixos isentos primeiro
        if self.exempt_prefixes.iter().any(|p| package_name.starts_with(p)) {
            return true;
        }
        self.allowed.contains(package_name)
    }
}
```

---

## Prompts afectados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `linter-core.md` | V14 nas verificações; `[l1_allowed_external]` no toml; `L1AllowedExternal` no pipeline |
| `sarif-formatter.md` | V14 na tabela SARIF e `EnabledChecks` |
| `rules/external-type-in-contract.md` | Novo — V14 |
| `cargo.md` | Nota: `thiserror` em `[l1_allowed_external]` no toml do projecto |

Nota sobre `violation-types.md` e os parsers: V14 não requer
novos campos na IR — opera sobre `Import.target_layer` e
`Import.path` já existentes. Apenas `CrystallineConfig` e
`L1AllowedExternal` são novos.

---

## Consequências

### ✅ Positivas

- **Inversão da lógica de segurança**: fechado por defeito,
  aberto por declaração explícita — correcto para segurança
  arquitectural
- **Resistente a novos pacotes**: qualquer novo framework que
  a IA tente usar em L1 é bloqueado automaticamente, sem
  actualizar listas de bloqueio
- **Custo visível**: cada dependência externa em L1 aparece
  explicitamente no `crystalline.toml` — decisão documentada,
  não acidente
- **Sem impacto em L2/L3/L4**: a whitelist aplica-se apenas
  a L1; infra e wiring podem usar qualquer crate necessário

### ❌ Negativas

- **Migração de projectos existentes**: projectos que já usam
  externos em L1 precisam de declarar a lista antes de activar
  V14 — ou corrigir as dependências
- **Manutenção da whitelist**: a lista deve ser actualizada
  quando o projecto adopta novos utilitários de domínio em L1
  (ex: adicionar `uuid` quando IDs de domínio são introduzidos)
- **Falsos positivos possíveis para re-exports**: se L3 re-exporta
  um tipo externo através de L1 (padrão raro mas possível), V14
  pode disparar em L1 incorrectamente — caso a documentar como
  anti-pattern

### ⚙️ Neutras

- O projecto actual `crystalline-lint` usa apenas `thiserror`
  em L1 — a whitelist inicial é `rust = ["thiserror"]`
- V14 e V3 são complementares: V3 protege contra imports de
  camadas internas na direcção errada; V14 protege contra
  imports externos não autorizados

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Lista de bloqueio no toml | Familiar, fácil de entender | Corrida armamentista; IA encontra sempre novos pacotes |
| Proibir todos os externos em L1 | Zero falsos negativos | Proíbe `thiserror` e outros utilitários legítimos |
| Verificar via análise semântica do tipo | Detecta re-exports e aliases | Complexidade fora do âmbito |
| V14 apenas para tipos em assinaturas públicas | Menos intrusivo | Imports internos de funções privadas também contaminam |

---

## Relação com V4

V4 e V14 são camadas de defesa distintas e complementares:

| Aspecto | V4 | V14 |
|---------|-----|-----|
| O que detecta | Símbolos de I/O conhecidos | Qualquer externo não autorizado |
| Abordagem | Blacklist de símbolos | Whitelist de pacotes |
| Escopo | Tokens em call expressions | Imports no topo do ficheiro |
| Resistência a aliases | Resolve FQN antes (Rust) | Opera no nome do pacote |
| Linguagens | Rust, TypeScript, Python | Rust (fase 1); TS/Python (fase 2) |

V4 permanece necessária para I/O da stdlib (`std::fs`, etc.)
que é da linguagem, não de um pacote externo — nunca apareceria
na whitelist de V14.

---

## Referências

- ADR-0001: Tree-sitter IR — `target_layer == Layer::Unknown`
  como identificador de externos
- ADR-0009: ADR-0009 — resolução física de imports; distinção
  stdlib vs externo para TypeScript e Python
- `rules/impure-core.md` — V4, abordagem complementar
- `file-walker.md` — `resolve_file_layer` retorna `Layer::Unknown`
  para crates externas
