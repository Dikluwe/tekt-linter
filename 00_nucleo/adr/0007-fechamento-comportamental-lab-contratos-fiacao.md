# ⚖️ ADR-0007: Fechamento Comportamental — Lab, Contratos e Fiação

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-16
**Errata**: 2026-03-16 — critério de V11 refinado após materialização revelar premissa incorrecta

---

## 💎 Formalism ($\mathcal{L}_{adr}$)

* **Invariante de Quarentena**: $\forall f \in Files_{L1 \cup L2 \cup L3 \cup L4} :
  \nexists i \in imports(f) : target(i) = Layer::Lab$
* **Invariante de Circuito Fechado**: $\forall t \in Traits_{L1.contracts \setminus L1.access} :
  \exists impl \in Impls_{L2 \cup L3} : impl.trait = t$
* **Invariante de Pureza de Fiação**: $\forall f \in Files_{L4} :
  \nexists d \in declarations(f) : d \in \{struct\_item, enum\_item, impl\_item\}$

---

## Contexto

O ADR-0006 fechou a topologia espacialmente: todo arquivo pertence a
uma camada, todo prompt tem materialização, todo import respeita as
portas de L1. O que permanece aberto são três vulnerabilidades
comportamentais — padrões que o linter atual permite mas que
introduzem dívida estrutural invisível, especialmente em ambientes
com múltiplos agentes de IA operando em paralelo.

### Vulnerabilidade 1 — Quarantine Leak

O diretório `lab/` existe para prototipação sem restrições. A IA
(ou o desenvolvedor) pode escrever código sujo ali sem violar V1–V9.
O problema: nada impede que L1, L2, L3 ou L4 importem de `lab/`.
A IA, ao buscar o caminho de menor resistência para reutilizar um
algoritmo que prototipou no lab, simplesmente adiciona
`use crate::lab::...` no código de produção.

V3 já proíbe que `lab` importe de L4. Mas não existe a direção
inversa — produção importando lab.

### Vulnerabilidade 2 — Dangling Contract

L1 declara traits em `contracts/`. L3 (e ocasionalmente L2)
implementa essas traits. Quando a IA perde contexto ou é
interrompida no meio de uma refatoração, o resultado comum é
uma trait declarada em L1 sem nenhum `impl` correspondente em
L2 ou L3. O contrato existe, mas o circuito está aberto — nenhuma
implementação concreta pode ser injetada em L4.

O linter atual detecta imports para contratos inexistentes (V1)
mas não detecta contratos sem implementação.

### Vulnerabilidade 3 — Wiring Logic Leak

L4 deve ser "burra" — instancia, injeta, orquestra. Não cria tipos.
Quando a IA precisa resolver um problema de formatação, um caso
especial de inicialização ou uma pequena transformação de dados,
o caminho de menor resistência é escrever a lógica diretamente em
`main.rs`. Com o tempo, L4 acumula structs de adaptação, enums
de estado e blocos `impl` que deveriam existir em L2 ou L3.

V3 protege a direção dos imports. Nada protege a densidade de
declarações em L4.

---

## Decisão

### V10 — Quarantine Leak

Qualquer import em arquivo de L1, L2, L3 ou L4 cujo
`target_layer == Layer::Lab` gera violação Fatal.

O lab pode importar produção para testar. A produção nunca importa
o lab. A assimetria é absoluta — não há configuração que permita
exceções, pelo mesmo motivo que V8 é Fatal: código de produção que
depende de código de laboratório não é código de produção.

```rust
pub fn check<'a, T: HasImports<'a>>(file: &T) -> Vec<Violation<'a>> {
    if matches!(file.layer(), Layer::Lab | Layer::L0 | Layer::Unknown) {
        return vec![];
    }
    file.imports()
        .iter()
        .filter(|i| i.target_layer == Layer::Lab)
        .map(|i| make_violation(file, i))
        .collect()
}
```

**Nível**: Fatal — não configurável.

**Nota sobre V3:** V3 já proíbe que L4 importe de Lab via matriz
de permissões. V10 é redundante para L4 mas explicita a semântica
para L1, L2 e L3, e eleva o nível para Fatal em todos os casos.

### V11 — Dangling Contract

#### Premissa inicial e correcção

A especificação original definia: "toda trait pública declarada em
L1/contracts/ deve ter pelo menos um `impl` correspondente em L2
ou L3."

