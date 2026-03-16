# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004, ADR-0005, ADR-0006)
**Repositório**: https://github.com/Dikluwe/tekt-linter

---

## Contexto

O linter é a ferramenta de enforcement da Arquitetura Cristalina.
Implementado em Rust. Analisa projetos Cristalinos e reporta
violações em SARIF, compatível com GitHub Code Scanning, VSCode
e agentes de IA.

O linter valida seu próprio código — suas próprias regras se
aplicam a si mesmo.

**ADR-0004:** Pipeline paralelo via rayon, zero-copy com
lifetimes, Fail-Fast com V0 Fatal, FQN resolvido em L3.

**ADR-0005:** `Location.path` usa `Cow<'a, Path>` —
elimina `Box::leak()`. `Cargo.toml` nucleado por `cargo.md`.

**ADR-0006:** Fechamento topológico completo via Map-Reduce.
V7 (prompts órfãos), V8 (arquivos alienígenas), V9 (pub leak).
`ProjectIndex` construído por fold+reduce sobre o pipeline
paralelo — sem locks, sem race conditions.

---

## Decisões Arquiteturais

- **Parser**: tree-sitter + tree-sitter-rust (crates oficiais)
- **Paralelismo**: rayon em L4 — Map-Reduce sobre o iterator
  do walker. Fase Map produz `(Vec<Violation>, LocalIndex)`.
  Fase Reduce funde em `(Vec<Violation>, ProjectIndex)`.
- **Representação intermediária**: `ParsedFile<'a>` com
  lifetimes. `Token.symbol: Cow<'a, str>`,
  `Location.path: Cow<'a, Path>`.
- **Saída**: SARIF 2.1.0 primário, `--format text` para terminal.
- **Distribuição**: `cargo install` + binário via GitHub Releases.
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

- **L1**: recebe `ParsedFile<'a>` ou `ProjectIndex<'a>`,
  retorna `Vec<Violation<'a>>`. Zero I/O, zero tree-sitter.
  Funções puras. Testável sem arquivos reais.
- **L2**: CLI via clap. Parseia flags, formata SARIF ou text,
  controla exit code. Não conhece L3.
- **L3**: implementa `FileProvider`, `LanguageParser`,
  `PromptReader`, `PromptSnapshotReader`, `PromptProvider`.
  Propaga `SourceError`. Resolve FQN e `target_subdir`.
- **L4**: instancia e injeta todos os componentes. Orquestra
  pipeline Map-Reduce via rayon. Zero lógica de negócio.

---

## Verificações

**V0 — Unreadable Source** *(ADR-0004)*
Arquivo ilegível. Fatal — bloqueia CI incondicionalmente.

**V1 — Missing Prompt Header**
`prompt_header == None` ou `prompt_file_exists == false`.
Error (bloqueante).

**V2 — Missing Test File**
`has_test_coverage == false` em L1. Isenções via AST.
Error (bloqueante).

**V3 — Forbidden Import**
`file.layer` vs `import.target_layer` via matriz de permissões.
`Layer::Unknown` não gera violação.
Error (bloqueante).

**V4 — Impure Core**
`Token.symbol` proibido em L1. FQN resolvido em L3 — aliases
não burla a regra.
Error (bloqueante).

**V5 — Prompt Drift**
`prompt_hash != current_hash`.
Warning (configurável).

**V6 — Prompt Stale**
`public_interface != prompt_snapshot`. PartialEq completo —
mudança de assinatura detectada.
Warning (configurável).

**V7 — Orphan Prompt** *(ADR-0006)*
Prompt em `00_nucleo/prompts/` sem nenhum arquivo em L1–L4
referenciando-o via `@prompt`. Opera sobre `ProjectIndex`
após fase Reduce. Exceções via `[orphan_exceptions]`.
Warning por padrão (configurável para Error).

**V8 — Alien File** *(ADR-0006)*
Arquivo de código com `Layer::Unknown` fora de diretórios
excluídos. Opera sobre `ProjectIndex` após fase Reduce.
Fatal — bloqueia CI incondicionalmente.

