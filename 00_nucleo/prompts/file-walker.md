# Prompt: File Walker (file-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Crawler
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-20 (teste adjacente TypeScript adicionado)
**Arquivos gerados**:
  - 03_infra/walker.rs + test

---

## Contexto

O walker é o ponto de entrada de dados do linter. Descobre arquivos
no disco, carrega-os para RAM como `SourceFile` e resolve metadados
básicos de camada e adjacência de testes.

Implementa a trait `FileProvider` declarada em
`01_core/contracts/file_provider.rs`.

**Diretiva Fail-Fast (ADR-0004):** Arquivos ilegíveis propagam
`SourceError::Unreadable` — nunca silenciados. A ausência de
violações deve garantir que todos os arquivos foram lidos.

**Diretiva de Fechamento Topológico (ADR-0006):** A distinção entre
diretório *excluído* e diretório *desconhecido* é crítica para V8:

- **Excluído** (`[excluded]` no toml) → não visitado, silêncio total
- **Desconhecido** (fora de `[layers]` e `[excluded]`) → visitado,
  `Layer::Unknown`, alimenta `LocalIndex::from_alien()` → V8 Fatal

Arquivos com `Layer::Unknown` **não são descartados** — chegam ao
pipeline para que V8 possa reportá-los.

**Diretiva Zero Hardcode (ADR-0006):** Nenhum nome de diretório
(`target`, `node_modules`, `.git`) pode ser hardcoded no walker.
Toda exclusão vem de `config.excluded`.

---

## Responsabilidades

### 1. Descoberta e filtro

- Usar `walkdir` para varrer o diretório raiz configurado
- **Excluir** apenas diretórios cujos nomes aparecem nos valores
  de `config.excluded` — lido de `[excluded]` no `crystalline.toml`
- **Nunca hardcodar** nomes de diretórios
- Filtrar por extensão baseado nas linguagens habilitadas em
  `crystalline.toml`
- O iterador é lazy — leitura para RAM ocorre à medida que L4
  consome

### 2. Função `is_ignored`
```rust
fn is_ignored(
    path: &Path,
    root: &Path,
    excluded_dirs: &HashSet<String>,
    excluded_files: &HashSet<String>,
) -> bool {
    if path.components().any(|c| {
        let name = c.as_os_str().to_str().unwrap_or("");
        excluded_dirs.contains(name)
    }) {
        return true;
    }
    if let Ok(relative) = path.strip_prefix(root) {
        if let Some(rel_str) = relative.to_str() {
            let normalized = rel_str.replace('\\', "/");
            if excluded_files.contains(&normalized) {
                return true;
            }
        }
    }
    false
}
```

`excluded_dirs` é construído de `config.excluded` (valores) — mapa de
`chave → nome_do_diretório` declarado em `[excluded]` do toml. Nunca
uma lista hardcoded.

**Nota:** `is_ignored` compara **componentes de path** (segmentos de
directório) para `excluded_dirs`. Portanto `[excluded]` é adequado
para **directórios** como `target`, `.git`, `node_modules`. Não deve
ser usado para excluir ficheiros individuais como `lib.rs` — isso
excluiria qualquer ficheiro com esse nome em qualquer subdirectório
do projecto.

### 7. Exclusão de ficheiros individuais — `[excluded_files]`

Distinto de `[excluded]` que opera sobre nomes de directório,
`[excluded_files]` exclui ficheiros por path relativo à raiz:

```toml
[excluded_files]
crate_root = "lib.rs"
```

`is_ignored` verifica primeiro os directórios (comportamento
existente) e depois os ficheiros individuais via `strip_prefix`
contra a raiz do projecto. Paths são normalizados (`\\` → `/`)
para comparação cross-platform.

### 3. Leitura segura com propagação de erro
```rust
match std::fs::read_to_string(&path) {
    Ok(content) => Some(Ok(SourceFile {
        path,
        content,
        language,
        layer,
        has_adjacent_test,
    })),
    Err(e) => Some(Err(SourceError::Unreadable {
        path,
        reason: e.to_string(),
    })),
}
```

