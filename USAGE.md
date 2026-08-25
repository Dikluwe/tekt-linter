# crystalline-lint — Guia de Uso

**Versão**: 0.1.0  
**Público**: Agentes de IA a inicializar ou trabalhar em projectos Cristalinos  
**Repositório**: https://github.com/Dikluwe/tekt-linter

---

## O que é

`crystalline-lint` é um linter arquitectural que verifica se um projecto
segue a Arquitetura Cristalina (Tekt). Analisa ficheiros Rust, TypeScript,
Python, C, C++, Go, Zig, Java e Elixir e reporta violações estruturais em SARIF ou texto.

O linter valida o próprio código contra as suas próprias regras — se o
projecto usa `crystalline-lint`, o linter deve passar com zero violations
no próprio projecto (`cargo run -- .` ou `crystalline-lint .`).

---

## Instalação

### Binário pré-compilado (recomendado)

```bash
# Linux x86_64
curl -L https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-linux-x86_64 \
  -o crystalline-lint && chmod +x crystalline-lint

# macOS ARM
curl -L https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-macos-arm64 \
  -o crystalline-lint && chmod +x crystalline-lint
```

Mover para `$PATH`:
```bash
mv crystalline-lint /usr/local/bin/
```

### Via cargo

```bash
cargo install --git https://github.com/Dikluwe/tekt-linter crystalline-lint
```

### Verificar instalação

```bash
crystalline-lint --version
# crystalline-lint 0.1.0
```

---

## Estrutura de directórios obrigatória

Um projecto Cristalino deve seguir esta topologia de camadas:

```
projecto/
├── 00_nucleo/          ← L0: prompts e ADRs (fonte de verdade)
│   ├── prompts/        ← prompts L0 que nucleiam o código
│   └── adr/            ← decisões arquitecturais
├── 01_core/            ← L1: lógica pura (sem I/O)
│   ├── entities/       ← tipos de domínio
│   ├── contracts/      ← traits/interfaces/Protocols
│   └── rules/          ← lógica de negócio
├── 02_shell/           ← L2: CLI, formatadores, adaptadores de entrada
├── 03_infra/           ← L3: I/O, parsers, filesystem, HTTP
├── 04_wiring/          ← L4: composição, injeção de dependências
│   └── main.rs / index.ts / main.py
├── lab/                ← experiências — nunca importado por produção
├── crystalline.toml    ← configuração do linter
└── [Cargo.toml / package.json / pyproject.toml]
```

**Regras topológicas:**

| Camada | Pode importar | Não pode importar |
|--------|--------------|-------------------|
| L1 | L1, stdlib pura | L2, L3, L4, externos não autorizados |
| L2 | L1, L2 | L3, L4 |
| L3 | L1, L3 | L2, L4 |
| L4 | L1, L2, L3, L4 | — |
| lab | qualquer | — (mas produção não pode importar lab) |

`Layer::Unknown` (ficheiros fora da topologia) dispara V8 Fatal.

---

## crystalline.toml — configuração completa

Criar na raiz do projecto:

```toml
[project]
root = "."

[languages]
# Activar apenas as linguagens usadas no projecto
rust       = { grammar = "tree-sitter-rust",       enabled = true }
typescript = { grammar = "tree-sitter-typescript", enabled = true }
python     = { grammar = "tree-sitter-python",     enabled = false }

[layers]
# Mapear nome da camada para nome do directório
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

[excluded]
# Directórios a excluir completamente da análise (por nome de componente)
build  = "target"        # Rust
deps   = "node_modules"  # TypeScript
vcs    = ".git"
cache  = ".cargo"
dist   = "dist"          # TypeScript build output

[excluded_files]
# Ficheiros individuais excluídos por path relativo à raiz.
# Usar apenas para ficheiros fora da topologia por razões estruturais
# da linguagem (ex: lib.rs em Rust, index.ts em TypeScript na raiz).
# crate_root = "lib.rs"     ← Rust: ponto de reexport da crate
# ts_entry   = "index.ts"   ← TypeScript: se existir na raiz

[module_layers]
# Mapear nomes de módulo para camadas (usado pelo RustParser)
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

[l1_ports]
# Subdirectórios de L1 que são "portas" públicas (para V9)
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"

[ts_aliases]
# Aliases TypeScript — devem corresponder ao tsconfig.json compilerOptions.paths
# Sem aliases: linter aceita apenas imports relativos (./  ../) em TS
# "@core"  = "01_core"
# "@shell" = "02_shell"
# "@infra" = "03_infra"

[py_aliases]
# Aliases Python — módulos que mapeiam para directórios da topologia
# "core"  = "01_core"
# "shell" = "02_shell"
# "infra" = "03_infra"

[orphan_exceptions]
# Prompts em 00_nucleo/prompts/ que não materializam código directamente
# "00_nucleo/prompts/readme_prompt.md" = "gera README.md, não código"

[wiring_exceptions]
# Em L4, classes que implementam contratos (adapters) são permitidas
allow_adapter_structs = true

[l1_allowed_external]
# Pacotes externos explicitamente autorizados em L1 (whitelist V14).
# L1 é fechada por defeito — qualquer externo não listado é Error.
# Manter esta lista pequena: cada entrada é uma dependência de domínio.
rust       = ["thiserror"]
typescript = []           # ex: ["zod"] se usar validação de domínio
python     = []

[rules]
V0  = { level = "fatal" }
V1  = { level = "error" }
V2  = { level = "error" }
V3  = { level = "error" }
V4  = { level = "error" }
V5  = { level = "warning" }
V6  = { level = "warning" }
V7  = { level = "warning" }
V8  = { level = "fatal" }
V9  = { level = "error" }
V10 = { level = "fatal" }
V11 = { level = "error" }
V12 = { level = "warning" }
V13 = { level = "error" }
V14 = { level = "error" }
```

---

## Header de linhagem — obrigatório em cada ficheiro

Todo ficheiro em L1–L4 deve começar com um header de linhagem.
Sem este header, V1 dispara (Error).

**Rust** (comentário de módulo `//!`):
```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```

**TypeScript** (comentários de linha `//` em bloco contíguo no topo):
```typescript
// Crystalline Lineage
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
```

**Python** (comentários `#` em bloco contíguo no topo):
```python
# Crystalline Lineage
# @prompt 00_nucleo/prompts/<nome>.md
# @prompt-hash <sha256[0..8]>
# @layer L<n>
# @updated YYYY-MM-DD
```

**Campos:**
- `@prompt` — path relativo à raiz do projecto para o prompt L0 que nucleou este ficheiro
- `@prompt-hash` — primeiros 8 caracteres do SHA-256 do conteúdo do prompt (actualizado com `--fix-hashes`)
- `@layer` — camada arquitectural: `L0`, `L1`, `L2`, `L3`, `L4`, ou `lab`
- `@updated` — data da última materialização

---

## Aliases TypeScript — sincronização com tsconfig.json

Se o projecto TypeScript usa path aliases, declarar em **ambos** os ficheiros:

**crystalline.toml:**
```toml
[ts_aliases]
"@core"  = "01_core"
"@shell" = "02_shell"
"@infra" = "03_infra"
```

