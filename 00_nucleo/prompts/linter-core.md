# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-19 (ADR-0009: Python, ImportKind semântico, V4 multi-linguagem)
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

**ADR-0007:** Fechamento comportamental. V10 (quarantine leak),
V11 (dangling contract), V12 (wiring logic leak).

**ADR-0008:** Distribuição via binários pré-compilados no GitHub
Releases. `cargo install` para uso local.

**ADR-0009:** Isolamento de parsers por linguagem. `parsers/`
contém `_template.md`, `rust.md`, `typescript.md`, `python.md`.
`ImportKind` é semântico (`Direct/Glob/Alias/Named`) — nunca
sintáctico. V4 multi-linguagem via `forbidden_symbols_for(language)`.
Adicionar qualquer linguagem futura requer apenas criar
`parsers/<lang>.md` e `03_infra/<lang>_parser.rs` — zero toques
em prompts universais.

---

## Decisões Arquiteturais

- **Parsers**: tree-sitter com grammar por linguagem.
  Rust: `tree-sitter-rust` via `RustParser` (`parsers/rust.md`).
  TypeScript: `tree-sitter-typescript` via `TsParser`
  (`parsers/typescript.md`).
  Python: `tree-sitter-python` via `PyParser`
  (`parsers/python.md`).
  Selecção em L4 por `file.language`.
- **Resolução de camadas Rust**: `crate::` absoluto via
  `LayerResolver`. TypeScript, Python e linguagens futuras:
  resolução física via `normalize` + `resolve_file_layer`.
- **`ImportKind`**: semântico — `Direct/Glob/Alias/Named`.
  Nunca contém variantes específicas de linguagem.
- **V4 multi-linguagem**: `forbidden_symbols_for(language)`
  em `impure_core.rs` — listas separadas para Rust, TypeScript,
  Python. `HasTokens` expõe `language()`.
- **Paralelismo**: rayon em L4 — Map-Reduce. Fase Map produz
  `(Vec<Violation>, LocalIndex)`. Fase Reduce funde em
  `(Vec<Violation>, ProjectIndex)`.
- **Representação intermediária**: `ParsedFile<'a>` com
  lifetimes. `Token.symbol: Cow<'a, str>`,
  `Location.path: Cow<'a, Path>`.