`SourceFile` torna-se dono da `String` de conteúdo. Todo
`ParsedFile<'a>` derivado vive dentro do lifetime dessa String.

### 4. Resolução de layer
```rust
pub fn resolve_file_layer(
    path: &Path,
    root: &Path,
    config: &CrystallineConfig,
) -> Layer {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let first = relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");

    for (layer_key, dir_name) in &config.layers {
        if first == dir_name.as_str() {
            return match layer_key.as_str() {
                "L0" => Layer::L0,
                "L1" => Layer::L1,
                "L2" => Layer::L2,
                "L3" => Layer::L3,
                "L4" => Layer::L4,
                "lab" | "Lab" => Layer::Lab,
                _ => Layer::Unknown,
            };
        }
    }
    // Arquivo fora de [layers] e fora de [excluded] → Layer::Unknown
    // Não descartar — L4 converte em LocalIndex::from_alien() → V8
    Layer::Unknown
}
```

### 5. Mapeamento de linguagem
```rust
fn language_for_path(path: &Path) -> Option<Language> {
    match path.extension()?.to_str()? {
        "rs"         => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "py"         => Some(Language::Python),
        _            => None,
    }
}
```

Extensão não reconhecida → `None` → arquivo filtrado silenciosamente.
Extensão desconhecida não é alien — alien é diretório fora de `[layers]`.

### 6. Metadado `has_adjacent_test`

`true` se existe ficheiro de teste no mesmo directório que o ficheiro
analisado, seguindo as convenções de cada linguagem:

| Linguagem | Ficheiro analisado | Padrões de teste adjacente |
|-----------|-------------------|---------------------------|
| Rust | `foo.rs` | `foo_test.rs` |
| TypeScript | `foo.ts` ou `foo.tsx` | `foo.test.ts`, `foo.spec.ts`, `foo.test.tsx`, `foo.spec.tsx` |
| Python | `foo.py` | `foo_test.py`, `test_foo.py` |

Ficheiros que já são ficheiros de teste recebem `false` — eles
são o arquivo de teste, não o arquivo testado. Os padrões de
identificação são:
- Rust: stem termina em `_test`
- TypeScript: stem contém `.test` ou `.spec`
- Python: stem termina em `_test` ou começa com `test_`

```rust
fn check_adjacent_test(path: &Path) -> bool {
    let stem = match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => return false,
    };
    let dir = match path.parent() {
        Some(d) => d,
        None => return false,
    };

    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => {
            // Pular se já é ficheiro de teste
            if stem.ends_with("_test") { return false; }
            dir.join(format!("{}_test.rs", stem)).exists()
        }
        Some("ts") | Some("tsx") => {
            // Pular se já é ficheiro de teste
            if stem.contains(".test") || stem.contains(".spec") { return false; }
            dir.join(format!("{}.test.ts",  stem)).exists()
                || dir.join(format!("{}.spec.ts",  stem)).exists()
                || dir.join(format!("{}.test.tsx", stem)).exists()
                || dir.join(format!("{}.spec.tsx", stem)).exists()
        }
        Some("py") => {
            // Pular se já é ficheiro de teste
            if stem.ends_with("_test") || stem.starts_with("test_") { return false; }
            dir.join(format!("{}_test.py", stem)).exists()
                || dir.join(format!("test_{}.py", stem)).exists()
        }
        _ => false,
    }
}
```

---

## Restrições

- `is_ignored` usa `config.excluded` — zero valores hardcoded
- `config.excluded` é para **directórios**, não ficheiros individuais
- `Layer::Unknown` para arquivos fora de `[layers]` — nunca descartados
- Arquivos em diretórios excluídos não aparecem no iterator —
  nem como `Ok` nem como `Err`
- `SourceFile` não implementa `Clone` — conteúdo carregado
  uma única vez por arquivo
- Não invoca tree-sitter
- Não contém nenhuma regra de violação

---