A materialização revelou que existem **dois tipos estruturalmente
distintos** de trait em `contracts/`:

**Tipo 1 — Portas de infraestrutura**
`FileProvider`, `LanguageParser`, `PromptReader`, `PromptSnapshotReader`,
`PromptProvider`. Existem para L3 (ou L2) implementar. O circuito é:
```
L1 declara → L3 implementa → L4 injeta
```
V11 faz sentido aqui — um contrato sem implementação é um circuito aberto.

**Tipo 2 — Contratos de acesso por regra**
`HasImports`, `HasCoverage`, `HasTokens`, `HasHashes`,
`HasPublicInterface`, `HasPubLeak`, `HasWiringPurity`.
Existem para as regras de L1 consumirem `ParsedFile` sem depender
da struct concreta. O circuito é:
```
L1 declara → L1 implementa (via ParsedFile) → L1 consome
```
V11 **não** faz sentido aqui — a implementação legítima está em L1,
não em L2/L3. Tratar estas traits como dangling é um falso positivo
estrutural, não uma excepção a configurar.

#### Solução — separação física em `entities/`

As traits de acesso por regra não são portas — são mecanismos
internos de L1 que permitem testabilidade sem acoplamento directo
a `ParsedFile`. O lugar correcto é `01_core/entities/`, não
`01_core/contracts/`.

**Acção:** mover `rule_traits.rs` de `01_core/contracts/` para
`01_core/entities/rule_traits.rs`. `contracts/` passa a conter
exclusivamente portas de infraestrutura — traits cujo `impl`
pertence a L2 ou L3.

Com esta separação, V11 aplica o critério original sem excepções:
toda trait em `contracts/` deve ter `impl` em L2 ou L3. O critério
é correcto por construção estrutural, não por configuração nominal.

#### Critério de extração actualizado

`RustParser` popula `declared_traits` apenas para ficheiros com
`layer == L1` e path contendo o segmento `"contracts"`. Com
`rule_traits.rs` em `entities/`, nunca será processado por este
critério — a exclusão é estrutural, não baseada em nome de ficheiro.

`implemented_traits` continua restrito a ficheiros em L2 ou L3,
extraindo nomes de traits de nós `impl_item` com campo `trait`.

#### Verificação

Opera sobre `ProjectIndex` após a fase Reduce, junto com V7 e V8.

```rust
pub fn check_dangling_contracts<'a>(
    index: &ProjectIndex<'a>,
) -> Vec<Violation<'a>> {
    index.all_declared_traits
        .iter()
        .filter(|t| !index.all_implemented_traits.contains(*t))
        .map(|trait_name| Violation {
            rule_id: "V11".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Contrato sem implementação: trait '{}' declarada em \
                 L1/contracts/ não tem impl correspondente em L2 ou L3. \
                 O circuito está aberto — nenhuma instância pode ser injetada.",
                trait_name
            ),
            location: Location {
                path: Cow::Owned(PathBuf::from("01_core/contracts")),
                line: 0,
                column: 0,
            },
        })
        .collect()
}
```

**Nível**: Error — bloqueia CI por padrão.

**Limitação declarada**: comparação por nome simples de trait.
Na prática, nomes de portas em `contracts/` são únicos por
convenção. Uma versão futura pode usar FQN se colisões surgirem.

### V12 — Wiring Logic Leak

Opera sobre `ParsedFile` por arquivo, na fase Map.
Verifica arquivos com `layer == L4`.

Nós proibidos em L4:
- `struct_item` — exceto se `allow_adapter_structs = true` (padrão)
- `enum_item` — sempre proibido
- `impl_item` sem trait (`impl Type { ... }`) — sempre proibido
- `impl Trait for Type` — **permitido**, é o padrão de adapter

```toml
[wiring_exceptions]
allow_adapter_structs = true
```

**Nível**: Warning por padrão — configurável para Error.

---

## Impacto na IR e nos ficheiros

### Mudança estrutural — `rule_traits.rs`

| Antes | Depois |
|-------|--------|
| `01_core/contracts/rule_traits.rs` | `01_core/entities/rule_traits.rs` |
| `contracts/mod.rs` exporta `rule_traits` | `entities/mod.rs` exporta `rule_traits` |
| Cada regra importa de `crate::contracts::rule_traits` | Cada regra importa de `crate::entities::rule_traits` |
| `parsed_file.rs` importa de `contracts::rule_traits` | `parsed_file.rs` importa de `entities::rule_traits` |

