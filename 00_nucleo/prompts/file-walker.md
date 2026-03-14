# Prompt: File Walker (file-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Crawler
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004)
**Arquivos gerados**:
  - 03_infra/walker.rs + test

---

## Contexto

O walker é o ponto de entrada de dados do linter. Descobre arquivos
no disco, carrega-os para RAM como `SourceFile` — o dono (`owner`)
do conteúdo — e resolve metadados básicos de camada e adjacência
de testes.

Implementa a trait `FileProvider` declarada em
`01_core/contracts/file_provider.rs`.

**Diretiva Fail-Fast (ADR-0004):** Arquivos ilegíveis não podem ser
descartados silenciosamente. A ausência de violações deve garantir
conformidade real — não que metade dos arquivos era ilegível. Erros
de I/O são propagados como `SourceError::Unreadable` e convertidos
em `V0 Fatal` pelo wiring em L4.

---

## Responsabilidades

### 1. Descoberta e filtro

- Usar `walkdir` para varrer o diretório raiz configurado
- Ignorar completamente: `target/`, `node_modules/`, `.git/`,
  `.cargo/` — não emite `SourceError` para esses, simplesmente
  não os visita
- Filtrar por extensão baseado nas linguagens habilitadas em
  `crystalline.toml` (`.rs` na v1)
- O iterador é lazy — leitura para RAM ocorre à medida que L4
  consome, não antecipadamente

### 2. Leitura segura com propagação de erro
```rust
match std::fs::read_to_string(&path) {
    Ok(content) => Some(Ok(SourceFile { path, content, .. })),
    Err(e) => Some(Err(SourceError::Unreadable {
        path,
        reason: e.to_string(),
    })),
}
```

`SourceFile` torna-se dono da `String` de conteúdo. Todo
`ParsedFile<'a>` derivado vive dentro do lifetime dessa String.

### 3. Metadados de arquivo

**`has_adjacent_test`** — verifica se existe `foo_test.rs` no
mesmo diretório que `foo.rs` no momento da descoberta. Arquivos
que já terminam em `_test.rs` recebem `false` — eles são o
arquivo de teste, não o arquivo testado.

**`layer`** — resolve comparando o primeiro componente do path
relativo à raiz contra o mapeamento `[layers]` de
`crystalline.toml`:
```rust
pub fn resolve_file_layer(path: &Path, root: &Path, config: &CrystallineConfig) -> Layer {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let first = relative.components().next()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");

    for (layer_key, dir_name) in &config.layers {
        if first == dir_name.as_str() {
            return match layer_key.as_str() {
                "L0" => Layer::L0, "L1" => Layer::L1,
                "L2" => Layer::L2, "L3" => Layer::L3,
                "L4" => Layer::L4, "lab" | "Lab" => Layer::Lab,
                _ => Layer::Unknown,
            };
        }
    }
    Layer::Unknown
}
```

**`language`** — mapeamento de extensão:
```rust
fn language_for_path(path: &Path) -> Option<Language> {
    match path.extension()?.to_str()? {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "py" => Some(Language::Python),
        _ => None,
    }
}
```

---

## Restrições

- Implementa `FileProvider` — retorna
  `impl Iterator<Item = Result<SourceFile, SourceError>>`
- Nunca suprime `io::Error` silenciosamente — sempre propaga
  como `SourceError::Unreadable`
- Ignora diretórios excluídos completamente — não emite erros
  para eles, apenas não os visita
- Não invoca tree-sitter
- Não contém nenhuma regra de violação
- `SourceFile` não implementa `Clone` — conteúdo carregado
  uma única vez por arquivo

---

## Critérios de Verificação
```
Dado diretório com foo.rs e foo_test.rs
Quando files() for consumido
Então Ok(SourceFile) para foo.rs tem has_adjacent_test = true

Dado diretório com bar.rs sem bar_test.rs
Quando files() for consumido
Então Ok(SourceFile) para bar.rs tem has_adjacent_test = false

Dado arquivo foo_test.rs
Quando files() for consumido
Então Ok(SourceFile) para foo_test.rs tem has_adjacent_test = false
— arquivo de teste não tem adjacent test, ele é o teste

Dado arquivo ilegível por permissão do SO
Quando files() for consumido
Então retorna Err(SourceError::Unreadable { path, reason })
E o iterator continua — não aborta nos arquivos seguintes

Dado diretório com target/debug/build.rs
Quando files() for consumido
Então target/debug/build.rs não aparece nem como Ok nem como Err
— diretório excluído não é visitado

Dado arquivo em 01_core/rules/forbidden_import.rs
Quando files() for consumido
Então SourceFile.layer = Layer::L1

Dado arquivo em 02_shell/cli.rs
Quando files() for consumido
Então SourceFile.layer = Layer::L2

Dado arquivo em 03_infra/walker.rs
Quando files() for consumido
Então SourceFile.layer = Layer::L3

Dado arquivo com extensão desconhecida (.toml, .md)
Quando files() for consumido
Então arquivo não aparece no iterator
— extensão não mapeada é filtrada silenciosamente

Dado mock de FileProvider retornando SourceFiles fixos
Quando usado em testes de L1
Então nenhum acesso a disco ocorre
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | walker.rs |
| 2025-03-13 | Gap 4: has_adjacent_test, resolve_file_layer, delegação ao RustParser | walker.rs |
| 2026-03-14 | ADR-0004: files() retorna Result<SourceFile, SourceError>, propagação de io::Error, iterator lazy, SourceFile não-Clone | walker.rs |
