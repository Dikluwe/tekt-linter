# Prompt: SARIF & CLI Formatter (sarif-formatter)

**Camada**: L2 (Shell)
**Padrão**: CLI Controller e Presenter
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

Uma vez que o check das regras do L1 retorna seu catálogo puro
`Vec<Violation>`, a fronteira do linter precisa moldar e publicar
estas sanções nos standard outputs apropriados de forma entendível
para terminais ou rotinas GitHub Actions.

Também é responsável por comandos de mutação — operações que
reescrevem arquivos do projeto. Atualmente: `--fix-hashes`.

## Responsabilidades CLI

A camada Shell define e consome o framework de argumento `clap`
para capturar as intenções do usuário. Traduz interações impuras
para ordens puras do lado de dentro. Gerencia `exit_code=1` se
houver infração fatal reportada pelo L1.

**Flags completas:**
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
  --fix-hashes         corrige @prompt-hash divergentes (reescreve arquivos)
  --dry-run            usado com --fix-hashes: mostra mudanças sem reescrever
```

`--dry-run` sem `--fix-hashes` é erro de uso — CLI deve
reportar e retornar exit 1.

## Responsabilidades Output (SARIF)

- Transformar `Vec<Violation>` em JSON válido sob SARIF `2.1.0`
- Popular `runs.tool.driver.rules` com metadados fixos de L1
- Mapear cada `Violation` em `runs.results.region.startLine`
- Como fallback (`--format text`): strings coloridas legíveis
  para stdout, estilo output do Cargo

## Responsabilidades Fix

Quando `--fix-hashes` está presente:
- Filtrar violations por `rule_id == "V5"`
- Delegar reescrita para `HashWriter` de L3
- Se `--dry-run`: apenas reportar, não reescrever
- Após correção: re-executar análise e confirmar zero V5

## Padrão L2

Impuro (controla STD/Exit), mas não contém regras. Delega
parseamento (L3→L1) e atua como adapter final de display
(L1→L2→out) e de mutação (L1→L2→L3→disco).

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | cli.rs |
| 2025-03-13 | Adicionado --fix-hashes e --dry-run, responsabilidades de mutação | cli.rs |
```

---