**V9 — Pub Leak** *(ADR-0006)*
Import de L2 ou L3 apontando para subdiretório interno de L1
não listado em `[l1_ports]`. Opera sobre `ParsedFile` na
fase Map. Error (bloqueante).

---

## Flags CLI
```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text | json        [padrão: text]
  --fail-on <level>      error | warning            [padrão: error]
  --checks <list>        v0,v1,...,v9               [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml           [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --update-snapshot      atualiza Interface Snapshot nos prompts (V6)
  --dry-run              usado com --fix-hashes ou --update-snapshot
```

**Combinações inválidas — CLI retorna exit 1:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Notas:**
- V0 e V8 Fatal sempre bloqueiam CI — `--checks` pode suprimir
  output mas não o exit code
- V7 Warning por padrão — não quebra projetos existentes

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust = { grammar = "tree-sitter-rust", enabled = true }

# Mapeados — arquivos aqui são analisados e devem ter layer
[layers]
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

# Excluídos — ignorados intencionalmente, não disparam V8
[excluded]
build = "target"
deps  = "node_modules"
vcs   = ".git"
cargo = ".cargo"

# Mapeamento de módulo Rust → camada (resolução de imports)
[module_layers]
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

# Portas públicas de L1 — únicos subdiretórios acessíveis
# de L2 e L3. Imports de outros subdiretórios disparam V9.
[l1_ports]
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"

# Prompts que existem legitimamente sem materialização Rust
[orphan_exceptions]
"prompts/template.md"  = "template — não materializa diretamente"
"prompts/readme.md"    = "gera README.md, não arquivo Rust"
"prompts/cargo.md"     = "gera Cargo.toml, não arquivo Rust"

[rules]
V0 = { level = "fatal" }
V1 = { level = "error" }
V2 = { level = "error" }
V3 = { level = "error" }
V4 = { level = "error" }
V5 = { level = "warning" }
V6 = { level = "warning" }
V7 = { level = "warning" }
V8 = { level = "fatal" }
V9 = { level = "error" }
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
          { "id": "V0", "name": "UnreadableSource",  "defaultConfiguration": { "level": "error" } },
          { "id": "V1", "name": "MissingPromptHeader","defaultConfiguration": { "level": "error" } },
          { "id": "V2", "name": "MissingTestFile",    "defaultConfiguration": { "level": "error" } },
          { "id": "V3", "name": "ForbiddenImport",    "defaultConfiguration": { "level": "error" } },
          { "id": "V4", "name": "ImpureCore",         "defaultConfiguration": { "level": "error" } },
          { "id": "V5", "name": "PromptDrift",        "defaultConfiguration": { "level": "warning" } },
          { "id": "V6", "name": "PromptStale",        "defaultConfiguration": { "level": "warning" } },
          { "id": "V7", "name": "OrphanPrompt",       "defaultConfiguration": { "level": "warning" } },
          { "id": "V8", "name": "AlienFile",          "defaultConfiguration": { "level": "error" } },
          { "id": "V9", "name": "PubLeak",            "defaultConfiguration": { "level": "error" } }
        ]
      }
    }
  }]
}
```

---

## Pipeline de execução (L4) — Map-Reduce
```rust
// Fase 0: AllPrompts — sequencial, antes do paralelo
let all_prompts = prompt_walker
    .scan_nucleo(&nucleo_root, &config.orphan_exceptions);

// Fase Map+Reduce paralela
let (mut all_violations, project_index) = walker
    .files()
    .par_bridge()
    .map(|result| -> (Vec<Violation>, LocalIndex) {
        match result {
            Ok(source) => match parser.parse(&source) {
                Ok(parsed) => {
                    let violations = run_checks(&parsed, &enabled, &l1_ports);
                    let local = LocalIndex::from_parsed(&parsed);
                    (violations, local)
                }
                Err(err) => (
                    vec![parse_error_to_violation(err)],
                    LocalIndex::empty(),
                ),
            },
            Err(err) => {
                let local = LocalIndex::from_alien(err.path());
                (vec![source_error_to_violation(err)], local)
            }
        }
    })
    .fold(
        || (Vec::new(), ProjectIndex::new()),
        |(mut viols, mut idx), (v, local)| {
            viols.extend(v);
            idx.merge_local(local);
            (viols, idx)
        },
    )
    .reduce(
        || (Vec::new(), ProjectIndex::new()),
        |(mut viols_a, idx_a), (viols_b, idx_b)| {
            viols_a.extend(viols_b);
            (viols_a, idx_a.merge(idx_b))
        },
    );

