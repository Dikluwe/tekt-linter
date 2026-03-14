# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004)
**Repositório**: https://github.com/Dikluwe/tekt-linter

---

## Contexto

O linter é a ferramenta de enforcement da Arquitetura Cristalina.
Implementado em Rust. Analisa projetos Cristalinos e reporta violações
em SARIF, compatível com GitHub Code Scanning, VSCode e agentes de IA.

O linter é ele mesmo um projeto Cristalino — suas próprias regras se
aplicam ao seu próprio código. A v1 verifica projetos Rust. Suporte a
outras linguagens é adicionado via plugins de grammar declarados em
configuração, sem mudança no núcleo.

**Reformulação ADR-0004:** O motor foi reformulado em três vetores:

- **Fail-Fast**: erros de I/O propagados como `V0 Fatal` — ausência
  de violações garante que todos os arquivos foram lidos com sucesso
- **Zero-Copy**: `ParsedFile<'a>` referencia o buffer do `SourceFile`,
  uma alocação por arquivo, zero cópias intermediárias
- **Concorrente**: `rayon::par_bridge()` em L4, threads independentes
  por arquivo, sem estado compartilhado

---

## Decisões Arquiteturais

- **Parser**: tree-sitter + tree-sitter-rust (crates oficiais)
- **Paralelismo**: `rayon` em L4 — `par_bridge()` sobre o iterator
  do walker. `RustParser` é `Send + Sync` porque a tabela de aliases
  é local por arquivo.
- **Representação intermediária**: `ParsedFile<'a>` com lifetimes.
  `Token.symbol` usa `Cow<'a, str>` — `Borrowed` para FQN direto,
  `Owned` para FQN construído por resolução de alias.
- **Saída**: SARIF 2.1.0 como formato primário. `--format text`
  para terminal humano.
- **Distribuição**: `cargo install` + binário para CI via
  GitHub Releases.
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

- **L1**: recebe `ParsedFile<'a>`, retorna `Vec<Violation<'a>>`.
  Zero I/O, zero tree-sitter, zero filesystem. Funções puras sobre
  referências. Testável sem nenhum arquivo real.
- **L2**: CLI via `clap`. Parseia flags, formata SARIF ou text,
  controla exit code. Não conhece L3.
- **L3**: implementa `FileProvider`, `LanguageParser`,
  `PromptReader`, `PromptSnapshotReader`. Propaga `SourceError`,
  resolve FQN via tabela de aliases local.
- **L4**: instancia e injeta todos os componentes, orquestra
  pipeline paralelo via rayon. Zero lógica de negócio.

---

## Verificações

**V0 — Unreadable Source** *(ADR-0004)*
Arquivo descoberto mas ilegível (permissão negada, disco corrompido).
`SourceError::Unreadable` convertido pelo wiring em L4.
Nível: **Fatal** — bloqueia CI incondicionalmente, não configurável.

**V1 — Missing Prompt Header**
`ParsedFile.prompt_header == None` ou
`ParsedFile.prompt_file_exists == false`.
Nível: **Error** (bloqueante).

**V2 — Missing Test File**
`ParsedFile.has_test_coverage == false` em arquivo L1.
Arquivos declaration-only são isentos — deduzido do AST por L3.
Nível: **Error** (bloqueante).

**V3 — Forbidden Import**
Comparação pura de `file.layer` com `import.target_layer`.
`Layer::Unknown` não gera violação.
Nível: **Error** (bloqueante).

**V4 — Impure Core**
`Token.symbol` resolve para símbolo proibido em arquivo L1.
FQN já resolvido por L3 via Motor de Duas Fases — aliases não
burla a regra.
Nível: **Error** (bloqueante).

**V5 — Prompt Drift**
`PromptHeader.prompt_hash != PromptHeader.current_hash`.
`current_hash` calculado por `FsPromptReader` via SHA256.
Nível: **Warning** (não bloqueia por padrão).

