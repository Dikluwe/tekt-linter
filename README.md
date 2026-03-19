# crystalline-lint

> Linter arquitetural para projetos que seguem a [Arquitetura Cristalina](https://github.com/Dikluwe/crystalline-architecture-standard).

Sem este linter, as regras estruturais são sugestões. Com ele,
violações se tornam ruído visível no CI, no editor e no terminal
— antes de virarem dívida técnica.

---

## Instalação

**Via Cargo:**
```bash
cargo install crystalline-lint
```

**Binário para CI (GitHub Releases):**
```bash
curl -sSL https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-linux-x86_64 \
  -o crystalline-lint && chmod +x crystalline-lint
```

---

## Uso rápido

```bash
# Verificar o projeto no diretório atual
crystalline-lint .

# Saída SARIF para GitHub Code Scanning
crystalline-lint --format sarif . > results.sarif

# Corrigir hashes de prompt desatualizados (V5)
crystalline-lint --fix-hashes .

# Atualizar snapshots de interface desatualizados (V6)
crystalline-lint --update-snapshot .

# Preview de qualquer correção sem reescrever
crystalline-lint --fix-hashes --dry-run .
crystalline-lint --update-snapshot --dry-run .
```

---

## Verificações

| ID | Nome | Nível | Descrição |
|----|------|-------|-----------|
| V0 | `UnreadableSource` | **fatal** | Arquivo ilegível. Bloqueia CI incondicionalmente — não configurável |
| V1 | `MissingPromptHeader` | **error** | Arquivo em L1–L4 sem `//! @prompt` ou com prompt referenciado inexistente |
| V2 | `MissingTestFile` | **error** | Arquivo em L1 sem cobertura de teste detectável no AST nem ficheiro de teste adjacente. Arquivos apenas declarativos são isentos (Rust: `#[cfg(test)]` ou `_test.rs`; TypeScript: `.test.ts`/`.spec.ts`; Python: `_test.py`/`test_*.py`) |
| V3 | `ForbiddenImport` | **error** | Import viola a direção do fluxo de dependência entre camadas |
| V4 | `ImpureCore` | **error** | Símbolo de I/O detectado em L1 via AST. Lista de símbolos proibidos seleccionada por linguagem (`forbidden_symbols_for(language)`) — aliases de importação não burlam a regra em nenhuma linguagem |
| V5 | `PromptDrift` | **warning** | Hash em `@prompt-hash` diverge do hash real do prompt em `00_nucleo/` |
| V6 | `PromptStale` | **warning** | Interface pública do código mudou desde o snapshot registrado no prompt de origem |
| V7 | `OrphanPrompt` | **warning** | Prompt em `00_nucleo/prompts/` sem nenhum arquivo em L1–L4 referenciando-o |
| V8 | `AlienFile` | **fatal** | Arquivo de código fora de todos os diretórios mapeados. Bloqueia CI incondicionalmente — não configurável |
| V9 | `PubLeak` | **error** | Import de L2 ou L3 acessa subdiretório interno de L1 não listado em `[l1_ports]` |
| V10 | `QuarantineLeak` | **fatal** | Arquivo de produção (L1–L4) importa de `lab/`. Bloqueia CI incondicionalmente — não configurável |
| V11 | `DanglingContract` | **error** | Trait em `L1/contracts/` sem `impl` correspondente em L2 ou L3. Verificado globalmente após análise completa |
| V12 | `WiringLogicLeak` | **warning** | `struct`, `enum` ou `impl` sem trait declarado em L4. L4 não cria tipos — apenas liga os que existem |

**Sobre níveis Fatal (V0, V8, V10):** a ausência de violações garante
que todos os arquivos foram lidos e analisados com sucesso. Fatal
não pode ser suprimido por `--fail-on` — bloqueia CI
independentemente de qualquer configuração.

**Sobre V4:** a lista de símbolos proibidos é seleccionada por linguagem
via `forbidden_symbols_for(language)`. Em Rust, aliases de importação
são resolvidos para FQN antes da verificação — `use std::fs as f; f::read(...)`
é detectado como `std::fs::read`. Em TypeScript e Python, call expressions
e imports proibidos são verificados directamente sobre o AST.

**Sobre V11:** opera sobre o índice global do projeto após a análise
paralela de todos os arquivos — não por arquivo individual.

---

## Flags CLI

```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text                   [padrão: text]
  --fail-on <level>      error | warning                [padrão: error]
  --checks <list>        v0,v1,...,v12                  [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml               [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --update-snapshot      atualiza Interface Snapshot nos prompts (V6)
  --dry-run              usado com --fix-hashes ou --update-snapshot
  -h, --help             exibe ajuda
  -V, --version          exibe versão
```

**Combinações inválidas:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Nota sobre V0, V8 e V10:** `--checks` pode omitir estas regras
para suprimir output, mas os três Fatal sempre bloqueiam CI
independentemente de `--fail-on`.

---

## crystalline.toml

```toml
[project]
root = "."

[languages]
rust       = { grammar = "tree-sitter-rust",       enabled = true }
typescript = { grammar = "tree-sitter-typescript", enabled = true }
python     = { grammar = "tree-sitter-python",     enabled = true }

# Mapeamento de diretório → camada
[layers]
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

# Diretórios ignorados intencionalmente — não disparam V8
[excluded]
build = "target"
deps  = "node_modules"
vcs   = ".git"
cargo = ".cargo"

# Mapeamento de módulo Rust → camada (para imports crate::)
[module_layers]
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

# Portas públicas de L1 — imports de outros subdiretórios disparam V9
[l1_ports]
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"

# Prompts sem materialização de código — isentos de V7
[orphan_exceptions]
"00_nucleo/prompts/cargo.md"             = "gera Cargo.toml, não arquivo de código"
"00_nucleo/prompts/readme_prompt.md"     = "gera README.md, não arquivo de código"
"00_nucleo/prompts/parsers/_template.md" = "contrato editorial, não materializa directamente"

# Aliases TypeScript — opcional
[ts_aliases]
# "@core"  = "01_core"
# "@shell" = "02_shell"
# "@infra" = "03_infra"

# Aliases Python — opcional
[py_aliases]
# "core"  = "01_core"
# "shell" = "02_shell"
# "infra" = "03_infra"

# Exceções para V12 — declarações permitidas em L4
[wiring_exceptions]
allow_adapter_structs = true  # structs de adapter são comuns em L4

# Severidade por regra — Fatal não é configurável
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
```

---

## Header canônico

Todo arquivo em L1–L4 deve conter o seguinte cabeçalho no topo:

**Rust** — comentário de módulo `//!`:
```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```

**TypeScript** — comentário de linha `//` em bloco contíguo:
```typescript
// Crystalline Lineage
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
```

**Python** — comentário de linha `#` em bloco contíguo:
```python
# Crystalline Lineage
# @prompt 00_nucleo/prompts/<nome>.md
# @prompt-hash <sha256[0..8]>
# @layer L<n>
# @updated YYYY-MM-DD
```

`@prompt-hash` contém os primeiros 8 caracteres do SHA256 do
arquivo de prompt correspondente. Use `--fix-hashes` para manter
os hashes atualizados após revisões em `00_nucleo/`.

---

## Workflow com --fix-hashes (V5)

Após revisar um prompt em `00_nucleo/`, os arquivos derivados
ficam com hash desatualizado e V5 dispara:

```bash
# 1. Ver quais arquivos serão corrigidos
crystalline-lint --fix-hashes --dry-run .

# 2. Aplicar correções
crystalline-lint --fix-hashes .

# 3. Verificar que zero V5 restam
crystalline-lint .
```

---

## Workflow com --update-snapshot (V6)

Após modificar a interface pública de um arquivo, V6 dispara
porque o snapshot no prompt de origem ficou desatualizado:

```bash
# 1. Ver quais prompts seriam atualizados
crystalline-lint --update-snapshot --dry-run .

# 2. Atualizar os snapshots
crystalline-lint --update-snapshot .

# 3. Verificar que zero V6 restam
crystalline-lint .
```

V6 detecta mudanças de assinatura além de adições e remoções —
`foo(a: String)` → `foo(a: Vec<String>)` é uma quebra de contrato
e dispara V6 mesmo com o nome da função inalterado.

---

## Auto-validação

```bash
# O linter deve passar em sua própria validação sem nenhuma violação
crystalline-lint .
# ✓ No violations found
```

Este é o critério de verificação mais importante — se o linter
não consegue validar seu próprio código com V0–V12 activos,
há um problema estrutural no projeto.

---

## Integração CI

### GitHub Actions

```yaml
name: Crystalline Integrity

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install crystalline-lint
        run: |
          curl -sSL https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-linux-x86_64 \
            -o crystalline-lint && chmod +x crystalline-lint

      - name: Run linter
        run: ./crystalline-lint --format sarif . > results.sarif

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
        if: always()
```

O SARIF é enviado ao GitHub Code Scanning — violações aparecem
como anotações diretamente no diff do PR. V0, V8 e V10 Fatal
aparecem como erros de nível máximo.

---

## Estrutura do projeto

O linter é ele mesmo um projeto Cristalino.

```
crystalline-lint/
├── 00_nucleo/                        # Prompts e ADRs (A Semente)
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── project-index.md
│   │   ├── cargo.md
│   │   ├── readme_prompt.md
│   │   ├── parsers/
│   │   │   ├── _template.md          # contrato editorial
│   │   │   ├── rust.md
│   │   │   ├── typescript.md
│   │   │   └── python.md
│   │   ├── contracts/
│   │   │   ├── file-provider.md
│   │   │   ├── language-parser.md
│   │   │   ├── parse-error.md
│   │   │   ├── prompt-reader.md
│   │   │   ├── prompt-snapshot-reader.md
│   │   │   └── prompt-provider.md
│   │   ├── rules/
│   │   │   ├── prompt-header.md      (V1)
│   │   │   ├── test-file.md          (V2)
│   │   │   ├── forbidden-import.md   (V3)
│   │   │   ├── impure-core.md        (V4)
│   │   │   ├── prompt-drift.md       (V5)
│   │   │   ├── prompt-stale.md       (V6)
│   │   │   ├── orphan-prompt.md      (V7)
│   │   │   ├── alien-file.md         (V8)
│   │   │   ├── pub-leak.md           (V9)
│   │   │   ├── quarantine-leak.md    (V10)
│   │   │   ├── dangling-contract.md  (V11)
│   │   │   └── wiring-logic-leak.md  (V12)
│   │   ├── file-walker.md
│   │   ├── prompt-walker.md
│   │   ├── sarif-formatter.md
│   │   └── fix-hashes.md
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       ├── 0002-code-to-prompt-feedback-direction.md
│       ├── 0004-motor-reformulation.md
│       ├── 0005-location-owned-paths-cargo-nucleation.md
│       ├── 0006-topological-closure.md
│       ├── 0007-fechamento-comportamental.md
│       ├── 0008-estrategia-de-distribuicao.md
│       └── 0009-suporte-typescript-python.md
│
├── 01_core/                          # Lógica pura — zero I/O
│   ├── entities/
│   │   ├── parsed_file.rs            # IR principal + ImportKind semântico
│   │   ├── project_index.rs          # LocalIndex + ProjectIndex
│   │   ├── rule_traits.rs            # HasImports, HasTokens (+ language()), HasWiringPurity...
│   │   ├── violation.rs
│   │   └── layer.rs
│   ├── contracts/                    # Portas de infraestrutura
│   │   ├── file_provider.rs
│   │   ├── language_parser.rs
│   │   ├── parse_error.rs
│   │   ├── prompt_reader.rs
│   │   ├── prompt_snapshot_reader.rs
│   │   └── prompt_provider.rs
│   └── rules/                        # V1–V12
│       ├── prompt_header.rs          (V1)
│       ├── test_file.rs              (V2)
│       ├── forbidden_import.rs       (V3)
│       ├── impure_core.rs            (V4) # forbidden_symbols_for(language)
│       ├── prompt_drift.rs           (V5)
│       ├── prompt_stale.rs           (V6)
│       ├── orphan_prompt.rs          (V7)
│       ├── alien_file.rs             (V8)
│       ├── pub_leak.rs               (V9)
│       ├── quarantine_leak.rs        (V10)
│       ├── dangling_contract.rs      (V11)
│       └── wiring_logic_leak.rs      (V12)
│
├── 02_shell/                         # CLI e formatadores
│   ├── cli.rs
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/                         # tree-sitter, walkdir, sha2, rayon
│   ├── rs_parser.rs                  # @prompt → parsers/rust.md
│   ├── ts_parser.rs                  # @prompt → parsers/typescript.md
│   ├── py_parser.rs                  # @prompt → parsers/python.md
│   ├── walker.rs
│   ├── prompt_walker.rs
│   ├── prompt_reader.rs
│   ├── prompt_snapshot_reader.rs
│   ├── hash_writer.rs
│   ├── snapshot_writer.rs
│   └── config.rs                     # ts_aliases, py_aliases
│
├── 04_wiring/                        # main() — composição e injeção
│   └── main.rs
│
├── lib.rs
├── Cargo.toml
└── crystalline.toml
```

---

## Dependências estruturais

```
L4 (main) — rayon paraleliza o pipeline; despacha por file.language
  ↓ instancia e injeta
L2 (cli, fix_hashes, update_snapshot) ← L1 (rules, entities, contracts)
L3 (walker, rs_parser, ts_parser, py_parser, prompt_reader,
    prompt_snapshot_reader, prompt_walker, hash_writer,
    snapshot_writer, config)
  ↓ implementa portas de
L1 (contracts: FileProvider, LanguageParser, PromptReader,
               PromptSnapshotReader, PromptProvider)
```

L2 e L3 nunca se importam diretamente — L4 os conecta via
injeção de dependência. `rayon` é restrito a L4.

---

## Licença

MIT — [https://github.com/Dikluwe/tekt-linter](https://github.com/Dikluwe/tekt-linter)