// Fase global — V7 e V8 sobre índice completo
if enabled.v7 {
    all_violations.extend(check_orphans(&project_index, &all_prompts));
}
if enabled.v8 {
    all_violations.extend(check_aliens(&project_index));
}
```

**Garantias de segurança:**
- Cada thread opera sobre `LocalIndex` próprio — sem estado
  compartilhado
- `fold` acumula por thread, `reduce` funde threads — ambos
  funcionais puros
- `ProjectIndex::merge` é associativa e comutativa — ordem de
  fusão não afeta resultado
- `AllPrompts` é imutável durante todo o pipeline paralelo

---

## Conversores de erro no wiring (L4)
```rust
fn source_error_to_violation(err: SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location {
                // ADR-0005: Cow::Owned elimina Box::leak()
                // path vem de SourceError — não existe no buffer
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
    }
}

fn parse_error_to_violation(err: ParseError) -> Violation<'static> {
    match err {
        ParseError::SyntaxError { path, line, column, message } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Erro de sintaxe: {message}"),
            location: Location {
                path: Cow::Owned(path),
                line,
                column,
            },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Linguagem não suportada: {language:?}"),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Arquivo vazio ignorado".to_string(),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
    }
}
```

---

## Lógica de exit code (L4)
```
V0 Fatal presente  → exit 1 incondicionalmente
V8 Fatal presente  → exit 1 incondicionalmente
--fail-on error    → exit 1 se qualquer Error presente
--fail-on warning  → exit 1 se qualquer Warning presente
Nenhuma condição   → exit 0
```

V0 e V8 verificados antes de `--fail-on` — não configuráveis.

---

## Novo componente L3: PromptWalker

Varre `00_nucleo/prompts/` sequencialmente antes do pipeline
paralelo e constrói `AllPrompts`. Implementa trait
`PromptProvider` declarada em L1. Exclui entradas de
`[orphan_exceptions]` antes de retornar.

Precisa de prompt próprio: `prompt-walker.md` (novo — L3).

---

## Estrutura de arquivos — derivada dos prompts
```
crystalline-lint/
├── 00_nucleo/
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── project-index.md          ← novo (ADR-0006)
│   │   ├── cargo.md
│   │   ├── contracts/
│   │   │   ├── file-provider.md
│   │   │   ├── language-parser.md
│   │   │   ├── parse-error.md
│   │   │   ├── prompt-reader.md
│   │   │   ├── prompt-snapshot-reader.md
│   │   │   └── prompt-provider.md    ← novo (ADR-0006)
│   │   ├── rules/
│   │   │   ├── prompt-header.md      (V1)
│   │   │   ├── test-file.md          (V2)
│   │   │   ├── forbidden-import.md   (V3)
│   │   │   ├── impure-core.md        (V4)
│   │   │   ├── prompt-drift.md       (V5)
│   │   │   ├── prompt-stale.md       (V6)
│   │   │   ├── orphan-prompt.md      (V7) ← novo
│   │   │   ├── alien-file.md         (V8) ← novo
│   │   │   └── pub-leak.md           (V9) ← novo
│   │   ├── rs-parser.md              ← revisado (target_subdir)
│   │   ├── file-walker.md            ← revisado (excluded vs unknown)
│   │   ├── prompt-walker.md          ← novo (ADR-0006)
│   │   ├── sarif-formatter.md        ← revisado (V7–V9)
│   │   └── fix-hashes.md
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       ├── 0002-code-to-prompt-feedback-direction.md
│       ├── 0004-motor-reformulation.md
│       ├── 0005-location-owned-paths-cargo-nucleation.md
│       └── 0006-topological-closure.md
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test     ← revisado (Import.target_subdir)
│   │   ├── project_index.rs + test   ← novo (ADR-0006)
│   │   ├── violation.rs + test
│   │   └── layer.rs + test
│   ├── contracts/
│   │   ├── file_provider.rs
│   │   ├── language_parser.rs
│   │   ├── parse_error.rs + test
│   │   ├── prompt_reader.rs
│   │   ├── prompt_snapshot_reader.rs
│   │   └── prompt_provider.rs        ← novo (ADR-0006)
│   └── rules/
│       ├── prompt_header.rs + test   (V1)
│       ├── test_file.rs + test       (V2)
│       ├── forbidden_import.rs + test (V3)
│       ├── impure_core.rs + test     (V4)
│       ├── prompt_drift.rs + test    (V5)
│       ├── prompt_stale.rs + test    (V6)
│       ├── orphan_prompt.rs + test   (V7) ← novo
│       ├── alien_file.rs + test      (V8) ← novo
│       └── pub_leak.rs + test        (V9) ← novo
│
├── 02_shell/
│   ├── cli.rs                        ← revisado (V7–V9)
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/
│   ├── walker.rs + test              ← revisado (excluded vs unknown)
│   ├── rs_parser.rs + test           ← revisado (target_subdir)
│   ├── prompt_walker.rs + test       ← novo (ADR-0006)
│   ├── prompt_reader.rs + test
│   ├── prompt_snapshot_reader.rs + test
│   ├── hash_writer.rs + test
│   ├── snapshot_writer.rs + test
│   └── config.rs + test              ← revisado (excluded, l1_ports,
│                                                  orphan_exceptions)
│
├── 04_wiring/
│   └── main.rs                       ← revisado (Map-Reduce, V7–V9)
│
├── lib.rs                            ← revisado (novos módulos)
├── Cargo.toml
└── crystalline.toml                  ← revisado (novas seções)
```

---

## Critérios de Verificação (sistema completo)
```
Dado projeto sem nenhuma violação
Quando crystalline-lint rodar
Então exit 0 e output vazio (--quiet)