- **Saída**: SARIF 2.1.0 primário, `--format text` para terminal.
- **Headers canónicos**:
```rust
//! Crystalline Lineage          ← Rust
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```
```typescript
// Crystalline Lineage           ← TypeScript
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
```
```python
# Crystalline Lineage            ← Python
# @prompt 00_nucleo/prompts/<nome>.md
# @prompt-hash <sha256[0..8]>
# @layer L<n>
# @updated YYYY-MM-DD
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
  Propaga `SourceError`. Resolve FQN, `target_subdir`,
  `declared_traits`, `implemented_traits` e `declarations`.
  Para TypeScript e Python: resolve imports fisicamente via
  `normalize` + `resolve_file_layer`.
- **L4**: instancia e injeta todos os componentes. Selecciona
  o parser correcto por `file.language`. Orquestra pipeline
  Map-Reduce via rayon. Zero lógica de negócio.

---

## Verificações

**V0 — Unreadable Source** *(ADR-0004)*
Fatal — bloqueia CI incondicionalmente.

**V1 — Missing Prompt Header**
`prompt_header == None` ou `prompt_file_exists == false`.
Error (bloqueante).

**V2 — Missing Test File**
`has_test_coverage == false` em L1. Isenções via AST.
Error (bloqueante).

**V3 — Forbidden Import**
`file.layer` vs `import.target_layer` via matriz de permissões.
`Layer::Unknown` não gera violação. `ImportKind` não é usado.
Error (bloqueante).

**V4 — Impure Core**
`Token.symbol` verificado contra `forbidden_symbols_for(file.language())`.
Lista separada por linguagem em `impure_core.rs` — agnóstico de
`ImportKind`. FQN resolvido em L3 para Rust — aliases não burlam.
Error (bloqueante).

**V5 — Prompt Drift**
`prompt_hash != current_hash`. Warning (configurável).

**V6 — Prompt Stale**
`public_interface != prompt_snapshot`. PartialEq completo.
Warning (configurável).

**V7 — Orphan Prompt** *(ADR-0006)*
Prompt sem materialização. Opera sobre `ProjectIndex`.
Warning por padrão (configurável para Error).

**V8 — Alien File** *(ADR-0006)*
`Layer::Unknown` fora de excluídos. Fatal.

**V9 — Pub Leak** *(ADR-0006)*
Import fora das portas de L1. Opera por arquivo.
Error (bloqueante).

**V10 — Quarantine Leak** *(ADR-0007)*
Import de produção para `lab/`. Fatal.

**V11 — Dangling Contract** *(ADR-0007)*
Trait/interface/Protocol em L1/contracts/ sem implementação
em L2/L3. Opera sobre `ProjectIndex`. Error (bloqueante).

**V12 — Wiring Logic Leak** *(ADR-0007)*
Declaração de tipo em L4. `impl Trait for Type`, `class implements`
e `class(Protocol/ABC)` são permitidos. Warning por padrão.

---

## Flags CLI
```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text | json             [padrão: text]
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
```

**Combinações inválidas — CLI retorna exit 1:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust       = { grammar = "tree-sitter-rust",       enabled = true }
typescript = { grammar = "tree-sitter-typescript", enabled = true }
python     = { grammar = "tree-sitter-python",     enabled = true }

[layers]
L0  = "00_nucleo"
L1  = "01_core"
L2  = "02_shell"
L3  = "03_infra"
L4  = "04_wiring"
lab = "lab"

[excluded]
build = "target"
deps  = "node_modules"
vcs   = ".git"
cargo = ".cargo"

[module_layers]
entities  = "L1"
contracts = "L1"
rules     = "L1"
shell     = "L2"
infra     = "L3"

[l1_ports]
entities  = "01_core/entities"
contracts = "01_core/contracts"
rules     = "01_core/rules"

# Prompts sem materialização de código — isentos de V7
# NOTA: parsers/rust.md, parsers/typescript.md e parsers/python.md
# NÃO são excepções — TÊM materialização (rs_parser.rs, ts_parser.rs, py_parser.rs)
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

[wiring_exceptions]
allow_adapter_structs = true

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
          { "id": "V0",  "name": "UnreadableSource",   "defaultConfiguration": { "level": "error" } },
          { "id": "V1",  "name": "MissingPromptHeader","defaultConfiguration": { "level": "error" } },
          { "id": "V2",  "name": "MissingTestFile",    "defaultConfiguration": { "level": "error" } },
          { "id": "V3",  "name": "ForbiddenImport",    "defaultConfiguration": { "level": "error" } },
          { "id": "V4",  "name": "ImpureCore",         "defaultConfiguration": { "level": "error" } },
          { "id": "V5",  "name": "PromptDrift",        "defaultConfiguration": { "level": "warning" } },
          { "id": "V6",  "name": "PromptStale",        "defaultConfiguration": { "level": "warning" } },
          { "id": "V7",  "name": "OrphanPrompt",       "defaultConfiguration": { "level": "warning" } },
          { "id": "V8",  "name": "AlienFile",          "defaultConfiguration": { "level": "error" } },
          { "id": "V9",  "name": "PubLeak",            "defaultConfiguration": { "level": "error" } },
          { "id": "V10", "name": "QuarantineLeak",     "defaultConfiguration": { "level": "error" } },
          { "id": "V11", "name": "DanglingContract",   "defaultConfiguration": { "level": "error" } },
          { "id": "V12", "name": "WiringLogicLeak",    "defaultConfiguration": { "level": "warning" } }
        ]
      }
    }
  }]
}
```

---