**V6 — Prompt Stale**
`ParsedFile.public_interface != ParsedFile.prompt_snapshot`.
Diff semântico via `PartialEq` completo — assinatura, não só nome.
Nível: **Warning** (não bloqueia por padrão).

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
  --fix-hashes           corrige @prompt-hash divergentes
  --update-snapshot      atualiza Interface Snapshot nos prompts com V6
  --dry-run              usado com --fix-hashes ou --update-snapshot
```

**Combinações inválidas — CLI retorna exit 1:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Nota sobre V0:** `--checks` pode omitir `v0` mas V0 Fatal
sempre bloqueia CI independentemente de `--fail-on`. A flag
apenas controla se V0 aparece no output SARIF.

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust = { grammar = "tree-sitter-rust", enabled = true }

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
V0 = { level = "fatal" }
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
          { "id": "V0", "name": "UnreadableSource", "defaultConfiguration": { "level": "error" } },
          { "id": "V1", "name": "MissingPromptHeader", "defaultConfiguration": { "level": "error" } },
          { "id": "V2", "name": "MissingTestFile", "defaultConfiguration": { "level": "error" } },
          { "id": "V3", "name": "ForbiddenImport", "defaultConfiguration": { "level": "error" } },
          { "id": "V4", "name": "ImpureCore", "defaultConfiguration": { "level": "error" } },
          { "id": "V5", "name": "PromptDrift", "defaultConfiguration": { "level": "warning" } },
          { "id": "V6", "name": "PromptStale", "defaultConfiguration": { "level": "warning" } }
        ]
      }
    }
  }]
}
```

---

## Pipeline de execução concorrente (L4)
```rust
// 04_wiring/main.rs — orquestração completa

let config  = CrystallineConfig::load_or_default(&cli.config);
let nucleo  = PathBuf::from(".");

let walker  = FileWalker::new(cli.path.clone(), config.clone());
let parser  = RustParser::new(
    FsPromptReader         { nucleo_root: nucleo.clone() },
    FsPromptSnapshotReader { nucleo_root: nucleo.clone() },
    config.clone(),
);

// Pipeline paralelo — cada arquivo processado em thread independente.
// RustParser é Send + Sync: tabela de aliases é local por arquivo.
// SourceFile vive na thread que o processa — sem compartilhamento.
let all_violations: Vec<Violation> = walker
    .files()
    .par_bridge()                        // rayon: sequential → parallel
    .map(|result| match result {
        Ok(source) => match parser.parse(&source) {
            Ok(parsed)  => run_checks(&parsed, &enabled),
            Err(e)      => vec![parse_error_to_violation(e)],
        },
        Err(e) => vec![source_error_to_violation(e)], // V0 Fatal
    })
    .flatten()
    .collect();                          // sincroniza threads aqui

// Após coletar todas as violações de todas as threads:
// 1. Formatar e imprimir (SARIF ou text)
// 2. Determinar exit code
```

**Por que é seguro:** `SourceFile` é criado e consumido na mesma
thread. `parser` é compartilhado via referência (`&parser`) —
`RustParser` não tem estado mutável, apenas leitores injetados que
também são `Sync`. A tabela de aliases em Fase 1 é uma variável
local da chamada `parse()`, não um campo do parser.

---

## Lógica de exit code (L4)
```
V0 Fatal presente        → exit 1 incondicionalmente
--fail-on error (padrão) → exit 1 se qualquer Error presente
--fail-on warning        → exit 1 se qualquer Warning presente
Nenhuma condição acima   → exit 0
```

`V0` é verificado antes de `--fail-on` — não pode ser suprimido.

---

## Conversores de erro no wiring (L4)
```rust
fn source_error_to_violation(err: SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location { path: path.leak(), line: 0, column: 0 },
            // .leak() para obter &'static Path de um PathBuf owned —
            // aceitável aqui porque V0 é raro e o processo termina logo
        },
    }
}

fn parse_error_to_violation(err: ParseError) -> Violation<'static> {
    match err {
        ParseError::SyntaxError { path, line, column, message } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Erro de sintaxe: {message}"),
            location: Location { path: path.leak(), line, column },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Linguagem não suportada: {language:?}"),
            location: Location { path: path.leak(), line: 0, column: 0 },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Arquivo vazio ignorado".to_string(),
            location: Location { path: path.leak(), line: 0, column: 0 },
        },
    }
}
```

