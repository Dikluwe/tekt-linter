# Prompt: File Walker (file-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Crawler
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

O linter precisa descobrir quais arquivos analisar e entregar
`SourceFile` limpo ao pipeline. Esta camada implementa a trait
`FileProvider` declarada em `01_core/contracts/file_provider.rs`.

Além de descobrir arquivos, o walker é responsável por injetar
em `SourceFile` o metadado `has_adjacent_test` — informação que
só existe no nível de filesystem e que o parser sozinho não
consegue derivar do source text.

---

## Responsabilidades

**Descoberta de arquivos:**
- Usar `walkdir` para varrer o diretório raiz configurado
- Ignorar `target/`, `node_modules/`, entradas do `.gitignore`
- Filtrar por extensão baseado nas linguagens habilitadas em
  `crystalline.toml` (`.rs` na v1)
- Determinar `Language` pelo mapeamento de extensão→grammar

**Metadado de teste adjacente:**

`SourceFile` recebe um campo adicional:
```rust
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: Language,
    pub has_adjacent_test: bool,
    // true se existe foo_test.rs no mesmo diretório que foo.rs
    // verificado pelo walker no momento da descoberta
}
```

O `RustParser` usa `SourceFile.has_adjacent_test` como fallback:
se `#[cfg(test)]` não está no AST, `has_test_coverage` é derivado
deste campo.

**Resolução de camada do arquivo:**

O walker determina `Layer` do arquivo pelo diretório pai,
baseado no mapeamento de `crystalline.toml`:
```rust
fn resolve_file_layer(path: &Path, config: &CrystallineConfig) -> Layer {
    // "01_core/..." → Layer::L1
    // "02_shell/..." → Layer::L2
    // etc.
}
```

Esse valor é passado para `SourceFile` e depois para `ParsedFile.layer`.

---

## Restrições

- Implementa `FileProvider` — retorna `impl Iterator<Item = SourceFile>`
- `std::io::Error` na leitura de arquivo é absorvido — arquivo
  ilegível é silenciosamente ignorado com log de warning
- Não contém nenhuma regra de violação
- Não invoca tree-sitter — apenas lê bytes e resolve metadados

---

## Critérios de Verificação
```
Dado diretório com foo.rs e foo_test.rs
Quando files() for chamado
Então SourceFile para foo.rs tem has_adjacent_test = true

Dado diretório com bar.rs sem bar_test.rs
Quando files() for chamado
Então SourceFile para bar.rs tem has_adjacent_test = false

Dado diretório com target/ contendo arquivos .rs
Quando files() for chamado
Então nenhum arquivo de target/ é retornado

Dado arquivo em 02_shell/api/auth.rs
Quando files() for chamado
Então SourceFile.layer = Layer::L2 (via crystalline.toml)

Dado arquivo ilegível por permissão
Quando files() for chamado
Então arquivo é ignorado sem panic
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | walker.rs |
| 2025-03-13 | Gap 4: adicionado has_adjacent_test em SourceFile, resolve_file_layer, delegação explícita ao RustParser | walker.rs |
