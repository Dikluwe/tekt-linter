# Prompt: File Walker (file-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Crawler
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-15 (ADR-0006: config.excluded, Layer::Unknown para aliens)
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
fn is_ignored(path: &Path, excluded: &HashMap<String, String>) -> bool {
    path.components().any(|c| {
        let name = c.as_os_str().to_str().unwrap_or("");
        excluded.values().any(|v| v == name)
    })
}
```

`excluded` é `&config.excluded` — mapa de `chave → nome_do_diretório`
declarado em `[excluded]` do toml. Nunca uma lista hardcoded.

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
        "rs"       => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "py"       => Some(Language::Python),
        _          => None,
    }
}
```

Extensão não reconhecida → `None` → arquivo filtrado silenciosamente.
Extensão desconhecida não é alien — alien é diretório fora de `[layers]`.

### 6. Metadado `has_adjacent_test`

`true` se `foo_test.rs` existe no mesmo diretório que `foo.rs`.
Arquivos que já terminam em `_test.rs` recebem `false` — eles
são o arquivo de teste, não o arquivo testado.

---

## Restrições

- `is_ignored` usa `config.excluded` — zero valores hardcoded
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

Dado arquivo foo_test.rs
Quando files() for consumido
Então SourceFile para foo_test.rs tem has_adjacent_test = false
— arquivo de teste não tem adjacent test, ele é o teste

Dado arquivo com extensão .toml ou .md
Quando files() for consumido
Então não aparece no iterator
— extensão não mapeada é filtrada silenciosamente
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | walker.rs |
| 2025-03-13 | Gap 4: has_adjacent_test, resolve_file_layer, delegação ao RustParser | walker.rs |
| 2026-03-14 | ADR-0004: SourceError, Result, iterator lazy, SourceFile não-Clone | walker.rs |
| 2026-03-15 | ADR-0006: is_ignored usa config.excluded (zero hardcode), Layer::Unknown não descartado, language_for_path documentado, critérios bar.rs/foo_test.rs/Layer::L2 restaurados | walker.rs |
