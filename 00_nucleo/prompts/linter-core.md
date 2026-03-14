# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

O linter é a ferramenta de enforcement da Arquitetura Cristalina.
Sem ele, todas as regras estruturais são sugestões. Com ele, violações
se tornam ruído visível no CI e no editor.

Implementado em Rust. Analisa projetos Cristalinos e reporta violações
em SARIF, compatível com GitHub Code Scanning, VSCode e agentes de IA.

O linter é ele mesmo um projeto Cristalino — suas próprias regras se
aplicam ao seu próprio código. A v1 verifica projetos Rust. Suporte a
outras linguagens (TypeScript, Python) é adicionado via plugins de
grammar declarados em configuração, sem mudança no núcleo.

---

## Decisões Arquiteturais

- **Parser**: tree-sitter + tree-sitter-rust (crates oficiais)
- **Representação intermediária**: L1 opera sobre `ParsedFile` —
  AST agnóstico de linguagem. Grammars em L3 traduzem source → `ParsedFile`.
- **Multi-linguagem**: grammars são plugins declarados em
  `crystalline.toml`. O núcleo não conhece nenhuma linguagem específica.
- **Saída**: SARIF 2.1.0 como formato primário. `--format text` para
  terminal humano.
- **Distribuição**: `cargo install` + binário para CI via GitHub Releases.
- **Header Rust canônico**:
```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```

---

## Restrições Estruturais

- **L1**: recebe `ParsedFile`, retorna `Vec<Violation>`. Zero I/O,
  zero tree-sitter, zero filesystem. Funções puras sobre estruturas
  de dados. Testável sem nenhum arquivo real.
- **L2**: CLI via `clap`. Parseia flags, formata SARIF ou text,
  controla exit code. Não conhece L3.
- **L3**: filesystem walker (`walkdir`), leitor de source, parser
  tree-sitter → `ParsedFile`. Implementa `FileProvider`,
  `LanguageParser` e `PromptReader` declarados em L1.
- **L4**: instancia walker + parser + reader + checker + formatter,
  executa. Zero lógica de negócio.

---

## Verificações da v1

**V1 — Presença de @prompt header**
Ausência de cabeçalho válido provido pela Trait construtura ou
inexistência de arquivo associado.
Erro bloqueante.

**V2 — Test file correspondente**
Ausência de cobertura (`has_test_coverage == false`) em arquivo L1 pela Trait respectiva.
Arquivos sem blocos `impl` com corpo lógico são isentos
— deduzido do AST por L3.
Erro bloqueante.

**V3 — Imports proibidos por camada**
Comparação pura de `file.layer` com `import.target_layer`
para cada `Import` extraído (Trait).
`Layer::Unknown` não gera violação.
Erro bloqueante.

**V4 — I/O em L1**
Presença de `Token` extraído na AST cujo `symbol`
resolve para símbolo proibido em arquivo L1.
Detecção semântica via AST — não regex.
Erro bloqueante.

**V5 — Hash de prompt (drift detection)**
`PromptHeader.prompt_hash != PromptHeader.current_hash`.
`current_hash` populado por L3 via `FsPromptReader`.
Warning — não bloqueia CI por padrão.

---

## Flags CLI
```
crystalline-lint [OPTIONS] [PATH]

OPTIONS:
  --format <fmt>       sarif | text | json    [default: text]
  --fail-on <level>    error | warning        [default: error]
  --checks <list>      v1,v2,v3,v4,v5        [default: all]
  --no-drift           desabilita V5
  --machine-readable   alias para --format sarif
  --quiet              apenas exit code, sem output
  --config <path>      crystalline.toml       [default: ./crystalline.toml]
```

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust = { grammar = "tree-sitter-rust", enabled = true }
# typescript = { grammar = "tree-sitter-typescript", enabled = false }

[layers]
L0 = "00_nucleo"
L1 = "01_core"
L2 = "02_shell"
L3 = "03_infra"
L4 = "04_wiring"
lab = "lab"

[rules]
V1 = { level = "error" }
V2 = { level = "error" }
V3 = { level = "error" }
V4 = { level = "error" }
V5 = { level = "warning" }
```

---

## Estrutura de arquivos — derivada dos prompts
```
crystalline-lint/
├── 00_nucleo/
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── contracts/
│   │   │   ├── file-provider.md
│   │   │   ├── language-parser.md
│   │   │   ├── parse-error.md
│   │   │   └── prompt-reader.md
│   │   ├── rules/
│   │   │   ├── prompt-header.md
│   │   │   ├── test-file.md
│   │   │   ├── forbidden-import.md
│   │   │   ├── impure-core.md
│   │   │   └── prompt-drift.md
│   │   ├── rs-parser.md
│   │   ├── file-walker.md
│   │   └── sarif-formatter.md
│   └── adr/
│       └── 0001-tree-sitter-intermediate-repr.md
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test  ← violation-types.md
│   │   ├── violation.rs + test    ← violation-types.md
│   │   └── layer.rs + test        ← violation-types.md
│   ├── contracts/
│   │   ├── file_provider.rs       ← file-provider.md
│   │   ├── language_parser.rs     ← language-parser.md
│   │   ├── parse_error.rs + test  ← parse-error.md
│   │   └── prompt_reader.rs       ← prompt-reader.md
│   └── rules/
│       ├── prompt_header.rs + test ← prompt-header.md
│       ├── test_file.rs + test     ← test-file.md
│       ├── forbidden_import.rs + test ← forbidden-import.md
│       ├── impure_core.rs + test   ← impure-core.md
│       └── prompt_drift.rs + test  ← prompt-drift.md
│
├── 02_shell/
│   └── cli.rs                     ← sarif-formatter.md
│
├── 03_infra/
│   ├── walker.rs + test           ← file-walker.md
│   ├── rs_parser.rs + test        ← rs-parser.md
│   └── prompt_reader.rs + test    ← prompt-reader.md
│
├── 04_wiring/
│   └── main.rs                    ← linter-core.md
│
├── Cargo.toml
└── crystalline.toml
```

---

## Pipeline de execução (L4)
```
FileWalker::files()
    → Iterator<SourceFile>
    → RustParser::parse(source_file)
    → Result<ParsedFile, ParseError>
    → [V1, V2, V3, V4, V5]::check(&parsed_file)
    → Vec<Violation>
    → SarifFormatter::format(violations)
    → stdout + exit_code
```

Erros de parse (`ParseError`) são convertidos em violações
sintéticas pelo wiring — não silenciados, não propagados como
panic.

---

## Critérios de Verificação (sistema completo)
```
Dado projeto Rust sem nenhuma violação cristalina
Quando crystalline-lint rodar
Então exit 0 e output vazio (--quiet)

Dado projeto com arquivo L1 sem @prompt header
Quando crystalline-lint rodar
Então exit 1 + SARIF com V1 apontando path e linha

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido e parseável

Dado --fail-on warning com violação V5 presente
Quando crystalline-lint rodar
Então exit 1

Dado o próprio projeto crystalline-lint
Quando crystalline-lint rodar sobre si mesmo
Então exit 0 — o linter passa em sua própria validação
```

O último critério é o mais importante — o linter deve ser
capaz de validar seu próprio código sem violações.

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | — |
| 2025-03-13 | Gap 5: estrutura de arquivos derivada dos prompts individuais, pipeline explícito, contratos adicionados, tratamento de ParseError no wiring | linter-core.md |