A mudança é mecânica — apenas paths de import. Nenhuma assinatura
de trait ou implementação muda.

### `LocalIndex` — campos para V11
```rust
pub declared_traits: Vec<&'a str>,
pub implemented_traits: Vec<&'a str>,
```

### `ProjectIndex` — campos para V11
```rust
pub all_declared_traits: HashSet<&'a str>,
pub all_implemented_traits: HashSet<&'a str>,
```

### `Declaration<'a>` e `HasWiringPurity<'a>` — para V12

`Declaration` e `DeclarationKind` vivem em `entities/parsed_file.rs`.
`HasWiringPurity` vive em `entities/rule_traits.rs` após a mudança.

---

## Prompts afectados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `Declaration`, `DeclarationKind`, `WiringConfig`, campos novos em `ParsedFile` |
| `project-index.md` | `declared_traits`, `implemented_traits` em `LocalIndex` e `ProjectIndex` |
| `contracts/rule-traits.md` | Movido para `entities/rule-traits.md`; nota sobre separação semântica |
| `rs-parser.md` | Extração de `declared_traits` restrita a `contracts/`; `implemented_traits` e `declarations` |
| `linter-core.md` | V10–V12; `[wiring_exceptions]`; estrutura de ficheiros actualizada |
| `sarif-formatter.md` | V10, V11, V12 na tabela de regras e `EnabledChecks` |
| `rules/quarantine-leak.md` | V10 — novo |
| `rules/dangling-contract.md` | V11 — critério corrigido, `rule_traits` fora de `contracts/` |
| `rules/wiring-logic-leak.md` | V12 — novo |

---

## Estado de implementação

| Item | Estado |
|------|--------|
| V10 — Quarantine Leak | ✅ Implementado |
| V11 — Dangling Contract (lógica) | ✅ Implementado |
| V11 — `rule_traits.rs` em `entities/` | ✅ Implementado (2026-03-16) |
| V11 — activar no default de `--checks` | ✅ Implementado (2026-03-16) |
| V12 — Wiring Logic Leak | ✅ Implementado |

**Confirmação:** `cargo run -- .` → ✓ No violations found.

---

## Consequências

### ✅ Positivas

- **Quarentena real**: o lab é radioativo em ambas as direções
- **Circuito fechado**: toda trait em `contracts/` tem garantia
  de implementação em L2/L3 — sem excepções, por construção
- **Separação semântica clara**: `contracts/` contém exclusivamente
  portas de infraestrutura; `entities/` contém tanto as entidades
  de domínio como os mecanismos internos de acesso de L1
- **L4 permanece burra**: acumulação de lógica em `main.rs` é
  detectada antes de virar um God Object

### ❌ Negativas

- V11 requer dois passes sobre o AST por arquivo e agregação
  global — aumenta a responsabilidade do `RustParser` e do
  `ProjectIndex`
- Mover `rule_traits.rs` requer actualização de imports em todos
  os ficheiros de regra e em `parsed_file.rs` — refatoração
  mecânica mas com superfície ampla
- Comparação por nome simples em V11 pode ter falsos negativos
  em projectos com nomes de trait duplicados entre módulos

### ⚙️ Neutras

- V10 Fatal — não configurável via `--checks` para suprimir exit code
- V11 Error por padrão — não requer mudança em projectos existentes
  que já têm todos os contratos implementados
- V12 Warning por padrão — friction intencional, não bloqueio imediato

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| V10 como Error em vez de Fatal | Menos agressivo | Produção importando lab é tão grave quanto arquivo fora da topologia |
| V11 com excepções em `crystalline.toml` | Zero mudança estrutural | Excepção disfarça um problema de design — `rule_traits` não pertence a `contracts/` |
| V11 com filtro por nome de ficheiro (`rule_traits.rs`) | Simples | Frágil, nominal, não arquitectural |
| V11 por FQN em vez de nome simples | Zero falsos negativos | Aumenta complexidade sem benefício prático |
| V12 como Error imediato | Força limpeza rápida | L4 com adapters legítimos é aceitável em fases de migração |

---

## Referências

- ADR-0003: Code-to-Prompt Feedback Direction
- ADR-0006: Fechamento Topológico e Proteção de Encapsulamento
- `violation-types.md` — IR actual
- `project-index.md` — `LocalIndex` e `ProjectIndex`
- `entities/rule-traits.md` — traits de acesso por regra (após mudança)
