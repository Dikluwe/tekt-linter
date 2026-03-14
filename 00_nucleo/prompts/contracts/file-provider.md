# Prompt: Contract - File Provider (file-provider)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004)
**Arquivos gerados**:
  - 01_core/contracts/file_provider.rs

---

## Contexto

Este contrato define como L1 espera receber arquivos de código do
mundo exterior. L1 não sabe o que é um disco rígido nem se importa
com `walkdir`.

`SourceFile` é o **dono** (`owner`) do conteúdo carregado em memória.
`ParsedFile<'a>` e `Violation<'a>` referenciam fatias desse conteúdo
via lifetime `'a` — nunca copiam. A ordem de destruição é garantida
pelo compilador: `SourceFile` deve viver mais que qualquer estrutura
que o referencia.

**Diretiva Fail-Fast (ADR-0004):** Um linter que silencia erros de
I/O gera falsos negativos — `exit 0` pode apenas significar que
metade dos arquivos era ilegível. O contrato proíbe essa supressão:
erros de leitura são propagados como `SourceError` e convertidos
em violações `V0 Fatal` pelo wiring em L4.

---

## Estruturas de Dados

### `SourceFile`
```rust
/// Dono (owner) do conteúdo do arquivo em memória.
/// ParsedFile<'a> e Violation<'a> pegam fatias emprestadas daqui.
/// Destruir SourceFile invalida todos os ParsedFile derivados dele.
#[derive(Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub language: Language,
    /// Resolvido pelo walker via crystalline.toml
    pub layer: Layer,
    /// true se foo_test.rs existe no mesmo diretório que foo.rs
    /// Verificado pelo walker no momento da descoberta
    pub has_adjacent_test: bool,
}
```

### `SourceError` (ADR-0004)
```rust
/// Falha crítica ao tentar carregar um arquivo.
/// Convertida em Violation { rule_id: "V0", level: Fatal } pelo wiring.
/// Não pode ser suprimida — ausência de V0 garante que todos os
/// arquivos foram lidos com sucesso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    Unreadable {
        path: PathBuf,
        reason: String,
    },
}
```

---

## Contrato (Trait)
```rust
pub trait FileProvider {
    /// Retorna iterador de arquivos lidos com sucesso ou erros de acesso.
    /// L4 usa par_bridge() para paralelizar — o iterador deve ser
    /// Send se a implementação suportar paralelismo.
    /// Nunca retorna Ok() para arquivo parcialmente lido.
    fn files(&self) -> impl Iterator<Item = Result<SourceFile, SourceError>>;
}
```

---

## Restrições

- `FileProvider` nunca silencia arquivos ilegíveis — propaga `SourceError`
- Ignora intencionalmente apenas diretórios configurados para exclusão
  (`target/`, `node_modules/`, `.git/`) — não arquivos individuais
- `SourceFile` não implementa `Clone` intencionalmente — o conteúdo
  é grande e deve ser carregado uma única vez por arquivo
- L1 nunca instancia `SourceFile` diretamente — é responsabilidade
  exclusiva de L3

---

## Critérios de Verificação
```
Dado diretório com dois arquivos .rs legíveis
Quando files() for chamado
Então retorna iterator com dois Ok(SourceFile)

Dado arquivo com permissão de leitura negada
Quando files() for chamado
Então retorna Err(SourceError::Unreadable) para esse arquivo
E continua iterando os demais — não aborta o iterator

Dado diretório com target/ contendo arquivos .rs
Quando files() for chamado
Então nenhum arquivo de target/ aparece no iterator

Dado SourceFile com path em 01_core/
Quando layer for acessado
Então layer = Layer::L1

Dado SourceFile para foo.rs com foo_test.rs adjacente
Quando has_adjacent_test for acessado
Então retorna true

Dado mock de FileProvider retornando SourceFiles fixos
Quando usado em teste de L1
Então nenhum acesso a disco ocorre

Dado FileProvider com um Ok e um Err
Quando L4 processar o iterator
Então Ok → ParsedFile → regras
E Err → Violation { rule_id: "V0", level: Fatal }
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | file_provider.rs |
| 2025-03-13 | Gap 4: adicionados layer e has_adjacent_test, tabela de responsabilidades | file_provider.rs |
| 2026-03-14 | ADR-0004: SourceError adicionado, files() retorna Result, nota sobre ownership e zero-copy, SourceFile não-Clone | file_provider.rs |