**tsconfig.json:**
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@core/*":  ["./01_core/*"],
      "@shell/*": ["./02_shell/*"],
      "@infra/*": ["./03_infra/*"]
    }
  }
}
```

O `crystalline.toml` é a fonte de verdade para o linter — não lê o
`tsconfig.json` directamente. Se os dois estiverem dessincronizados,
o linter pode resolver imports para `Layer::Unknown` incorrectamente,
fazendo V14 disparar para imports legítimos.

Sem aliases configurados, apenas imports relativos (`./` e `../`) são
resolvidos fisicamente. Qualquer import que não começa com `./`, `../`
ou um alias configurado é tratado como pacote externo (`Layer::Unknown`).

---

## Comandos CLI

A sinopse raiz abaixo descreve o lint. Os fluxos também incluem nominalmente os
subcomandos `snapshot`, `refine` e `refine-revisions`; a gramática CLI global será
reconciliada em F09.

```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projecto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text              [padrão: text]
  --fail-on <level>      error | warning           [padrão: error]
  --checks <list>        v0,v1,...,v25             [padrão: V1–V21,V23–V25]
  --no-drift             desactiva V5 (prompt drift)
  --no-stale             desactiva V6 (prompt stale)
  --quiet                apenas exit code, sem output
  --config <path>        path para crystalline.toml [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --fix-hashes --dry-run preview sem escrever
  --update-snapshot      actualiza Interface Snapshot nos prompts (V6)
  --update-snapshot --dry-run  preview sem escrever
```

**Exit codes:**
- `0` — zero violations no nível configurado em `--fail-on`
- `1` — violations presentes, ou erro de configuração

**Exemplos comuns:**

```bash
# Verificar o projecto actual
crystalline-lint .

# Verificar com output SARIF (para CI / GitHub Code Scanning)
crystalline-lint --format sarif . > results.sarif

# Corrigir hashes após editar prompts
crystalline-lint --fix-hashes .

# Verificar apenas regras de importação
crystalline-lint --checks v3,v9,v14 .

# Não falhar em warnings — apenas errors e fatals
crystalline-lint --fail-on error .
```

### Contratos de preservação semântica (V23–V25)

As três regras são conservadoras: sem contrato explícito, não emitem diagnóstico.

```toml
[[semantic.context]]
id = "rounded-radius"
language = "rust"
scopes = ["03_infra/export.rs::draw"]
sources = ["radius"]
resolvers = [{ symbol = "resolve_pt", context_arg = 0 }]
erasing_projections = ["abs"]
sinks = ["rounded_rect"]
absolute_sources = ["absolute_radius"]

[[semantic.projection]]
id = "font-identity-variations"
language = "rust"
scope = "03_infra/font.rs::identity"
source = "style.variations"
destination = "return.2"
neutral_forms = ["default", "none", "zero"]
normalization = "preserve"

[[semantic.decision]]
id = "math-authority"
language = "rust"
owner = "03_infra/style.rs::is_math"
consumers = ["03_infra/export.rs::*"]
explicit_sources = ["style.math"]
proxies = ["contains(\"math\")"]
duplicate_owners = []
canonicalizers = ["map_glyph"]
resolved_after = ["03_infra/export.rs::emit"]
```

V23 cobre contexto neutro e projeção apagadora; V24 cobre slots de retorno declarados;
V25 cobre owner duplicado, proxy e canonicalizador após marco resolvido. A primeira
versão é intraprocedural e Rust-only. Configuração incompleta, destino não suportado ou
IDs duplicados falham como erro de configuração.

### Comparar snapshots por refinamento

`refine` opera somente sobre arquivos de snapshot fornecidos e não executa Git.

Gerar um snapshot Rust determinístico:

```bash
crystalline-lint snapshot . \
  --contract refinement.toml \
  --artifact-id working-tree \
  --output working-tree.refinement.json
```

Cada `[[observable]]` do contrato declara `key`, `language = "rust"`, `file`, query
tree-sitter, `capture`, `cardinality = "one" | "many"` e
`on_missing = "unknown" | "absent"`. O arquivo deve ser relativo e permanecer dentro
da raiz mesmo após resolver symlinks. Capturas múltiplas são ordenadas; o snapshot não
contém relógio e duas extrações iguais produzem os mesmos bytes.

```bash
crystalline-lint refine \
  --before before.refinement.json \
  --after after.refinement.json \
  --contract refinement.toml \
  --format text
```

#### Comparar revisões Git locais

```bash
crystalline-lint refine-revisions <repository-root> \
  --before-ref <sha-ou-ref> --after-ref <sha-ou-ref> \
  --contract refinement.toml
```

O requisito externo é Git local 2.43 ou compatibilidade demonstrada com
`--batch-command` e `--end-of-options`. Cada ref é resolvida uma única vez para commit
OID; somente esses OIDs imutáveis participam da enumeração, extração, identidade e
testemunhas. O adapter usa argumentos separados, nunca shell, e lê somente blobs
regulares e objetos locais. Não usa rede, fetch ou protocolos externos, não executa
build, não grava temporários de diagnóstico, não executa nem atravessa hooks, filtros,
textconv, LFS, symlinks ou submódulos e não altera checkout, working tree, índice, HEAD,
branch, refs ou stash.

Os budgets são 512 paths observáveis, 4 MiB por blob, 32 MiB por revisão e 10 segundos
por operação Git. Arquivo ausente no tree segue `on_missing`; ref inexistente é erro de
entrada. Objeto esperado ausente ou ilegível, symlink, submódulo, framing inválido ou
budget excedido produz erro de entrada ou `Unknown`, nunca ausência conhecida. Nenhum
resultado pode ser publicado a partir de conteúdo truncado. Para o mesmo conteúdo, o
resultado deve ser equivalente a `snapshot + refine`.

Os resultados semânticos possíveis são `Preserved`, `Violated`, `Unknown` e erro de
entrada. A definição de códigos numéricos e precedência de `refine-revisions` pertence
a F09; este guia não altera a tabela geral de exits do lint nem os exits já publicados
de `refine`.

Snapshot JSON, com ausência conhecida diferente de evidência desconhecida:

```json
{
  "format_version": 1,
  "artifact_id": "before",
  "extractor_version": "manual-1",
  "observables": {
    "style.variations": { "state": "known", "value": "wght=650" },
    "decision.proxy": { "state": "absent" },
    "macro.value": { "state": "unknown", "reason": "opaque-construction" }
  }
}
```

Contrato TOML:

```toml
id = "font-identity"

[[observable]]
key = "style.variations"
language = "rust"
file = "03_infra/font.rs"
query = '''
(field_expression field: (field_identifier) @value)
'''
capture = "value"
cardinality = "many"
on_missing = "unknown"

[[relation]]
kind = "preserve"
source = "style.variations"
target = "identity.variations"

[[relation]]
kind = "may-normalize"
source = "style.weight"
target = "identity.weight"
accepted_targets = ["700"]

[[relation]]
kind = "must-not-invent"
target = "decision.proxy"
```

Razões `unknown` aceitas: `missing-observable`, `ambiguous-identity`,
`unsupported-parser`, `opaque-construction`, `partial-contract` e
`budget-exhausted`. Exit codes: 0 preservado, 1 violado, 2 inconclusivo ou entrada
inválida. O formato `sarif` usa `REFINEMENT` e `REFINEMENT_UNKNOWN` sem reservar um
número `V*`.

**Combinações inválidas (exit 1 imediato):**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

---

## Violations — referência completa

### Fatais — bloqueiam CI incondicionalmente

| ID | Nome | Causa | Resolução |
|----|------|-------|-----------|
| V0 | UnreadableSource | Ficheiro não pode ser lido (permissões, encoding) | Corrigir permissões ou encoding |
| V8 | AlienFile | Ficheiro fora de `[layers]`, `[excluded]` e `[excluded_files]` | Mapear o directório em `[layers]` ou adicionar a `[excluded]` |
| V10 | QuarantineLeak | Import de código de produção para `lab/` | Remover o import; `lab/` é unidireccional |

### Errors — bloqueiam CI com `--fail-on error` (padrão)

| ID | Nome | Causa | Resolução |
|----|------|-------|-----------|
| V1 | MissingPromptHeader | Ficheiro em L1–L4 sem header `@prompt` / `@layer` | Adicionar header de linhagem |
| V2 | MissingTestFile | Ficheiro L1 sem cobertura de teste | Adicionar testes; ou ficheiro é declaration-only (isento automaticamente) |
| V3 | ForbiddenImport | Import que viola a topologia de camadas (ex: L1 importa L3) | Reorganizar a dependência ou mover o ficheiro para a camada correcta |
| V4 | ImpureCore | Símbolo de I/O em L1 (ex: `std::fs`, `fetch`, `open`) | Mover I/O para L3; injectar via trait em L1 |
| V9 | PubLeak | Import de L1 fora das portas declaradas em `[l1_ports]` | Mover o símbolo para um subdirectório declarado como porta |
| V11 | DanglingContract | Trait/interface/Protocol em L1/contracts/ sem implementação em L2/L3 | Implementar o contrato ou remover a declaração |
| V13 | MutableStateInCore | `static mut`, `Mutex`, `OnceLock`, `LazyLock`, `AtomicXxx` em L1 | Injectar estado por parâmetro; mover singleton para L3 |
| V14 | ExternalTypeInContract | Import externo em L1 não declarado em `[l1_allowed_external]` | Adicionar à whitelist se legítimo; ou mover dependência para L3 |

### Warnings — não bloqueiam CI por padrão

| ID | Nome | Causa | Resolução |
|----|------|-------|-----------|
| V5 | PromptDrift | Hash do prompt em `@prompt-hash` diverge do ficheiro actual | Correr `crystalline-lint --fix-hashes .` |
| V6 | PromptStale | Interface pública do ficheiro diverge do snapshot no prompt | Correr `crystalline-lint --update-snapshot .` após validar a mudança |
| V7 | OrphanPrompt | Prompt em `00_nucleo/prompts/` sem materialização correspondente | Materializar o prompt ou adicionar a `[orphan_exceptions]` |
| V12 | WiringLogicLeak | Declaração de tipo em L4 que não é adapter (`class implements` / `impl Trait`) | Mover o tipo para L1 ou L3 conforme o caso |

---

## Integração com CI — GitHub Actions

```yaml
# .github/workflows/architecture.yml
name: Architecture

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install crystalline-lint
        run: |
          curl -L https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-linux-x86_64 \
            -o crystalline-lint && chmod +x crystalline-lint
          sudo mv crystalline-lint /usr/local/bin/

      - name: Run architectural linter
        run: crystalline-lint --format sarif . > results.sarif

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
        if: always()
```

Para falhar o CI em warnings (mais restritivo):
```yaml
- run: crystalline-lint --fail-on warning .
```

---

## Inicializar um projecto novo

Sequência mínima para um projecto TypeScript com a Arquitetura Cristalina:

```bash
# 1. Criar estrutura de directórios
mkdir -p 00_nucleo/{prompts,adr}
mkdir -p 01_core/{entities,contracts,rules}
mkdir -p 02_shell
mkdir -p 03_infra
mkdir -p 04_wiring
mkdir -p lab

# 2. Criar crystalline.toml (ver secção acima)
# 3. Criar tsconfig.json com paths (ver secção de aliases)

# 4. Verificar estrutura inicial
crystalline-lint .
# Esperado: apenas V1 (headers em falta) e V7 (prompts em falta)
# V8 não deve disparar — todos os directórios mapeados em [layers]

# 5. Adicionar headers a cada ficheiro criado
# 6. Criar prompts L0 em 00_nucleo/prompts/ para cada módulo

# 7. Após editar qualquer prompt:
crystalline-lint --fix-hashes .

# 8. Estado limpo:
crystalline-lint .
# ✓ No violations found
```

---

## Fluxo de trabalho com IA

Quando uma IA (Claude Code ou similar) materializa código neste projecto:

1. **Antes de escrever código**: ler o prompt L0 correspondente em `00_nucleo/prompts/`
2. **Escrever testes primeiro** a partir dos critérios do prompt — verificar que falham
3. **Implementar** para os testes passarem
4. **Adicionar header de linhagem** ao ficheiro com `@layer` correcto
5. **Verificar**: `crystalline-lint .` — deve retornar zero violations
6. **Se V5 disparar**: `crystalline-lint --fix-hashes .`

Se não existir prompt L0 para o módulo a criar:
- **Parar** — não criar código sem prompt
- Propor o prompt ao developer antes de materializar

A ausência de prompt não é um detalhe — é uma violação do contrato da arquitectura.
