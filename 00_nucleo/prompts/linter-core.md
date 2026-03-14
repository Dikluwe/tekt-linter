# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

---

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
- **Repositório**: https://github.com/Dikluwe/tekt-linter
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
  `LanguageParser`, `PromptReader`, `PromptSnapshotReader`
  declarados em L1.
- **L4**: instancia walker + parser + readers + checker + formatter,
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

**V6 — Prompt stale (code-to-prompt feedback)**
`ParsedFile.public_interface != ParsedFile.prompt_snapshot`.
Detecta quando a interface pública do código mudou desde o
último snapshot registrado no prompt de origem.
Diff semântico via AST — mudanças cosméticas não disparam.
Warning — não bloqueia CI por padrão.

---

## Flags CLI
```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text | json        [padrão: text]
  --fail-on <level>      error | warning            [padrão: error]
  --checks <list>        v1,v2,v3,v4,v5,v6         [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml           [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (reescreve arquivos)
  --update-snapshot      atualiza Interface Snapshot nos prompts com V6
  --dry-run              usado com --fix-hashes ou --update-snapshot:
                         mostra mudanças sem reescrever
```

Combinações inválidas — CLI retorna exit 1:
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust = { grammar = "tree-sitter-rust", enabled = true }
# typescript = { grammar = "tree-sitter-typescript", enabled = false }

[layers]
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

[module_layers]
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

[rules]
V1 = { level = "error" }
V2 = { level = "error" }
V3 = { level = "error" }
V4 = { level = "error" }
V5 = { level = "warning" }
V6 = { level = "warning" }
```

---

## Formato SARIF de saída
```json
{
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {
        "name": "crystalline-lint",
        "version": "0.1.0",
        "rules": [
          { "id": "V1", "name": "MissingPromptHeader",  "defaultConfiguration": { "level": "error" } },
          { "id": "V2", "name": "MissingTestFile",      "defaultConfiguration": { "level": "error" } },
          { "id": "V3", "name": "ForbiddenImport",      "defaultConfiguration": { "level": "error" } },
          { "id": "V4", "name": "ImpureCore",           "defaultConfiguration": { "level": "error" } },
          { "id": "V5", "name": "PromptDrift",          "defaultConfiguration": { "level": "warning" } },
          { "id": "V6", "name": "PromptStale",          "defaultConfiguration": { "level": "warning" } }
        ]
      }
    }
  }]
}
```

---

## Pipeline de execução (L4)
```
FileWalker::files()
    → Iterator<SourceFile>
    → RustParser::parse(source_file)
        (injetado com FsPromptReader + FsPromptSnapshotReader)
    → Result<ParsedFile, ParseError>
    → [V1, V2, V3, V4, V5, V6]::check(&parsed_file)
    → Vec<Violation>
    → SarifFormatter::format(violations)
    → stdout + exit_code

ParseError → violação sintética PARSE (não silenciado, não panic)
```

---

## Wiring (L4) — instanciação completa
```rust
// 04_wiring/main.rs

let config = CrystallineConfig::load_or_default(&cli.config);
let nucleo_root = PathBuf::from(".");

let parser = RustParser::new(
    FsPromptReader    { nucleo_root: nucleo_root.clone() },
    FsPromptSnapshotReader { nucleo_root: nucleo_root.clone() },
    config.clone(),
);

