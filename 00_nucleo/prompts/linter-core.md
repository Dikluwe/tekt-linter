# Prompt: Crystalline Linter (crystalline-lint)

**Camada**: L1 → L4 (sistema completo)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-18 (ADR-0009: TsParser, parsers/, resolução física)
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

**ADR-0007:** Fechamento comportamental. V10 (quarantine leak),
V11 (dangling contract), V12 (wiring logic leak). `ProjectIndex`
estendido com `all_declared_traits` e `all_implemented_traits`
para V11. `ParsedFile` estendido com `declarations` para V12 e
com `declared_traits`/`implemented_traits` transportados para
`LocalIndex`.

**ADR-0009:** Isolamento de parsers por linguagem. `rs-parser.md`
migrado para `parsers/rust.md`. `TsParser` adicionado para
TypeScript com resolução física de imports via `normalize` +
`resolve_file_layer`. Novos prompts: `parsers/_template.md`,
`parsers/rust.md`, `parsers/typescript.md`. Adicionar qualquer
linguagem futura requer apenas criar `parsers/<lang>.md` e
`03_infra/<lang>_parser.rs` — zero toques em prompts universais.

---

## Decisões Arquiteturais

- **Parsers**: tree-sitter com grammar por linguagem.
  Rust: `tree-sitter-rust` via `RustParser` (`parsers/rust.md`).
  TypeScript: `tree-sitter-typescript` via `TsParser`
  (`parsers/typescript.md`). Selecção em L4 por `file.language`.
- **Resolução de camadas Rust**: `crate::` absoluto via
  `LayerResolver`. TypeScript e linguagens futuras: resolução
  física via `normalize` + `resolve_file_layer` (ADR-0009).
- **Paralelismo**: rayon em L4 — Map-Reduce sobre o iterator
  do walker. Fase Map produz `(Vec<Violation>, LocalIndex)`.
  Fase Reduce funde em `(Vec<Violation>, ProjectIndex)`.
- **Representação intermediária**: `ParsedFile<'a>` com
  lifetimes. `Token.symbol: Cow<'a, str>`,
  `Location.path: Cow<'a, Path>`.
- **Saída**: SARIF 2.1.0 primário, `--format text` para terminal.
- **Distribuição**: binário pré-compilado via GitHub Releases
  (ADR-0008). `cargo install` para uso local.
- **Header Rust canônico**:
```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/<nome>.md
//! @prompt-hash <sha256[0..8]>
//! @layer L<n>
//! @updated YYYY-MM-DD
```
- **Header TypeScript canônico**:
```typescript
// Crystalline Lineage
// @prompt 00_nucleo/prompts/<nome>.md
// @prompt-hash <sha256[0..8]>
// @layer L<n>
// @updated YYYY-MM-DD
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
  Para TypeScript: resolve imports fisicamente via `normalize`
  + `resolve_file_layer`.
- **L4**: instancia e injeta todos os componentes. Selecciona
  o parser correcto por `file.language`. Orquestra pipeline
  Map-Reduce via rayon. Zero lógica de negócio.

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
não burla a regra. Lista de símbolos por linguagem em
`parsers/<lang>.md`.
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

**V10 — Quarantine Leak** *(ADR-0007)*
Import em arquivo de L1, L2, L3 ou L4 cujo
`target_layer == Layer::Lab`. Fatal — bloqueia CI
incondicionalmente, não configurável.

**V11 — Dangling Contract** *(ADR-0007)*
Trait/interface pública declarada em L1/contracts/ sem `impl`/
`implements` correspondente em L2 ou L3. Opera sobre
`ProjectIndex` após fase Reduce.
Error (bloqueante).

**V12 — Wiring Logic Leak** *(ADR-0007)*
Declaração de tipo (`struct`/`enum`/`impl`-sem-trait em Rust,
`class`/`interface`/`type` em TypeScript) em arquivo de L4.
`impl Trait for Type` e `class implements` são permitidos.
`struct`/`class` configurável via `[wiring_exceptions]`.
Opera sobre `ParsedFile` na fase Map.
Warning por padrão (configurável para Error).

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
```

