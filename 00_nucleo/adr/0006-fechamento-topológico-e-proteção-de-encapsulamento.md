# ⚖️ ADR-0006: Fechamento Topológico e Proteção de Encapsulamento

**Status**: `PROPOSTO`
**Data**: 2026-03-14

---

## 💎 Formalism ($\mathcal{L}_{adr}$)

* **Fechamento do Lattice**: Seja $F$ o conjunto de todos os arquivos
  de código no repositório e $M$ o conjunto de diretórios mapeados
  em `crystalline.toml`. O sistema é topologicamente fechado sse
  $\forall f \in F : \exists m \in M : f \in m$.
* **Completude de Nucleação**: Seja $P$ o conjunto de prompts em
  `00_nucleo/prompts/` e $R$ o conjunto de prompts referenciados
  por `@prompt` headers. O sistema é completo sse $P = R$.
* **Invariante de Porta**: Seja $ports(L1)$ o conjunto de
  subdiretórios de L1 designados como portas públicas. Para qualquer
  import de L2 ou L3 em L1: $import.subdir \in ports(L1)$.

---

## Status

`PROPOSTO`

## Data

2026-03-14

---

## Contexto

O sistema atual de verificação (V1–V6) garante rastreabilidade
causal e integridade bidirecional entre prompts e código. Porém,
três buracos estruturais permitem que agentes de IA introduzam
dívida técnica invisível sem disparar nenhuma violação:

**Buraco 1 — Prompts órfãos:**
V1 verifica que todo arquivo de código aponta para um prompt
existente. A direção inversa não existe — prompts sem
materialização correspondente acumulam silenciosamente em L0.
Durante refatorações, código é removido mas o prompt permanece.
A IA propõe novos contratos em L0 e esquece de materializá-los.

**Buraco 2 — Terra de ninguém:**
O walker resolve `Layer::Unknown` silenciosamente para arquivos
fora de diretórios mapeados. Uma IA que cria `src/utils/` ou
`helpers/` produz código fora da gaiola arquitetural sem nenhum
sinal de alerta. O linter não analisa o que não consegue mapear
— o que significa que as regras V1–V6 simplesmente não se aplicam
a esses arquivos.

**Buraco 3 — Pub desnecessário:**
V3 garante a direção do fluxo (L2 não importa L3). Mas não
garante a granularidade. Quando o compilador Rust rejeita
visibilidade, a resposta imediata de qualquer agente é adicionar
`pub` a helpers internos. L2 passa a importar detalhes de
implementação de L1 que nunca deveriam ser visíveis externamente,
criando acoplamento implícito não documentado em nenhum contrato.

---

## Decisão

Introduzir três novas verificações que fecham os buracos
identificados:

### V7 — Orphan Prompt (Semente Estéril)

Verificação bidirecional de nucleação. Para cada arquivo `.md`
em `00_nucleo/prompts/`, verifica se existe pelo menos um arquivo
em L1–L4 com `@prompt` header apontando para ele.

Prompts sem nenhuma materialização são órfãos — violação de
nível **warning** por padrão, configurável para **error**.

**Exceções legítimas declaradas em `crystalline.toml`:**
```toml
[orphan_exceptions]
# Prompts que existem sem materialização por design
"prompts/template.md" = "template — não materializa diretamente"
"prompts/readme.md"   = "gera README.md, não arquivo Rust"
```

### V8 — Alien File (Vácuo Topológico)

Todo arquivo de código (`.rs` e demais extensões habilitadas)
encontrado pelo walker fora de diretórios explicitamente mapeados
em `[layers]` — e fora de diretórios explicitamente excluídos —
gera violação de nível **Fatal**.

A distinção entre excluído e desconhecido é crítica:
```toml
[layers]
# Mapeados — arquivos aqui são analisados
L1 = "01_core"

[excluded]
# Excluídos — arquivos aqui são ignorados intencionalmente
build = "target"
deps  = "node_modules"
```

Arquivo em `src/utils/` não está em `[layers]` nem em
`[excluded]` → V8 Fatal.

### V9 — Pub Leak (Solução Preguiçosa)

Imports de L2 ou L3 em L1 são válidos apenas se apontarem para
subdiretórios designados como portas em `crystalline.toml`.
```toml
[l1_ports]
# Subdiretórios de L1 acessíveis externamente
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"
```

