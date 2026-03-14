# Prompt: FileProvider (file-provider)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13

## Contexto

O linter precisa iterar sobre arquivos do projeto para analisá-los.
L1 não pode acessar disco — então declara aqui o contrato que L3
deve satisfazer para entregar arquivos ao pipeline de análise.

`SourceFile` é a unidade de transferência entre FileWalker (L3) e
LanguageParser (L3). Carrega não apenas o conteúdo do arquivo mas
todos os metadados deriváveis do filesystem antes do parse — para
que o parser nunca precise voltar ao disco.

---

## Instrução
```rust
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: Language,

    // Resolvido pelo walker via crystalline.toml
    pub layer: Layer,

    // true se foo_test.rs existe no mesmo diretório que foo.rs
    // verificado pelo walker no momento da descoberta
    // usado pelo RustParser como fallback quando #[cfg(test)]
    // não está presente no AST
    pub has_adjacent_test: bool,
}

pub trait FileProvider {
    fn files(&self) -> impl Iterator<Item = SourceFile>;
}
```

`Language` e `Layer` são importados de `crate::entities::layer`.

---

## Responsabilidades de população (L3 — FileWalker)

| Campo | Como |
|-------|------|
| `path` | Path absoluto do arquivo descoberto |
| `content` | Conteúdo lido via `std::fs::read_to_string` |
| `language` | Mapeamento extensão → Language (`.rs` → `Language::Rust`) |
| `layer` | Prefix do path contra mapeamento em `crystalline.toml` |
| `has_adjacent_test` | Existência de `foo_test.rs` no mesmo diretório |

Todos os campos são populados antes de `SourceFile` ser entregue
ao parser. O parser nunca acessa disco para complementar o que
o walker já deveria ter resolvido.

---

## Restrições

- L1: zero I/O, zero dependências externas
- `SourceFile` é imutável após construção por L3
- `content` já é `String` limpa — L3 absorveu `io::Error`
- `layer` e `has_adjacent_test` nunca são derivados pelo parser

---

## Critérios de Verificação
```
Dado FileProvider com dois SourceFiles
Quando files() for chamado
Então retorna iterator com exatamente dois itens

Dado SourceFile com path em 01_core/
Quando layer for acessado
Então layer = Layer::L1

Dado SourceFile para foo.rs com foo_test.rs adjacente
Quando has_adjacent_test for acessado
Então retorna true

Dado SourceFile para bar.rs sem bar_test.rs adjacente
Quando has_adjacent_test for acessado
Então retorna false

Dado mock de FileProvider retornando SourceFiles fixos
Quando usado em teste de L1
Então nenhum acesso a disco ocorre
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | file_provider.rs |
| 2025-03-13 | Gap 4: adicionados campos layer e has_adjacent_test, tabela de responsabilidades de população | file_provider.rs |