**Combinações inválidas — CLI retorna exit 1:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Notas:**
- V0, V8 e V10 Fatal sempre bloqueiam CI — `--checks` pode
  suprimir output mas não o exit code
- V7 Warning por padrão — não quebra projetos existentes
- V12 Warning por padrão — projetos em migração podem ter
  adapter structs/classes legítimas em L4

---

## crystalline.toml
```toml
[project]
root = "."

[languages]
rust       = { grammar = "tree-sitter-rust",       enabled = true }
typescript = { grammar = "tree-sitter-typescript", enabled = true }

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

# Mapeamento de módulo Rust → camada (resolução de imports crate::)
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

# Prompts sem materialização Rust/TS — isentos de V7
# NOTA: parsers/rust.md e parsers/typescript.md NÃO são exceções —
# eles TÊM materialização (rs_parser.rs e ts_parser.rs)
[orphan_exceptions]
"00_nucleo/prompts/cargo.md"          = "gera Cargo.toml, não arquivo de código"
"00_nucleo/prompts/readme_prompt.md"  = "gera README.md, não arquivo de código"
"00_nucleo/prompts/parsers/_template.md" = "contrato editorial, não materializa directamente"

# Aliases TypeScript — opcional, apenas se o projecto usa path aliases
[ts_aliases]
# "@core"  = "01_core"
# "@shell" = "02_shell"
# "@infra" = "03_infra"

# Exceções para V12 — declarações permitidas em L4
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

*Nota SARIF:* V0, V8 e V10 são `Fatal` internamente mas mapeados
para `"error"` no SARIF — o formato 2.1.0 não tem nível `fatal`.

---

## Pipeline de execução (L4) — Map-Reduce

```rust
// Fase 0: AllPrompts — sequencial, antes do paralelo
let all_prompts = prompt_walker
    .scan_nucleo(&nucleo_root, &config.orphan_exceptions);

// WiringConfig a partir do crystalline.toml
let wiring_config = WiringConfig {
    allow_adapter_structs: config.wiring_exceptions
        .allow_adapter_structs
        .unwrap_or(true),
};

// Selecção de parser por linguagem (ADR-0009)
// Cada SourceFile carrega file.language — resolvido pelo walker
// a partir da extensão do ficheiro e de [languages] no toml.
// L4 selecciona o parser correcto antes do par_iter.
// Ambos os parsers implementam LanguageParser — L4 usa trait objects
// ou despacho por enum conforme a implementação concreta.