Import de L2 ou L3 apontando para qualquer subdiretório de L1
não listado em `[l1_ports]` gera violação de nível **error**.

Isso força que helpers, utilitários internos e implementações
auxiliares de L1 permaneçam inacessíveis sem que o agente
precise explicitamente adicionar o subdiretório às portas —
uma decisão humana deliberada.

---

## Impacto na IR (ParsedFile)

V7 não opera sobre `ParsedFile` — opera sobre dois conjuntos
globais que L3 entrega ao pipeline depois de varrer todos os
arquivos. Requer nova entidade em L1:
```rust
pub struct ProjectIndex<'a> {
    /// Todos os prompts em 00_nucleo/prompts/ (exceto exceções)
    pub all_prompts: HashSet<&'a str>,
    /// Todos os prompt_paths referenciados em @prompt headers
    pub referenced_prompts: HashSet<&'a str>,
    /// Todos os arquivos em Layer::Unknown fora de excluídos
    pub alien_files: Vec<&'a Path>,
}
```

V7 e V8 recebem `ProjectIndex`, não `ParsedFile`.
V9 recebe `ParsedFile` com `Import.subdir: Option<&'a str>`
adicionado — subdiretório resolvido por L3.

## Impacto em `Import`
```rust
pub struct Import<'a> {
    pub path: &'a str,
    pub line: usize,
    pub kind: ImportKind,
    pub target_layer: Layer,
    pub target_subdir: Option<&'a str>, // novo — para V9
    // None se target_layer == Unknown (crate externa)
    // Some("entities") se import aponta para 01_core/entities/
}
```

---

## Prompts Afetados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | `ProjectIndex`, `Import.target_subdir` |
| `contracts/file-provider.md` | `ProjectIndex` como segundo produto do walker |
| `file-walker.md` | construção de `ProjectIndex` durante varredura |
| `rs-parser.md` | resolução de `target_subdir` em imports |
| `linter-core.md` | V7, V8, V9 nas verificações, `[excluded]` e `[l1_ports]` no toml |
| `cargo.md` | `crystalline.toml` atualizado |
| `rules/orphan-prompt.md` | novo — V7 |
| `rules/alien-file.md` | novo — V8 |
| `rules/pub-leak.md` | novo — V9 |

---

## Consequências

### ✅ Positivas

- **Fechamento completo**: não existe arquivo de código fora
  da topologia — a gaiola é hermética
- **Nucleação bidirecional**: L0 não acumula prompts mortos —
  toda semente deve ter fruto ou ser explicitamente excluída
- **Encapsulamento real**: agentes não podem usar `pub` como
  atalho para contornar fronteiras — as portas de L1 são
  declaradas explicitamente por humanos

### ❌ Negativas

- `ProjectIndex` requer que o pipeline complete a varredura
  antes de verificar V7 e V8 — não é mais possível reportar
  violações em streaming para esses dois casos
- `[l1_ports]` exige decisão humana explícita sobre o que é
  público em L1 — adiciona friction intencional
- `[excluded]` e `[orphan_exceptions]` adicionam superfície
  de configuração ao `crystalline.toml`

### ⚙️ Neutras

- V8 Fatal significa que `--checks` não pode suprimir V8
  (mesmo comportamento de V0)
- V7 Warning por padrão — não quebra projetos existentes
  na adoção inicial
- V9 reusa `Import` já existente com campo adicional —
  sem nova entidade de domínio

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| V8 como Error em vez de Fatal | Menos agressivo | Terra de ninguém é tão grave quanto arquivo ilegível — código não analisado é garantia falsa |
| V7 baseado em timestamp em vez de referência | Detecta prompts antigos | Frágil em CI, sensível a git checkout |
| V9 via visibilidade Rust (`pub(crate)`) | Enforce pelo compilador | Não resolve o problema — o compilador aceita `pub` em qualquer lugar |

---

## Referências

- ADR-0001: Tree-sitter Intermediate Representation
- ADR-0002: Code-to-Prompt Feedback Direction
- ADR-0004: Reformulação do Motor de Análise
- `violation-types.md` — IR atual
- `file-walker.md` — walker atual