## Pipeline de execução (L4) — Map-Reduce
```rust
// Fase 0: AllPrompts — sequencial
let all_prompts = prompt_walker.scan_nucleo(&nucleo_root, &config.orphan_exceptions);

let wiring_config = WiringConfig {
    allow_adapter_structs: config.wiring_exceptions.allow_adapter_structs.unwrap_or(true),
};

// Fase Map+Reduce paralela
let (mut all_violations, project_index) = walker
    .files()
    .par_bridge()
    .map(|result| -> (Vec<Violation>, LocalIndex) {
        match result {
            Ok(source) => {
                // Selecção do parser por file.language (ADR-0009)
                let parse_result = match source.language {
                    Language::Rust       => rust_parser.parse(&source),
                    Language::TypeScript => ts_parser.parse(&source),
                    Language::Python     => py_parser.parse(&source),
                    _                    => Err(ParseError::UnsupportedLanguage {
                        path: source.path.clone(),
                        language: source.language.clone(),
                    }),
                };
                match parse_result {
                    Ok(parsed) => {
                        let violations = run_checks(
                            &parsed, &enabled, &l1_ports, &wiring_config,
                        );
                        let local = LocalIndex::from_parsed(&parsed);
                        (violations, local)
                    }
                    Err(err) => (
                        vec![parse_error_to_violation(err)],
                        LocalIndex::from_parse_error(),
                    ),
                }
            }
            Err(err) => {
                (vec![source_error_to_violation(&err)], LocalIndex::from_source_error())
            }
        }
    })
    .fold(
        || (Vec::new(), ProjectIndex::new()),
        |(mut viols, mut idx), (v, local)| { viols.extend(v); idx.merge_local(local); (viols, idx) },
    )
    .reduce(
        || (Vec::new(), ProjectIndex::new()),
        |(mut viols_a, idx_a), (viols_b, idx_b)| {
            viols_a.extend(viols_b); (viols_a, idx_a.merge(idx_b))
        },
    );

// Fase global — V7, V8, V11
if enabled.v7  { all_violations.extend(check_orphans(&project_index, &all_prompts)); }
if enabled.v8  { all_violations.extend(check_aliens(&project_index)); }
if enabled.v11 { all_violations.extend(check_dangling_contracts(&project_index)); }
```

**Garantias de segurança:**
- Cada thread opera sobre `LocalIndex` próprio — sem estado
  compartilhado
- `fold` acumula por thread, `reduce` funde threads — ambos
  funcionais puros
- `ProjectIndex::merge` é associativa e comutativa — ordem de
  fusão não afeta resultado
- `AllPrompts` é imutável durante todo o pipeline paralelo
- `WiringConfig` é imutável após construção em L4 — partilhado
  por referência nas threads via `par_bridge`

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
V0 Fatal  → exit 1 incondicionalmente
V8 Fatal  → exit 1 incondicionalmente
V10 Fatal → exit 1 incondicionalmente
--fail-on error   → exit 1 se qualquer Error presente
--fail-on warning → exit 1 se qualquer Warning presente
Nenhuma condição  → exit 0
```

---

## Novo componente L3: PromptWalker

Varre `00_nucleo/prompts/` sequencialmente antes do pipeline
paralelo e constrói `AllPrompts`. Implementa trait
`PromptProvider` declarada em L1. Exclui entradas de
`[orphan_exceptions]` antes de retornar.

---

## Estrutura de arquivos
```
crystalline-lint/
├── 00_nucleo/
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── project-index.md
│   │   ├── cargo.md
│   │   ├── readme_prompt.md
│   │   ├── parsers/
│   │   │   ├── _template.md
│   │   │   ├── rust.md
│   │   │   ├── typescript.md
│   │   │   └── python.md              ← novo
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
│   │   │   ├── impure-core.md        (V4) ← revisado (multi-linguagem)
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
│       └── 0009-suporte-typescript.md
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test     ← ImportKind semântico (ADR-0009)
│   │   ├── project_index.rs + test
│   │   ├── rule_traits.rs            ← HasTokens.language() (ADR-0009)
│   │   ├── violation.rs + test
│   │   └── layer.rs + test
│   ├── contracts/
│   │   ├── file_provider.rs
│   │   ├── language_parser.rs
│   │   ├── parse_error.rs
│   │   ├── prompt_reader.rs
│   │   ├── prompt_snapshot_reader.rs
│   │   └── prompt_provider.rs
│   └── rules/
│       ├── prompt_header.rs          (V1)
│       ├── test_file.rs              (V2)
│       ├── forbidden_import.rs       (V3)
│       ├── impure_core.rs            (V4) ← forbidden_symbols_for(language)
│       ├── prompt_drift.rs           (V5)
│       ├── prompt_stale.rs           (V6)
│       ├── orphan_prompt.rs          (V7)
│       ├── alien_file.rs             (V8)
│       ├── pub_leak.rs               (V9)
│       ├── quarantine_leak.rs        (V10)
│       ├── dangling_contract.rs      (V11)
│       └── wiring_logic_leak.rs      (V12)
│
├── 02_shell/
│   ├── cli.rs
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/
│   ├── walker.rs
│   ├── rs_parser.rs                  ← @prompt → parsers/rust.md
│   ├── ts_parser.rs                  ← @prompt → parsers/typescript.md
│   ├── py_parser.rs                  ← novo (ADR-0009)
│   ├── prompt_walker.rs
│   ├── prompt_reader.rs
│   ├── prompt_snapshot_reader.rs
│   ├── hash_writer.rs
│   ├── snapshot_writer.rs
│   └── config.rs                     ← ts_aliases, py_aliases
│
├── 04_wiring/
│   └── main.rs                       ← PyParser instanciado e despachado
│
├── lib.rs
├── Cargo.toml                        ← tree-sitter-typescript, tree-sitter-python
└── crystalline.toml                  ← [py_aliases], python em [languages]
```

---

## Critérios de Verificação (sistema completo)
```
Dado projeto sem nenhuma violação
Quando crystalline-lint rodar
Então exit 0

