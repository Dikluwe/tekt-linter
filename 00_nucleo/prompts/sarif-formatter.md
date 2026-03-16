# Prompt: SARIF & CLI Formatter (sarif-formatter)

**Camada**: L2 (Shell)
**Padrão**: CLI Controller e Presenter
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-15 (ADR-0006: V7, V8, V9)

---

## Contexto

Uma vez que o check das regras do L1 retorna seu catálogo puro
`Vec<Violation>`, a fronteira do linter precisa moldar e publicar
estas sanções nos standard outputs apropriados de forma entendível
para terminais ou rotinas GitHub Actions.

Também é responsável por comandos de mutação — operações que
reescrevem arquivos do projeto. Atualmente: `--fix-hashes` e
`--update-snapshot`.

---

## Responsabilidades CLI

A camada Shell define e consome o framework de argumento `clap`
para capturar as intenções do usuário. Traduz interações impuras
para ordens puras do lado de dentro. Gerencia `exit_code=1` se
houver infração fatal reportada pelo L1.

**Flags completas:**
```
crystalline-lint [OPTIONS] [PATH]

ARGS:
  [PATH]    Raiz do projeto a analisar [padrão: .]

OPTIONS:
  --format <fmt>         sarif | text | json        [padrão: text]
  --fail-on <level>      error | warning            [padrão: error]
  --checks <list>        v0,v1,v2,v3,v4,v5,v6,v7,v8,v9  [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml           [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --update-snapshot      atualiza Interface Snapshot nos prompts (V6)
  --dry-run              usado com --fix-hashes ou --update-snapshot
```

**Combinações inválidas — CLI retorna exit 1 com mensagem:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Notas sobre V0 e V8:**
`--checks` pode omitir `v0` ou `v8` para suprimir output, mas
ambos são Fatal — sempre bloqueiam CI independentemente de
`--fail-on` e `--checks`.

---

## Responsabilidades Output (SARIF)

- Transformar `Vec<Violation>` em JSON válido sob SARIF `2.1.0`
- Popular `runs.tool.driver.rules` com metadados de V0–V9
- Mapear cada `Violation` em `runs.results.region.startLine`
- Como fallback (`--format text`): strings coloridas legíveis
  para stdout, estilo output do Cargo

**Tabela de regras SARIF:**

| ID | Nome | Level padrão |
|----|------|--------------|
| V0 | UnreadableSource | fatal |
| V1 | MissingPromptHeader | error |
| V2 | MissingTestFile | error |
| V3 | ForbiddenImport | error |
| V4 | ImpureCore | error |
| V5 | PromptDrift | warning |
| V6 | PromptStale | warning |
| V7 | OrphanPrompt | warning |
| V8 | AlienFile | fatal |
| V9 | PubLeak | error |

---

## Responsabilidades Fix

**Quando `--fix-hashes` está presente:**
- Filtrar violations por `rule_id == "V5"`
- Delegar reescrita para `HashRewriter` de L3 (via adapter L4)
- Se `--dry-run`: apenas reportar, não reescrever
- Após correção: re-executar análise e confirmar zero V5

**Quando `--update-snapshot` está presente:**
- Filtrar violations por `rule_id == "V6"`
- Delegar serialização + escrita para `SnapshotWriter` (via adapter L4)
- Se `--dry-run`: apenas reportar interface que seria escrita
- Após atualização: re-executar análise e confirmar zero V6

---

## EnabledChecks
```rust
pub struct EnabledChecks {
    pub v1: bool,
    pub v2: bool,
    pub v3: bool,
    pub v4: bool,
    pub v5: bool,
    pub v6: bool,
    pub v7: bool,
    pub v8: bool,
    pub v9: bool,
}

impl EnabledChecks {
    pub fn from_cli(checks: &str, no_drift: bool, no_stale: bool) -> Self {
        let lower = checks.to_lowercase();
        Self {
            v1: lower.contains("v1"),
            v2: lower.contains("v2"),
            v3: lower.contains("v3"),
            v4: lower.contains("v4"),
            v5: lower.contains("v5") && !no_drift,
            v6: lower.contains("v6") && !no_stale,
            v7: lower.contains("v7"),
            v8: lower.contains("v8"),
            v9: lower.contains("v9"),
        }
    }
}
```

**Nota sobre V7 e V8 no pipeline:**
V7 e V8 são verificados na fase global pós-reduce, não por arquivo.
`enabled.v7` e `enabled.v8` controlam se as verificações globais
são executadas após o Map-Reduce — não são passados para `run_checks`.
`enabled.v9` é passado para `run_checks` por arquivo.

---

## Padrão L2

Impuro (controla STD/Exit), mas não contém regras. Delega
parseamento (L3→L1) e atua como adapter final de display
(L1→L2→out) e de mutação (L1→L2→L3→disco).

L2 nunca importa L3 diretamente — adapters são injetados via L4.

---

## Critérios de Verificação
```
Dado Vec<Violation> vazio
Quando format_text() for chamado
Então output contém "No violations found"

Dado Vec<Violation> com V6 warning
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V6" e level "warning"

Dado Vec<Violation> com V7 warning
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V7" e level "warning"

Dado Vec<Violation> com V8 fatal
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V8" e level "error"
— SARIF não tem nível "fatal", V8 mapeado para "error" no output

Dado Vec<Violation> com V9 error
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V9" e level "error"

Dado --dry-run sem --fix-hashes e sem --update-snapshot
Quando validate_args() for chamado
Então retorna Err com mensagem de uso

Dado --fix-hashes e --update-snapshot simultaneamente
Quando validate_args() for chamado
Então retorna Err com mensagem de uso

Dado --no-stale
Quando EnabledChecks::from_cli() for chamado
Então v6 = false

Dado --checks v1,v3,v9
Quando EnabledChecks::from_cli() for chamado
Então v1 = true, v2 = false, v3 = true, v4 = false,
     v5 = false, v6 = false, v7 = false, v8 = false, v9 = true

Dado --checks all (padrão)
Quando EnabledChecks::from_cli() for chamado
Então v1..v9 = true (exceto v5 se --no-drift, v6 se --no-stale)
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | cli.rs |
| 2025-03-13 | --fix-hashes e --dry-run, responsabilidades de mutação | cli.rs |
| 2025-03-13 | V6: --update-snapshot, --no-stale, V6 na tabela SARIF | cli.rs |
| 2026-03-14 | ADR-0004: V0 na tabela SARIF, EnabledChecks atualizado | cli.rs |
| 2026-03-15 | ADR-0006: V7, V8, V9 nas flags, tabela SARIF, EnabledChecks, nota sobre V7/V8 na fase global vs V9 por arquivo | cli.rs |
