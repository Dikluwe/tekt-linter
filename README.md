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
| V0 | `UnreadableSource` | **fatal** | Arquivo descoberto mas ilegível (permissão negada, disco corrompido). Bloqueia CI incondicionalmente — não configurável |
| V1 | `MissingPromptHeader` | **error** | Arquivo em L1–L4 sem cabeçalho `//! @prompt` ou com prompt referenciado inexistente em `00_nucleo/` |
| V2 | `MissingTestFile` | **error** | Arquivo em L1 sem `#[cfg(test)]` interno nem `_test.rs` adjacente. Arquivos apenas com `pub trait`/`pub struct`/`pub enum` são isentos |
| V3 | `ForbiddenImport` | **error** | Import que viola a direção do fluxo de dependência (ex: L2 importando L3) |
| V4 | `ImpureCore` | **error** | Símbolo de I/O detectado em L1 via AST com FQN resolvido — aliases não burlam a regra |
| V5 | `PromptDrift` | **warning** | Hash declarado em `@prompt-hash` diverge do hash real do arquivo de prompt em `00_nucleo/` |
| V6 | `PromptStale` | **warning** | Interface pública do código mudou desde o snapshot registrado no prompt de origem |

**Sobre V0:** A ausência de violações garante que todos os arquivos
foram lidos e analisados com sucesso — não apenas que o linter não
encontrou problemas nos arquivos que conseguiu abrir.

**Sobre V4:** O linter resolve aliases de importação antes de
verificar símbolos proibidos. `use std::fs as f; f::read(...)` é
detectado como `std::fs::read` — a regra não pode ser burlada
com renomeação.

---

## Flags CLI
```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text | json        [padrão: text]
  --fail-on <level>      error | warning            [padrão: error]
  --checks <list>        v0,v1,v2,v3,v4,v5,v6      [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml           [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --update-snapshot      atualiza Interface Snapshot nos prompts (V6)
  --dry-run              usado com --fix-hashes ou --update-snapshot
  -h, --help             exibe ajuda
  -V, --version          exibe versão
```

**Combinações inválidas:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Nota sobre V0:** `--checks` pode omitir `v0` para suprimir o
output, mas V0 Fatal sempre bloqueia CI independentemente de
`--fail-on`.

---

## crystalline.toml
```toml
[project]
root = "."

# Linguagens habilitadas e suas grammars tree-sitter
[languages]
rust = { grammar = "tree-sitter-rust", enabled = true }
# typescript = { grammar = "tree-sitter-typescript", enabled = false }

# Mapeamento de diretório → camada
[layers]
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

# Mapeamento de módulo Rust → camada (para resolução de imports crate::)
[module_layers]
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

# Severidade configurável por regra
# V0 Fatal não é configurável — sempre bloqueia CI
[rules]
V0 = { level = "fatal" }
V1 = { level = "error" }
V2 = { level = "error" }
V3 = { level = "error" }
V4 = { level = "error" }
V5 = { level = "warning" }
V6 = { level = "warning" }
```

---

## Header canônico

Todo arquivo em L1–L4 deve conter o seguinte cabeçalho no topo:
```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
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

Após modificar a interface pública de um arquivo — adicionar,
remover ou alterar assinatura de função ou tipo — V6 dispara
porque o snapshot registrado no prompt de origem ficou
desatualizado:
```bash
# 1. Ver quais prompts seriam atualizados e qual o delta
crystalline-lint --update-snapshot --dry-run .

# 2. Atualizar os snapshots nos prompts
crystalline-lint --update-snapshot .

# 3. Verificar que zero V6 restam
crystalline-lint .
```

V6 detecta mudanças de assinatura além de adições e remoções —
`foo(a: String)` → `foo(a: Vec<String>)` é uma quebra de contrato
e dispara V6 mesmo com o nome da função inalterado.

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
como anotações diretamente no diff do PR. V0 Fatal aparece como
erro de nível máximo.

---

## Auto-validação
```bash
# O linter deve passar em sua própria validação sem nenhuma violação
crystalline-lint .
# ✓ No violations found
```

Este é o critério de verificação mais importante — se o linter
não consegue validar seu próprio código, há um problema estrutural
no projeto.

---

## Estrutura do projeto

O linter é ele mesmo um projeto Cristalino.
```
crystalline-lint/
├── 00_nucleo/               # Prompts e ADRs (A Semente)
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── cargo.md
│   │   ├── contracts/
│   │   │   ├── file-provider.md
│   │   │   ├── language-parser.md
│   │   │   ├── parse-error.md
│   │   │   ├── prompt-reader.md
│   │   │   └── prompt-snapshot-reader.md
│   │   ├── rules/
│   │   │   ├── prompt-header.md    (V1)
│   │   │   ├── test-file.md        (V2)
│   │   │   ├── forbidden-import.md (V3)
│   │   │   ├── impure-core.md      (V4)
│   │   │   ├── prompt-drift.md     (V5)
│   │   │   └── prompt-stale.md     (V6)
│   │   ├── rs-parser.md
│   │   ├── file-walker.md
│   │   ├── sarif-formatter.md
│   │   └── fix-hashes.md
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       ├── 0002-code-to-prompt-feedback-direction.md
│       ├── 0004-motor-reformulation.md
│       └── 0005-location-owned-paths-cargo-nucleation.md
│
├── 01_core/                 # Lógica pura — zero I/O
│   ├── entities/            # ParsedFile<'a>, Violation<'a>, Layer
│   ├── contracts/           # Traits para L3 implementar
│   └── rules/               # V1–V6
│
├── 02_shell/                # CLI, formatadores SARIF e text
│   ├── cli.rs
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/                # tree-sitter, walkdir, sha2, rayon
│   ├── rs_parser.rs         # Motor de Duas Fases (FQN + aliases)
│   ├── walker.rs            # Fail-fast: propaga SourceError
│   ├── prompt_reader.rs
│   ├── prompt_snapshot_reader.rs
│   ├── hash_writer.rs
│   ├── snapshot_writer.rs
│   └── config.rs
│
├── 04_wiring/               # main() — composição paralela via rayon
│   └── main.rs
│
├── lib.rs
├── Cargo.toml               # Gerido por cargo.md
└── crystalline.toml
```

---

## Dependências estruturais
```
L4 (main) — rayon paralleliza o pipeline
  ↓ instancia
L2 (cli, fix_hashes, update_snapshot) ← L1 (rules, entities, contracts)
L3 (walker, rs_parser, prompt_reader, prompt_snapshot_reader,
    hash_writer, snapshot_writer)
  ↓ implementa traits de
L1 (contracts: FileProvider, LanguageParser, PromptReader,
               PromptSnapshotReader)
```

L2 e L3 nunca se importam diretamente — L4 os conecta via
injeção de dependência. `rayon` é restrito a L4.

---

## Licença

MIT — [https://github.com/Dikluwe/tekt-linter](https://github.com/Dikluwe/tekt-linter)