Dado arquivo .rs L1 sem @prompt header
Quando crystalline-lint rodar
Então exit 1 + V1

Dado alias use std::fs as f; e f::read() em L1 Rust
Quando crystalline-lint rodar
Então exit 1 + V4 com symbol "std::fs::read"

Dado arquivo .ts L1 com import { readFileSync } from 'fs'
Quando crystalline-lint rodar
Então exit 1 + V4 — lista TypeScript via file.language()

Dado arquivo .py L1 com import os
Quando crystalline-lint rodar
Então exit 1 + V4 — lista Python via file.language()

Dado arquivo .py L1 com open("file.txt")
Quando crystalline-lint rodar
Então exit 1 + V4 — builtin proibido

Dado arquivo .ts com import { X } from '../../src/../01_core/entities'
Quando crystalline-lint rodar
Então import resolvido para Layer::L1 — resolução física

Dado arquivo .ts com import { X } from '../../../../../etc/passwd'
Quando crystalline-lint rodar
Então import resolvido para Layer::Unknown — fuga bloqueada

Dado arquivo .py com from .lab.experiment import X
Quando crystalline-lint rodar
Então exit 1 + V10 Fatal — lab detectado por resolução física

Dado arquivo ilegível
Quando crystalline-lint rodar
Então exit 1 + V0 Fatal

Dado arquivo fora de [layers] e [excluded]
Quando crystalline-lint rodar
Então exit 1 + V8 Fatal

Dado trait sem impl em L1/contracts/
Quando crystalline-lint rodar
Então exit 1 + V11 Error

Dado enum em 04_wiring/main.rs
Quando crystalline-lint rodar
Então V12 Warning

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido com V0–V12

Dado o próprio projeto crystalline-lint
Quando crystalline-lint rodar sobre si mesmo
Então exit 0 — o linter passa em sua própria validação
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | — |
| 2025-03-13 | Gap 5: estrutura derivada, pipeline, contratos | linter-core.md |
| 2025-03-13 | V6: PromptStale, PublicInterface, --update-snapshot | linter-core.md |
| 2026-03-14 | ADR-0004: rayon, zero-copy, V0 Fatal, FQN | linter-core.md, main.rs |
| 2026-03-14 | ADR-0005: Cow<'a,Path> elimina Box::leak() | linter-core.md, main.rs |
| 2026-03-14 | ADR-0006: Map-Reduce, V7–V9, ProjectIndex | linter-core.md, main.rs |
| 2026-03-15 | collect_walker_results(), from_parsed(), SourceError.path() | main.rs |
| 2026-03-16 | ADR-0007: V10–V12, WiringConfig, check_dangling_contracts | linter-core.md, main.rs |
| 2026-03-18 | ADR-0009: TsParser, parsers/, ImportKind semântico, V4 multi-linguagem | linter-core.md, main.rs |
| 2026-03-19 | ADR-0009 Python: PyParser no pipeline, [py_aliases], python em [languages], tree-sitter-python, critérios Python adicionados | linter-core.md, main.rs, crystalline.toml, Cargo.toml |
