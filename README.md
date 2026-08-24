# crystalline-lint

> Linter arquitetural para projetos que seguem a [Arquitetura Cristalina](https://github.com/Dikluwe/crystalline-architecture-standard).

Os arquivos chamados “passos” são comandos operacionais temporários herdados do fluxo
do `typst-crystalline`; não são regras nem unidades da Arquitetura Tekt. Regras (`V*`),
decisões (`ADR-*`) e prompts são os artefatos permanentes. Consulte o
[`índice do núcleo`](00_nucleo/README.md).

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

# Comparar dois snapshots semânticos sob um contrato direcional
crystalline-lint refine --before before.json --after after.json --contract refinement.toml
```

---

## Verificações

| ID | Nome | Nível | Descrição |
|----|------|-------|-----------|
| V0 | `UnreadableSource` | **fatal** | Arquivo ilegível. Bloqueia CI incondicionalmente — não configurável |
| V1 | `MissingPromptHeader` | **error** | Arquivo em L1–L4 sem `//! @prompt` ou com prompt referenciado inexistente |
| V2 | `MissingTestFile` | **error** | Arquivo em L1 sem cobertura de teste detectável no AST nem ficheiro de teste adjacente. Arquivos apenas declarativos são isentos (Rust: `#[cfg(test)]` ou `_test.rs`; TypeScript: `.test.ts`/`.spec.ts`; Python: `_test.py`/`test_*.py`; Go: `*_test.go`; Zig: blocos `test` ou `*_test.zig`) |
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
| V13 | `MutableStateInCore` | **error** | Estado global mutável (`static mut`, `Mutex`, `Atomic*`, etc.) declarado em L1 |
| V14 | `ExternalTypeInContract` | **error** | Dependência externa não autorizada em L1 (fora de `[l1_allowed_external]`) |
| V15 | `MultiPromptHeader` | **error** | Arquivo com 2+ linhas `@prompt` no doc-header. A regra de linhagem é um ficheiro, um prompt — com multi-`@prompt` o `--fix-hashes` é indefinido, por isso o lint bloqueia em vez de corrigir ambiguamente |
| V16 | `WildcardSaturation` | **warning** | Braço catch-all descarta informação de enum fechado de domínio sem erro de compilação. Saturação arbitrária (DENY-class) ou default neutro (WARN-class) |
| V17 | `CompoundGuard` | **warning** | Guard de braço de decisão com operadores booleanos compostos (`&&`, `||`) |
| V18 | `RangePatternInMatch` | **warning** | Padrão de range numérico em match de domínio fora de módulo de lexing/numeração |
| V19 | `OrPatternAlternatives` | **info** | Braço de decisão condensa múltiplas alternativas or-pattern (métrica: informa subdimensionamento de cobertura de braços) |
| V20 | `DeepPatternNesting` | **info** | Aninhamento de padrão > 2 fora de contexto de tabela de tuplas regulares (métrica de complexidade) |
| V21 | `HardcodedContextualValue` | **warning** | Literal numérico escala variável de fonte contextual e alimenta sumidouro geométrico sem proveniência declarada |
| V22 | `ProvenanceInventory` | **info** | Métrica agregada por módulo: rácio `(literais citados) / (total de literais)` para vigilância de tendência (opt-in) |
| V23 | `ContextErasure` | **warning** | Contexto requerido é neutralizado ou projetado antes de sumidouro declarado |
| V24 | `SemanticFieldLoss` | **warning** | Campo obrigatório de identidade/projeção é substituído por neutro |
| V25 | `DecisionOwnership` | **warning** | Decisão é duplicada, recomposta por proxy ou recanonicalizada fora do owner |

## Validação de refinamento

`refine` compara fatos explícitos de um artefato fonte e alvo. O resultado é
`PRESERVED` (exit 0), `VIOLATED` com testemunha (exit 1) ou `UNKNOWN` com razão
acionável (exit 2). O modo não lê Git, não executa comandos e não usa SMT.

```bash
crystalline-lint refine \
  --before before.refinement.json \
  --after after.refinement.json \
  --contract refinement.toml \
  --format text # ou sarif
```

Snapshots podem ser gerados deterministicamente por queries Rust explícitas:

```bash
crystalline-lint snapshot . \
  --contract refinement-self.toml \
  --artifact-id working-tree \
  --output working-tree.refinement.json
```

Consulte [USAGE.md](USAGE.md) para os formatos de snapshot e contrato.

**Sobre níveis Fatal (V0, V8, V10):** a ausência de violações garante
que todos os arquivos foram lidos e analisados com sucesso. Fatal
não pode ser suprimido por `--fail-on` — bloqueia CI
independentemente de qualquer configuração.

**Sobre V4:** a lista de símbolos proibidos é seleccionada por linguagem
via `forbidden_symbols_for(language)`. Em Rust, aliases de importação
são resolvidos para FQN antes da verificação — `use std::fs as f; f::read(...)`
é detectado como `std::fs::read`. Em TypeScript, Python, Go e Zig, call expressions
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
  --format <fmt>         sarif | text | n16-summary     [padrão: text]
  --fail-on <level>      error | warning                [padrão: error]
  --checks <list>        v0,v1,...,v25                  [padrão: V1–V21,V23–V25]
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
V13 = { level = "error" }
V14 = { level = "error" }
V15 = { level = "error" }
V16 = { level = "warning", languages = ["rust"] }
V17 = { level = "warning", languages = ["rust"] }
V18 = { level = "warning", languages = ["rust"] }
V19 = { level = "info", languages = ["rust"] }
V20 = { level = "info", languages = ["rust"] }
V21 = { level = "warning", languages = ["rust"] }
V22 = { level = "info", languages = ["rust"] }
V23 = { level = "warning", languages = ["rust"] }
V24 = { level = "warning", languages = ["rust"] }
V25 = { level = "warning", languages = ["rust"] }

# Configuração de contexto e sumidouros para V21
[v21]
# context_vars = ["size", "style", "em", "font", "weight", "ascent", "descent", "width", "height", "frame", "margin"]
# geometric_sinks = ["gap", "inset", "offset", "pos", "x", "y", "width", "height", "length", "pt", "em"]
# format_syntax_modules = ["export/pdf", "export/svg"]

# Exceções declaradas para V16 — hubs intencionais com razão técnica documentada
[wildcard_exceptions]
# "01_core/src/entities/gradient.rs:221" = "hub intencional: fallback lossy documentado no ADR-0109"

# Configuração do relatório N16 (limiar de amostra pequena)
[n16_summary]
min_sample_size = 5
```

---

## Formato de Saída: Relatório de Taxonomia N16 (`--format n16-summary`)

O formato `--format n16-summary` agrega as anotações de exceção de wildcard (`N16[α/β/γ]`) por módulo arquitetural, permitindo vigiar a concentração de fallbacks abertos (`γ`) de alto risco sem gerar ruído por ocorrência.

### Exemplo de Uso:
```bash
crystalline-lint --checks v16 --format n16-summary .
```

### Exemplo de Saída:
```markdown
| Módulo | Total | α | β | γ | % γ |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `layout/` | 20 | 1 | 15 | 4 | 20.0% |
| `introspect/` | 3 | 0 | 1 | 2 | 66.7% |
| `math/layout/` | 2 | 0 | 1 | 1 | 50.0% |
| `03_infra/` | 12 | 0 | 11 | 1 | 8.3% |
| `entities/` | 28 | 0 | 28 | 0 | 0.0% |
| `stdlib/` | 19 | 0 | 19 | 0 | 0.0% |
| `eval/` | 6 | 0 | 6 | 0 | 0.0% |
| `export/` | 1 | 1 | 0 | 0 | — |
| `parse/` | 1 | 0 | 1 | 0 | 0.0% |
| **Total** | **92** | **2** | **82** | **8** | **8.7%** |

⚠ amostra pequena em `introspect/` (n=3) — percentual pouco confiável, 1 caso muda o resultado em ~33pp
⚠ amostra pequena em `math/layout/` (n=2) — percentual pouco confiável, 1 caso muda o resultado em ~50pp
```

### Regras do Relatório:
1. **Ordenação:** Ordenado por gravidade absoluta (`γ` decrescente).
2. **Aviso de Amostra Pequena:** Módulos com `total < min_sample_size` e `γ > 0` exibem aviso indicando a sensibilidade percentual (`~pp`). O limiar padrão é 5, configurável via `[n16_summary] min_sample_size` no `crystalline.toml`.
3. **Escopo e Nível:** Modo puramente informativo (nível `info`), não falha CI e não introduz novas regras bloqueantes.

---

## Mecanismo de crescimento do predicado V21 (Memória Institucional)

A garantia que a regra V21 oferece não é de ausência arbitrária de números, mas de **impedir o reingresso de classes já nomeadas de valores contextuais fixados sem rastreabilidade**.

> **Regra de Processo:** Quando um placeholder ou escalar fixo de "fechar buraco" for encontrado e a V21 **não** o tiver detectado (falso negativo confirmado), o passo de correção do bug **deve** obrigatoriamente incluir a extensão do predicado da V21 (nova entrada em `context_vars`, `geometric_sinks` ou nova heurística). Corrigir o bug sem estender a regra é dívida técnica aberta.

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
│   │   │   ├── wiring-logic-leak.md  (V12)
│   │   │   ├── mutable-state-core.md (V13)
│   │   │   ├── external-type-in-contract.md (V14)
│   │   │   ├── multi-prompt-header.md (V15)
│   │   │   ├── wildcard-saturation.md (V16–V20)
│   │   │   └── unsourced-constant.md (V21–V22)
│   │   ├── file-walker.md
│   │   ├── prompt-walker.md
│   │   ├── sarif-formatter.md
│   │   └── fix-hashes.md
│   └── adr/
│       ├── 0001-tree-sitter-intermediate-repr.md
│       ├── 0002-typed-extensions-for-parsed-file.md
│       ├── 0003-code-to-prompt-feedback-direction.md
│       ├── 0004-reformulação-do-motor-de-análise.md
│       ├── 0005-location-owned-paths-e-cargo.toml-como-artefato-gerido.md
│       ├── 0006-fechamento-topológico-e-proteção-de-encapsulamento.md
│       ├── 0007-fechamento-comportamental-lab-contratos-fiacao.md
│       ├── 0008-estrategia-de-distribuicao.md
│       ├── 0009-isolamento-de-parsers-por-linguagem.md
│       ├── 0010-exclusao-ficheiros-individuais.md
│       ├── 0011-mutable-state-in-core.md
│       ├── 0012-external-type-in-contract.md
│       ├── 0013-import-vs-module-decl.md
│       ├── 0014-v11-configurable-level.md
│       ├── 0015-detecção-de-blanket-impls-para-V11.md
│       └── 0016-regras-decisao-mecanica.md
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
│   └── rules/                        # V1–V20
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
│       ├── wiring_logic_leak.rs      (V12)
│       ├── mutable_state_core.rs     (V13)
│       ├── external_type_in_contract.rs (V14)
│       ├── multi_prompt_header.rs    (V15)
│       ├── wildcard_saturation.rs    (V16)
│       ├── compound_guard.rs         (V17)
│       ├── range_pattern.rs          (V18)
│       ├── or_pattern_alternatives.rs (V19)
│       ├── deep_pattern_nesting.rs   (V20)
│       ├── unsourced_constant.rs     (V21)
│       └── provenance_inventory.rs   (V22)
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
