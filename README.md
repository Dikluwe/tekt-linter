# crystalline-lint

> Linter arquitetural para projetos que seguem a [Arquitetura Cristalina](https://github.com/Dikluwe/crystalline-architecture-standard).

Sem este linter, as regras estruturais são sugestões. Com ele, violações se tornam ruído visível no CI, no editor e no terminal — antes de virarem dívida técnica.

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

# Corrigir hashes de prompt desatualizados
crystalline-lint --fix-hashes .

# Preview das correções sem reescrever
crystalline-lint --fix-hashes --dry-run .
```

---

## Verificações

| ID | Nome | Severidade | Descrição |
|----|------|------------|-----------|
| V1 | `MissingPromptHeader` | **error** | Arquivo em L1–L4 sem cabeçalho `//! @prompt` ou com prompt referenciado inexistente em `00_nucleo/` |
| V2 | `MissingTestFile` | **error** | Arquivo em L1 sem `#[cfg(test)]` interno nem arquivo `_test.rs` adjacente. Arquivos apenas com `pub trait`/`pub struct`/`pub enum` são isentos |
| V3 | `ForbiddenImport` | **error** | Import que viola a direção do fluxo de dependência (ex: L2 importando L3) |
| V4 | `ImpureCore` | **error** | Símbolo de I/O detectado em L1 via AST (`std::fs`, `reqwest`, `sqlx`, etc.) |
| V5 | `PromptDrift` | **warning** | Hash declarado em `@prompt-hash` diverge do hash real do arquivo de prompt em `00_nucleo/` |

Todos os erros bloqueiam o CI por padrão. Warnings não bloqueiam — configurável via `crystalline.toml` ou `--fail-on`.

---

## Flags CLI

```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>       Formato de saída: sarif | text | json  [padrão: text]
  --fail-on <level>    Nível que dispara exit 1: error | warning  [padrão: error]
  --checks <list>      Verificações a executar: v1,v2,v3,v4,v5  [padrão: all]
  --no-drift           Desabilita V5 (drift detection)
  --machine-readable   Alias para --format sarif
  --quiet              Apenas exit code, sem output
  --config <path>      Caminho para crystalline.toml  [padrão: ./crystalline.toml]
  --fix-hashes         Reescreve @prompt-hash divergentes com o hash real
  --dry-run            Usado com --fix-hashes: mostra mudanças sem reescrever
  -h, --help           Exibe ajuda
  -V, --version        Exibe versão
```

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
[rules]
V1 = { level = "error" }
V2 = { level = "error" }
V3 = { level = "error" }
V4 = { level = "error" }
V5 = { level = "warning" }
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

O campo `@prompt-hash` contém os primeiros 8 caracteres do SHA256 do arquivo de prompt correspondente. Use `--fix-hashes` para manter os hashes atualizados automaticamente após revisões em `00_nucleo/`.

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

O SARIF é enviado ao GitHub Code Scanning — violações aparecem como anotações diretamente no diff do PR.

---

## Workflow com --fix-hashes

Após revisar um prompt em `00_nucleo/`, os arquivos derivados ficam com hash desatualizado e V5 dispara. O ciclo correto:

```bash
# 1. Ver quais arquivos serão corrigidos
crystalline-lint --fix-hashes --dry-run .

# 2. Aplicar correções
crystalline-lint --fix-hashes .

# 3. Verificar que zero V5 restam
crystalline-lint .
```

---

## Estrutura do projeto

O linter é ele mesmo um projeto Cristalino e valida seu próprio código.

```
crystalline-lint/
├── 00_nucleo/               # Prompts e ADRs (A Semente)
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── contracts/       # FileProvider, LanguageParser, ParseError, PromptReader
│   │   ├── rules/           # V1–V5
│   │   ├── rs-parser.md
│   │   ├── file-walker.md
│   │   ├── sarif-formatter.md
│   │   └── fix-hashes.md
│   └── adr/
│       └── 0001-tree-sitter-intermediate-repr.md
│
├── 01_core/                 # Lógica pura — zero I/O
│   ├── entities/            # ParsedFile, Violation, Layer
│   ├── contracts/           # Traits para L3 implementar
│   └── rules/               # V1, V2, V3, V4, V5
│
├── 02_shell/                # CLI, formatadores SARIF e text
│   ├── cli.rs
│   └── fix_hashes.rs
│
├── 03_infra/                # tree-sitter, walkdir, sha2
│   ├── rs_parser.rs
│   ├── walker.rs
│   ├── prompt_reader.rs
│   ├── hash_writer.rs
│   └── config.rs
│
├── 04_wiring/               # main() — composição sem lógica
│   └── main.rs
│
├── lib.rs
├── Cargo.toml
└── crystalline.toml
```

---

## Auto-validação

```bash
# O linter deve passar em sua própria validação sem nenhuma violação
crystalline-lint .
# ✓ No violations found
```

Este é o critério de verificação mais importante — se o linter não consegue validar seu próprio código, há um problema estrutural no projeto.

---

## Dependências estruturais

```
L4 (main)
  ↓ instancia
L2 (cli, fix_hashes) ← L1 (rules, entities, contracts)
L3 (walker, rs_parser, prompt_reader, hash_writer)
  ↓ implementa traits de
L1 (contracts: FileProvider, LanguageParser, PromptReader)
```

L2 e L3 nunca se importam diretamente — L4 os conecta via injeção de dependência.

---

## Licença

MIT — [https://github.com/Dikluwe/tekt-linter](https://github.com/Dikluwe/tekt-linter)