// Fase Map+Reduce paralela
let (mut all_violations, project_index) = walker
    .files()
    .par_bridge()
    .map(|result| -> (Vec<Violation>, LocalIndex) {
        match result {
            Ok(source) => {
                // Selecção do parser por file.language
                let parse_result = match source.language {
                    Language::Rust       => rust_parser.parse(&source),
                    Language::TypeScript => ts_parser.parse(&source),
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
                let local = LocalIndex::from_source_error();
                (vec![source_error_to_violation(&err)], local)
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

// Fase global — V7, V8, V11 sobre índice completo
if enabled.v7 {
    all_violations.extend(check_orphans(&project_index, &all_prompts));
}
if enabled.v8 {
    all_violations.extend(check_aliens(&project_index));
}
if enabled.v11 {
    all_violations.extend(check_dangling_contracts(&project_index));
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
- `WiringConfig` é imutável após construção — partilhado por
  referência nas threads via `par_bridge`
- `RustParser` e `TsParser` são `Sync` — criam o parser
  tree-sitter localmente em cada chamada a `parse()`

---

## Conversores de erro no wiring (L4)
```rust
fn source_error_to_violation(err: &SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location {
                path: Cow::Owned(path.clone()),
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
            location: Location { path: Cow::Owned(path), line, column },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Linguagem não suportada: {language:?}"),
            location: Location { path: Cow::Owned(path), line: 0, column: 0 },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Arquivo vazio ignorado".to_string(),
            location: Location { path: Cow::Owned(path), line: 0, column: 0 },
        },
    }
}
```

---

## Lógica de exit code (L4)
```
V0 Fatal presente  → exit 1 incondicionalmente
V8 Fatal presente  → exit 1 incondicionalmente
V10 Fatal presente → exit 1 incondicionalmente
--fail-on error    → exit 1 se qualquer Error presente
--fail-on warning  → exit 1 se qualquer Warning presente
Nenhuma condição   → exit 0
```

V0, V8 e V10 verificados antes de `--fail-on` — não configuráveis.

---

## Estrutura de arquivos — derivada dos prompts
```
crystalline-lint/
├── 00_nucleo/
│   ├── prompts/
│   │   ├── linter-core.md
│   │   ├── violation-types.md
│   │   ├── project-index.md
│   │   ├── cargo.md
│   │   ├── readme_prompt.md
│   │   ├── parsers/                         ← ADR-0009
│   │   │   ├── _template.md                 ← contrato editorial
│   │   │   ├── rust.md                      ← migrado de rs-parser.md
│   │   │   └── typescript.md                ← novo
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
│       └── 0009-suporte-typescript.md
│
├── 01_core/
│   ├── entities/
│   │   ├── parsed_file.rs + test     ← revisado (Declaration, TypeKind, DeclarationKind OO)
│   │   ├── project_index.rs + test
│   │   ├── rule_traits.rs            ← movido de contracts/ (ADR-0007)
│   │   ├── violation.rs + test
│   │   └── layer.rs + test
│   ├── contracts/
│   │   ├── file_provider.rs
│   │   ├── language_parser.rs
│   │   ├── parse_error.rs + test
│   │   ├── prompt_reader.rs
│   │   ├── prompt_snapshot_reader.rs
│   │   └── prompt_provider.rs
│   └── rules/
│       ├── prompt_header.rs + test   (V1)
│       ├── test_file.rs + test       (V2)
│       ├── forbidden_import.rs + test (V3)
│       ├── impure_core.rs + test     (V4)
│       ├── prompt_drift.rs + test    (V5)
│       ├── prompt_stale.rs + test    (V6)
│       ├── orphan_prompt.rs + test   (V7)
│       ├── alien_file.rs + test      (V8)
│       ├── pub_leak.rs + test        (V9)
│       ├── quarantine_leak.rs + test (V10)
│       ├── dangling_contract.rs + test (V11)
│       └── wiring_logic_leak.rs + test (V12)
│
├── 02_shell/
│   ├── cli.rs
│   ├── fix_hashes.rs
│   └── update_snapshot.rs
│
├── 03_infra/
│   ├── walker.rs + test
│   ├── rs_parser.rs + test           ← @prompt actualizado para parsers/rust.md
│   ├── ts_parser.rs + test           ← novo (ADR-0009)
│   ├── prompt_walker.rs + test
│   ├── prompt_reader.rs + test
│   ├── prompt_snapshot_reader.rs + test
│   ├── hash_writer.rs + test
│   ├── snapshot_writer.rs + test
│   └── config.rs + test              ← revisado (ts_aliases)
│
├── 04_wiring/
│   └── main.rs                       ← revisado (selecção de parser por linguagem)
│
├── lib.rs
├── Cargo.toml                        ← tree-sitter-typescript adicionado
└── crystalline.toml                  ← [ts_aliases], typescript em [languages],
                                         parsers/_template.md em [orphan_exceptions]
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

Dado alias use std::fs as f; e chamada f::read() em L1 Rust
Quando crystalline-lint rodar
Então exit 1 + V4 com symbol "std::fs::read"
— alias resolvido, regra não burlável

Dado arquivo .ts em L1 com import { readFileSync } from 'fs'
Quando crystalline-lint rodar
Então exit 1 + V4 — módulo fs proibido em L1

Dado arquivo .ts com import { X } from '../../src/../01_core/entities'
(path com ../ que normaliza para 01_core/entities)
Quando crystalline-lint rodar
Então import resolvido para Layer::L1 — resolução física correcta

Dado arquivo .ts com import { X } from '../../../../../etc/passwd'
(path que escapa da raiz do projecto)
Quando crystalline-lint rodar
Então import resolvido para Layer::Unknown — fuga bloqueada

Dado arquivo ilegível por permissão
Quando crystalline-lint rodar
Então exit 1 + V0 Fatal
E demais arquivos continuam sendo analisados

Dado prompt em 00_nucleo/prompts/ sem @prompt em nenhum ficheiro
Quando crystalline-lint rodar
Então V7 warning com path do prompt órfão

Dado parsers/rust.md sem @prompt em nenhum ficheiro
Quando crystalline-lint rodar
Então V7 — parsers/rust.md TEM materialização (rs_parser.rs)
E NÃO está em [orphan_exceptions]

Dado arquivo src/utils/helper.rs fora de [layers] e [excluded]
Quando crystalline-lint rodar
Então exit 1 + V8 Fatal com path do arquivo alien

Dado import crate::core::internal::helper em L2 Rust
E "internal" não em [l1_ports]
Quando crystalline-lint rodar
Então exit 1 + V9 Error com linha do import

Dado import use crate::lab::experiment em arquivo L1
Quando crystalline-lint rodar
Então exit 1 + V10 Fatal
E exit 1 incondicionalmente independente de --fail-on

Dado arquivo .ts em L2 com import { X } from '../lab/experiment'
Quando crystalline-lint rodar
Então exit 1 + V10 Fatal — resolução física detecta lab/

Dado trait FileProvider declarada em 01_core/contracts/
Sem nenhum impl FileProvider for ... em L2 ou L3
Quando crystalline-lint rodar
Então exit 1 + V11 Error mencionando "FileProvider"

Dado enum OutputMode declarado em 04_wiring/main.rs
Quando crystalline-lint rodar
Então V12 Warning mencionando "OutputMode"

Dado struct L3HashRewriter declarada em 04_wiring/main.rs
E allow_adapter_structs = true (padrão)
Quando crystalline-lint rodar
Então nenhum V12 para L3HashRewriter

Dado --fail-on warning e apenas V5 presente
Quando crystalline-lint rodar
Então exit 1

Dado --fail-on error (padrão) e apenas V5 presente
Quando crystalline-lint rodar
Então exit 0

Dado V0, V8 ou V10 Fatal presente com qualquer --fail-on
Quando crystalline-lint rodar
Então exit 1 — Fatal incondicionalmente

Dado --format sarif
Quando crystalline-lint rodar
Então stdout é SARIF 2.1.0 válido com V0–V12 na tabela de regras

Dado pipeline com 500 arquivos Rust e TypeScript misturados
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
| 2026-03-15 | collect_walker_results() helper documentado; from_parsed() detecta aliens; SourceError.path() accessor registrado | main.rs |
| 2026-03-16 | ADR-0007: V10–V12, WiringConfig, [wiring_exceptions], check_dangling_contracts na fase global, exit code actualizado, estrutura actualizada, critérios V10–V12 | linter-core.md, main.rs, crystalline.toml |
| 2026-03-18 | ADR-0009: TsParser adicionado, selecção por file.language em L4, parsers/ na estrutura, rs-parser.md → parsers/rust.md, [ts_aliases] no toml, parsers/_template.md em orphan_exceptions, header TypeScript canónico, critérios de resolução física e TypeScript adicionados | linter-core.md, main.rs, crystalline.toml |
| 2026-03-19 | Passo 1: TypeKind::Class/Interface/TypeAlias, DeclarationKind::Class/Interface/TypeAlias, ImportKind::EsImport, type_kind_str() helper; V12 estendido para Class/Interface/TypeAlias; InterfaceDelta::describe() usa type_kind_str(); OwnedTypeKind no snapshot reader actualizado | parsed_file.rs, wiring_logic_leak.rs, prompt_snapshot_reader.rs |
