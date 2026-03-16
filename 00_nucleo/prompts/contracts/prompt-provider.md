# Prompt: PromptProvider (prompt-provider)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 01_core/contracts/prompt_provider.rs

---

## Contexto

V7 (Orphan Prompt) precisa do conjunto completo de prompts
existentes em `00_nucleo/prompts/` para comparar com os prompts
referenciados pelo código. L1 não pode varrer o disco — este
contrato define o que L3 deve entregar.

`PromptProvider` é análogo a `FileProvider`: define a fronteira
entre a varredura de disco (L3) e a verificação de regras (L1).
A diferença é que `PromptProvider` é invocado sequencialmente
antes do pipeline paralelo — não participa do Map-Reduce.

---

## Estruturas

### `PromptEntry<'a>`
```rust
/// Um prompt descoberto em 00_nucleo/prompts/.
/// Carrega apenas o path relativo à raiz do nucleo —
/// suficiente para comparação com @prompt headers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PromptEntry<'a> {
    /// Path relativo a 00_nucleo/.
    /// Exemplo: "prompts/rules/forbidden-import.md"
    pub relative_path: &'a str,
}
```

### `AllPrompts<'a>`
```rust
/// Conjunto de todos os prompts existentes em 00_nucleo/prompts/,
/// excluindo as exceções declaradas em [orphan_exceptions].
/// Construído por L3 (FsPromptWalker) antes do pipeline paralelo.
/// Imutável durante toda a execução — seguro para acesso concorrente.
#[derive(Debug)]
pub struct AllPrompts<'a> {
    pub entries: HashSet<PromptEntry<'a>>,
}

impl<'a> AllPrompts<'a> {
    pub fn contains(&self, path: &str) -> bool {
        self.entries.iter().any(|e| e.relative_path == path)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
```

---

## Contrato (Trait)
```rust
pub trait PromptProvider {
    /// Varre 00_nucleo/prompts/ e retorna todos os prompts
    /// existentes, excluindo as exceções configuradas.
    ///
    /// Invocado sequencialmente uma única vez antes do pipeline
    /// paralelo. O resultado é passado como referência imutável
    /// a V7 após a fase Reduce.
    ///
    /// Erros de leitura de diretório são propagados — não
    /// silenciados. Se 00_nucleo/ não puder ser lido, o linter
    /// não pode garantir completude e deve falhar.
    fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError>;
}

/// Erro ao varrer 00_nucleo/prompts/.
/// Distinto de SourceError — ocorre antes do pipeline paralelo.
#[derive(Debug)]
pub enum PromptScanError {
    NucleoUnreadable { reason: String },
    InvalidUtf8 { path: PathBuf },
}
```

---

## Restrições

- L1: trait pura — zero std::fs, zero walkdir
- `AllPrompts<'a>` é imutável após construção — segura para
  referência concorrente durante pipeline paralelo
- `PromptScanError` é erro de infraestrutura — L4 converte
  em falha fatal se `scan()` retornar `Err`
- `PromptEntry.relative_path` é `&'a str` — o buffer dos
  paths vive no `FsPromptWalker` em L3

---

## Critérios de Verificação
```
Dado AllPrompts com "prompts/auth.md" nas entries
Quando contains("prompts/auth.md") for chamado
Então retorna true

Dado AllPrompts sem "prompts/missing.md"
Quando contains("prompts/missing.md") for chamado
Então retorna false

Dado mock de PromptProvider retornando AllPrompts fixo
Quando usado em teste de V7
Então nenhum acesso a disco ocorre

Dado PromptScanError::NucleoUnreadable
Quando L4 receber o Err de scan()
Então linter falha com mensagem explicativa antes de iniciar pipeline
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | prompt_provider.rs |