## Critérios de Verificação
```
Dado config.excluded = { "build": "target" }
E arquivo target/debug/build.rs
Quando files() for consumido
Então arquivo não aparece no iterator — nem Ok nem Err

Dado config.excluded vazio
E arquivo target/debug/build.rs
Quando files() for consumido
Então arquivo aparece com Layer::Unknown
— sem hardcode, exclusão vem só da config

Dado arquivo src/utils/helper.rs
E "src" não em config.layers nem config.excluded
Quando files() for consumido
Então SourceFile.layer = Layer::Unknown
— não descartado, alimenta V8 via LocalIndex::from_alien()

Dado arquivo em 02_shell/api/auth.rs
Quando files() for consumido
Então SourceFile.layer = Layer::L2

Dado arquivo ilegível por permissão
Quando files() for consumido
Então retorna Err(SourceError::Unreadable)
E iterator continua nos demais arquivos — não aborta

Dado diretório com foo.rs e foo_test.rs
Quando files() for consumido
Então SourceFile para foo.rs tem has_adjacent_test = true

Dado diretório com bar.rs sem bar_test.rs
Quando files() for consumido
Então SourceFile para bar.rs tem has_adjacent_test = false

Dado diretório com foo.ts e foo.test.ts
Quando files() for consumido
Então SourceFile para foo.ts tem has_adjacent_test = true

Dado diretório com foo.ts e foo.spec.ts
Quando files() for consumido
Então SourceFile para foo.ts tem has_adjacent_test = true

Dado diretório com foo.tsx e foo.test.tsx
Quando files() for consumido
Então SourceFile para foo.tsx tem has_adjacent_test = true

Dado diretório com bar.ts sem foo.test.ts nem bar.spec.ts
Quando files() for consumido
Então SourceFile para bar.ts tem has_adjacent_test = false

Dado arquivo foo.test.ts (já é ficheiro de teste)
Quando files() for consumido
Então SourceFile para foo.test.ts tem has_adjacent_test = false
— ficheiro de teste não tem adjacent test, ele é o teste

Dado diretório com foo.py e foo_test.py
Quando files() for consumido
Então SourceFile para foo.py tem has_adjacent_test = true

Dado diretório com foo.py e test_foo.py
Quando files() for consumido
Então SourceFile para foo.py tem has_adjacent_test = true

Dado arquivo foo_test.rs
Quando files() for consumido
Então SourceFile para foo_test.rs tem has_adjacent_test = false
— arquivo de teste não tem adjacent test, ele é o teste

Dado arquivo com extensão .toml ou .md
Quando files() for consumido
Então não aparece no iterator
— extensão não mapeada é filtrada silenciosamente

Dado excluded_files = { "crate_root": "lib.rs" }
E arquivo lib.rs na raiz do projecto
Quando files() for consumido
Então lib.rs não aparece no iterator

Dado excluded_files = { "crate_root": "lib.rs" }
E arquivo 01_core/lib.rs (subdirectório)
Quando files() for consumido
Então 01_core/lib.rs aparece no iterator com Layer::L1
— excluded_files é por path relativo exacto, não por nome

Dado excluded_files vazio
E arquivo lib.rs na raiz
Quando files() for consumido
Então lib.rs aparece com Layer::Unknown (sem excluded_files, é alien)
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | walker.rs |
| 2025-03-13 | Gap 4: has_adjacent_test, resolve_file_layer, delegação ao RustParser | walker.rs |
| 2026-03-14 | ADR-0004: SourceError, Result, iterator lazy, SourceFile não-Clone | walker.rs |
| 2026-03-15 | ADR-0006: is_ignored usa config.excluded (zero hardcode), Layer::Unknown não descartado, language_for_path documentado, critérios bar.rs/foo_test.rs/Layer::L2 restaurados | walker.rs |
| 2026-03-20 | Padrões de teste adjacente TypeScript adicionados (.test.ts, .spec.ts, .test.tsx, .spec.tsx); check_adjacent_test refactored por linguagem via extensão do ficheiro; nota sobre config.excluded ser para directórios, não ficheiros individuais; critérios TypeScript adicionados | walker.rs |
| 2026-03-20 | ADR-0010: [excluded_files] para exclusão por path relativo; is_ignored actualizado com dois conjuntos (excluded_dirs + excluded_files); critérios adicionados | walker.rs |