Dado projeto com arquivo L1 sem @prompt header
Quando crystalline-lint rodar
Então exit 1 + SARIF com V1 apontando path e linha

Dado alias use std::fs as f; e chamada f::read() em L1
Quando crystalline-lint rodar
Então exit 1 + V4 com symbol "std::fs::read"
— alias resolvido, regra não burlável

Dado arquivo ilegível por permissão
Quando crystalline-lint rodar
Então exit 1 + V0 Fatal
E demais arquivos continuam sendo analisados

Dado prompt em 00_nucleo/prompts/ sem @prompt em nenhum .rs
Quando crystalline-lint rodar
Então V7 warning com path do prompt órfão

Dado arquivo src/utils/helper.rs fora de [layers] e [excluded]
Quando crystalline-lint rodar
Então exit 1 + V8 Fatal com path do arquivo alien

Dado import crate::core::internal::helper em L2
E "internal" não em [l1_ports]
Quando crystalline-lint rodar
Então exit 1 + V9 Error com linha do import

Dado --fail-on warning e apenas V5 presente
Quando crystalline-lint rodar
Então exit 1

Dado --fail-on error (padrão) e apenas V5 presente
Quando crystalline-lint rodar
Então exit 0

Dado V0 ou V8 Fatal presente com qualquer --fail-on
Quando crystalline-lint rodar
Então exit 1 — Fatal incondicionalmente

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido com V0–V9 na tabela de regras

Dado pipeline com 500 arquivos
Quando crystalline-lint rodar
Então ProjectIndex idêntico independente da ordem de fusão
— Map-Reduce comutativo e associativo

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
| 2026-03-14 | ADR-0004: rayon, zero-copy, V0 Fatal, FQN, conversores com Cow::Owned | linter-core.md, main.rs |
| 2026-03-14 | ADR-0005: Cow<'a,Path> nos conversores elimina Box::leak() | linter-core.md, main.rs |
| 2026-03-14 | ADR-0006: Map-Reduce, V7–V9, ProjectIndex, PromptWalker, [excluded], [l1_ports], [orphan_exceptions], SARIF atualizado, critérios V7–V9 adicionados | linter-core.md, main.rs, crystalline.toml |
