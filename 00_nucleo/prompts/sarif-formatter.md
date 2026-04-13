# Prompt: SARIF & CLI Formatter (sarif-formatter)
Hash do Código: 9cc62545

**Camada**: L2 (Shell)
**Padrão**: CLI Controller e Presenter
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-20 (nota de ordenação: violations chegam já ordenadas de L4)

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
  --format <fmt>         sarif | text | json                  [padrão: text]
  --fail-on <level>      error | warning                      [padrão: error]
  --checks <list>        v0,v1,...,v12                        [padrão: all]
  --no-drift             desabilita V5
  --no-stale             desabilita V6
  --machine-readable     alias para --format sarif
  --quiet                apenas exit code, sem output
  --config <path>        crystalline.toml                     [padrão: ./crystalline.toml]
  --fix-hashes           corrige @prompt-hash divergentes (V5)
  --update-snapshot      atualiza Interface Snapshot nos prompts (V6)
  --dry-run              usado com --fix-hashes ou --update-snapshot
```

**Combinações inválidas — CLI retorna exit 1 com mensagem:**
- `--dry-run` sem `--fix-hashes` ou `--update-snapshot`
- `--fix-hashes` e `--update-snapshot` simultaneamente

**Notas sobre V0, V8 e V10:**
`--checks` pode omitir `v0`, `v8` ou `v10` para suprimir output,
mas os três são Fatal — sempre bloqueiam CI independentemente de
`--fail-on` e `--checks`.

---

## Responsabilidades Output (SARIF e texto)

- Transformar `Vec<Violation>` em JSON válido sob SARIF `2.1.0`
- Popular `runs.tool.driver.rules` com metadados de V0–V12
- Mapear cada `Violation` em `runs.results.region.startLine`
- Como fallback (`--format text`): strings coloridas legíveis
  para stdout, estilo output do Cargo

**Nota sobre ordenação:** o formatter recebe violations já
ordenadas por L4 (Fatal → Error → Warning, depois por path
e linha). O formatter não ordena — apenas formata o que recebe.
Nunca reordenar dentro do formatter.

**Tabela de regras SARIF:**

| ID  | Nome | Level padrão |
|-----|------|--------------|
| V0  | UnreadableSource | fatal → mapeado para `error` no SARIF |
| V1  | MissingPromptHeader | error |
| V2  | MissingTestFile | error |
| V3  | ForbiddenImport | error |
| V4  | ImpureCore | error |
| V5  | PromptDrift | warning |
| V6  | PromptStale | warning |
| V7  | OrphanPrompt | warning |
| V8  | AlienFile | fatal → mapeado para `error` no SARIF |
| V9  | PubLeak | error |
| V10 | QuarantineLeak | fatal → mapeado para `error` no SARIF |
| V11 | DanglingContract | error |
| V12 | WiringLogicLeak | warning |

*SARIF 2.1.0 não tem nível `fatal`. V0, V8 e V10 são mapeados
para `"error"` no output SARIF. O comportamento Fatal (bloqueia
CI independentemente de `--fail-on`) é aplicado pelo linter
internamente antes de consultar o nível SARIF.*

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
    pub v10: bool,
    pub v11: bool,
    pub v12: bool,
}

impl EnabledChecks {
    pub fn from_cli(checks: &str, no_drift: bool, no_stale: bool) -> Self {
        // Parsing por token exacto após split — evita falso positivo onde
        // "v11".contains("v1") == true e "v12".contains("v1") == true.
        let tokens: std::collections::HashSet<&str> = checks
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let has = |id: &str| -> bool {
            tokens.contains("all") || tokens.contains(id)
        };

        Self {
            v1:  has("v1"),
            v2:  has("v2"),
            v3:  has("v3"),
            v4:  has("v4"),
            v5:  has("v5") && !no_drift,
            v6:  has("v6") && !no_stale,
            v7:  has("v7"),
            v8:  has("v8"),
            v9:  has("v9"),
            v10: has("v10"),
            v11: has("v11"),
            v12: has("v12"),
        }
    }
}
```

**Semântica de `--checks`:**
- Cada token comparado exactamente após trim — `"v1"` ≠ `"v11"`
- Tokens desconhecidos ignorados silenciosamente
- `--checks v11,v12` activa apenas V11 e V12

**Nota sobre V7, V8 e V11 no pipeline:**
V7, V8 e V11 são verificados na fase global pós-reduce, não por
arquivo. `enabled.v9`, `enabled.v10` e `enabled.v12` são passados
para `run_checks` por arquivo.

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

Dado Vec<Violation> com violations de níveis mistos
Quando format_text() for chamado
Então o formatter preserva a ordem recebida — não reordena
— a ordenação é responsabilidade de L4, não do formatter

Dado Vec<Violation> com V6 warning
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V6" e level "warning"

Dado Vec<Violation> com V8 fatal
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V8" e level "error"
— SARIF não tem nível "fatal", V8 mapeado para "error"

Dado Vec<Violation> com V10 fatal
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V10" e level "error"

Dado Vec<Violation> com V11 error
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V11" e level "error"

Dado Vec<Violation> com V12 warning
Quando format_sarif() for chamado
Então SARIF contém resultado com ruleId "V12" e level "warning"

Dado --dry-run sem --fix-hashes e sem --update-snapshot
Quando validate_args() for chamado
Então retorna Err com mensagem de uso

Dado --fix-hashes e --update-snapshot simultaneamente
Quando validate_args() for chamado
Então retorna Err com mensagem de uso

Dado --checks v11
Quando EnabledChecks::from_cli() for chamado
Então v11 = true, v1 = false, v2 = false

Dado --checks v12
Quando EnabledChecks::from_cli() for chamado
Então v12 = true, v1 = false, v2 = false

Dado --checks v11,v12
Quando EnabledChecks::from_cli() for chamado
Então v11 = true, v12 = true, v1 = false, v2 = false

Dado --checks all (padrão)
Quando EnabledChecks::from_cli() for chamado
Então v1..v12 = true (exceto v5 se --no-drift, v6 se --no-stale)

Dado Vec<Violation> com apenas V10 Fatal
Quando should_fail() for chamado com --fail-on error
Então retorna true — V10 Fatal bloqueia independentemente de --fail-on

Dado format_sarif() com violations V0–V12
Quando chamado
Então SARIF driver.rules contém exatamente 13 entradas (V0 a V12)
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | cli.rs |
| 2025-03-13 | --fix-hashes e --dry-run, responsabilidades de mutação | cli.rs |
| 2025-03-13 | V6: --update-snapshot, --no-stale, V6 na tabela SARIF | cli.rs |
| 2026-03-14 | ADR-0004: V0 na tabela SARIF, EnabledChecks atualizado | cli.rs |
| 2026-03-15 | ADR-0006: V7, V8, V9 nas flags, tabela SARIF, EnabledChecks | cli.rs |
| 2026-03-16 | ADR-0007: V10, V11, V12; nota Fatal para V10; nota V11 fase global | cli.rs |
| 2026-03-20 | from_cli: split(',') com token exacto elimina falso positivo v1/v11 | cli.rs |
| 2026-03-20 | Nota de ordenação: violations chegam já ordenadas de L4; formatter não reordena; critério adicionado | cli.rs |