**Nota sobre `.leak()`:** Violações de erro de infraestrutura
têm lifetime `'static` porque seus paths vêm de `PathBuf` owned,
não do buffer do `SourceFile`. `.leak()` é a forma idiomática de
obter `&'static Path` de `PathBuf` em casos onde o processo
termina em seguida. Alternativa sem leak: usar `Arc<Path>` e
ajustar `Location` para suportar owned paths — decisão de v2.

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
│   │   │   └── prompt-snapshot-reader.md
│   │   ├── rules/
│   │   │   ├── prompt-header.md
│   │   │   ├── test-file.md
│   │   │   ├── forbidden-import.md
│   │   │   ├── impure-core.md
│   │   │   ├── prompt-drift.md
│   │   │   └── prompt-stale.md
│   │   ├── rs-parser.md
│   │   ├── file-walker.md
│   │   ├── sarif-formatter.md
│   │   └── fix-hashes.md
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       ├── 0002-code-to-prompt-feedback-direction.md
│       └── 0004-motor-reformulation.md
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test    ← violation-types.md (lifetimes, Cow)
│   │   ├── violation.rs + test
│   │   └── layer.rs + test
│   ├── contracts/
│   │   ├── file_provider.rs         ← file-provider.md (SourceError, Result)
│   │   ├── language_parser.rs       ← language-parser.md (lifetime <'a>)
│   │   ├── parse_error.rs + test
│   │   ├── prompt_reader.rs
│   │   └── prompt_snapshot_reader.rs
│   └── rules/
│       ├── prompt_header.rs + test
│       ├── test_file.rs + test
│       ├── forbidden_import.rs + test
│       ├── impure_core.rs + test
│       ├── prompt_drift.rs + test
│       └── prompt_stale.rs + test
│
├── 02_shell/
│   ├── cli.rs
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/
│   ├── walker.rs + test             ← file-walker.md (SourceError, Result)
│   ├── rs_parser.rs + test          ← rs-parser.md (Duas Fases, Cow, lifetimes)
│   ├── prompt_reader.rs + test
│   ├── prompt_snapshot_reader.rs + test
│   ├── hash_writer.rs + test
│   ├── snapshot_writer.rs + test
│   └── config.rs + test
│
├── 04_wiring/
│   └── main.rs                      ← este prompt (rayon, leak, exit code)
│
├── lib.rs
├── Cargo.toml                       ← rayon adicionado
└── crystalline.toml
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

Dado arquivo com FQN use std::fs as f; e chamada f::read()
Quando crystalline-lint rodar
Então exit 1 + V4 com symbol "std::fs::read"
— alias resolvido, regra não burlável

Dado arquivo ilegível por permissão
Quando crystalline-lint rodar
Então exit 1 + SARIF com V0 Fatal
E demais arquivos continuam sendo analisados

Dado --fail-on warning e apenas V5 presente
Quando crystalline-lint rodar
Então exit 1

Dado --fail-on error (padrão) e apenas V5 presente
Quando crystalline-lint rodar
Então exit 0 — warning não bloqueia por padrão

Dado V0 Fatal presente com --fail-on error
Quando crystalline-lint rodar
Então exit 1 — Fatal bloqueia independentemente de --fail-on

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido e parseável

Dado projeto com 500 arquivos Rust
Quando crystalline-lint rodar
Então processamento usa múltiplas threads via rayon

Dado o próprio projeto crystalline-lint
Quando crystalline-lint rodar sobre si mesmo
Então exit 0 — o linter passa em sua própria validação
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | — |
| 2025-03-13 | Gap 5: estrutura derivada, pipeline explícito, contratos, ParseError no wiring | linter-core.md |
| 2025-03-13 | V6: PromptStale, PublicInterface, --update-snapshot, L3SnapshotWriter | linter-core.md |
| 2026-03-14 | ADR-0004: pipeline paralelo via rayon, V0 Fatal, lifetimes no pipeline, conversores source_error e parse_error, nota sobre .leak(), lógica de exit code explícita | linter-core.md, main.rs, Cargo.toml |