let walker  = FileWalker::new(cli.path.clone(), config.clone());
let rewriter = L3HashRewriter { nucleo_root: nucleo_root.clone() };
let snapshot_writer = L3SnapshotWriter { nucleo_root: nucleo_root.clone() };
```

`L3SnapshotWriter` é o adapter de L4 que implementa o contrato
de L2 para `--update-snapshot`, análogo ao `L3HashRewriter`
existente para `--fix-hashes`.

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
│   │   │   ├── prompt-reader.md
│   │   │   └── prompt-snapshot-reader.md    ← novo (V6)
│   │   ├── rules/
│   │   │   ├── prompt-header.md
│   │   │   ├── test-file.md
│   │   │   ├── forbidden-import.md
│   │   │   ├── impure-core.md
│   │   │   ├── prompt-drift.md
│   │   │   └── prompt-stale.md              ← novo (V6)
│   │   ├── rs-parser.md                     ← revisado (V6)
│   │   ├── file-walker.md
│   │   ├── sarif-formatter.md               ← revisado (V6)
│   │   └── fix-hashes.md                   ← revisado (--update-snapshot)
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       └── 0002-code-to-prompt-feedback-direction.md ← novo (V6)
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test    ← revisado (V6: PublicInterface, InterfaceDelta)
│   │   ├── violation.rs + test
│   │   └── layer.rs + test
│   ├── contracts/
│   │   ├── file_provider.rs
│   │   ├── language_parser.rs
│   │   ├── parse_error.rs + test
│   │   ├── prompt_reader.rs
│   │   └── prompt_snapshot_reader.rs        ← novo (V6)
│   └── rules/
│       ├── prompt_header.rs + test
│       ├── test_file.rs + test
│       ├── forbidden_import.rs + test
│       ├── impure_core.rs + test
│       ├── prompt_drift.rs + test
│       └── prompt_stale.rs + test           ← novo (V6)
│
├── 02_shell/
│   ├── cli.rs                               ← revisado (V6, --update-snapshot)
│   ├── fix_hashes.rs
│   └── update_snapshot.rs                  ← novo (V6)
│
├── 03_infra/
│   ├── walker.rs + test
│   ├── rs_parser.rs + test                 ← revisado (V6: PublicInterface)
│   ├── prompt_reader.rs + test
│   ├── prompt_snapshot_reader.rs + test    ← novo (V6)
│   ├── hash_writer.rs + test
│   ├── snapshot_writer.rs + test           ← novo (V6)
│   └── config.rs + test
│
├── 04_wiring/
│   └── main.rs                             ← revisado (V6: novos adapters)
│
├── lib.rs                                  ← revisado (novos módulos)
├── Cargo.toml
└── crystalline.toml                        ← revisado (V6 nas rules)
```

---

## Critérios de Verificação (sistema completo)
```
Dado projeto Rust sem nenhuma violação cristalina
Quando crystalline-lint rodar
Então exit 0 e output vazio (--quiet)

Dado projeto com arquivo L1 sem @prompt header
Quando crystalline-lint rodar
Então exit 1 + SARIF com V1 apontando path e linha

Dado arquivo com interface pública alterada desde o snapshot
Quando crystalline-lint rodar
Então exit 0 + SARIF com V6 como warning descrevendo o delta

Dado --fail-on warning com V6 presente
Quando crystalline-lint rodar
Então exit 1

Dado --update-snapshot --dry-run
Quando crystalline-lint rodar
Então nenhum arquivo modificado + output mostra snapshots que seriam atualizados

Dado --update-snapshot sem --dry-run
Quando crystalline-lint rodar
Então seção Interface Snapshot atualizada nos prompts com V6
E re-análise retorna zero V6

Dado --dry-run sem --fix-hashes ou --update-snapshot
Quando crystalline-lint rodar
Então exit 1 com mensagem de erro de uso

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido e parseável

Dado o próprio projeto crystalline-lint
Quando crystalline-lint rodar sobre si mesmo
Então exit 0 — o linter passa em sua própria validação
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | — |
| 2025-03-13 | Gap 5: estrutura derivada dos prompts, pipeline explícito, contratos adicionados, ParseError no wiring | linter-core.md |
| 2025-03-13 | V6: PromptStale, PublicInterface, PromptSnapshotReader, --update-snapshot, L3SnapshotWriter, estrutura revisada | linter-core.md, main.rs, cli.rs, lib.rs, crystalline.toml |
