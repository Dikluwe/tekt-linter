# Prompt: Prompt Walker (prompt-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Scanner
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 03_infra/prompt_walker.rs + test

---

## Contexto

V7 (Orphan Prompt) precisa do conjunto completo de prompts em
`00_nucleo/prompts/` para comparar com os prompts referenciados
pelo código. Este componente L3 faz a varredura e constrói
`AllPrompts`.

Implementa a trait `PromptProvider` declarada em
`01_core/contracts/prompt_provider.rs`.

**Timing:** Invocado sequencialmente em L4 antes do pipeline
paralelo iniciar. `AllPrompts` é construído uma única vez e
passado como referência imutável ao longo de toda a execução —
incluindo durante o Map-Reduce do rayon. Não participa do
Map-Reduce porque não depende dos arquivos de código.

---

## Responsabilidades

**Descoberta de prompts:**
- Usar `walkdir` para varrer `00_nucleo/prompts/` recursivamente
- Incluir apenas arquivos `.md`
- Excluir entradas declaradas em `[orphan_exceptions]` do
  `crystalline.toml` antes de retornar

**Construção de paths relativos:**
- Cada `PromptEntry.relative_path` é o path relativo à raiz
  de `00_nucleo/` — não ao projeto inteiro
- Exemplo: arquivo em `00_nucleo/prompts/rules/v3.md` →
  `relative_path = "prompts/rules/v3.md"`
- Isso garante comparabilidade direta com `@prompt` headers
  que declaram `00_nucleo/prompts/rules/v3.md`

**Propagação de erros:**
- Se `00_nucleo/` não puder ser lido → `PromptScanError::NucleoUnreadable`
- Se path contiver bytes inválidos UTF-8 → `PromptScanError::InvalidUtf8`
- Nunca silencia erros — se o nucleo é ilegível, o linter não
  pode garantir completude de V7

---

## Implementação
```rust
pub struct FsPromptWalker {
    pub nucleo_root: PathBuf,
    pub orphan_exceptions: HashSet<String>,
}

impl PromptProvider for FsPromptWalker {
    fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError> {
        let prompts_dir = self.nucleo_root.join("prompts");

        let entries: Result<HashSet<PromptEntry<'a>>, PromptScanError> =
            WalkDir::new(&prompts_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| {
                    e.path().extension()
                        .and_then(|ext| ext.to_str())
                        == Some("md")
                })
                .filter_map(|e| {
                    // Path relativo a nucleo_root
                    let relative = e.path()
                        .strip_prefix(&self.nucleo_root)
                        .ok()?
                        .to_str()
                        .map(|s| s.to_string())?;

                    // Excluir orphan_exceptions
                    if self.orphan_exceptions.contains(&relative) {
                        return None;
                    }

                    // &'a str que vive no walker
                    // (ver nota sobre lifetime abaixo)
                    Some(Ok(PromptEntry {
                        relative_path: self.intern(relative),
                    }))
                })
                .collect();

        Ok(AllPrompts { entries: entries? })
    }
}
```

**Nota sobre lifetime e interning:**
`PromptEntry.relative_path` é `&'a str` que deve sobreviver
ao lifetime de `FsPromptWalker`. A implementação usa um
`Vec<String>` interno ao walker para armazenar os paths
construídos, e retorna referências a essas strings:
```rust
pub struct FsPromptWalker {
    pub nucleo_root: PathBuf,
    pub orphan_exceptions: HashSet<String>,
    // Buffer interno para paths interned
    paths_buffer: std::cell::RefCell<Vec<String>>,
}

impl FsPromptWalker {
    fn intern<'a>(&'a self, path: String) -> &'a str {
        let mut buf = self.paths_buffer.borrow_mut();
        buf.push(path);
        // SAFETY: referência ao último elemento,
        // Vec não realoca durante scan() pois borrow é exclusivo
        buf.last().unwrap().as_str()
    }
}
```

Alternativa sem unsafe: usar `Arc<str>` em `PromptEntry`
e remover o lifetime. Decisão de implementação para L3 —
o contrato em L1 aceita ambas as abordagens.

---

## Restrições

- Implementa `PromptProvider` — retorna `Result<AllPrompts<'a>, PromptScanError>`
- Nunca silencia erros de leitura do diretório nucleo
- Exclui `[orphan_exceptions]` antes de retornar — V7 nunca
  vê prompts excluídos
- Não contém nenhuma regra de violação
- Não invoca tree-sitter
- Invocado sequencialmente — não precisa ser Send + Sync

---

## Critérios de Verificação
```
Dado 00_nucleo/prompts/ com três arquivos .md
E nenhuma orphan_exception configurada
Quando scan() for chamado
Então AllPrompts.len() == 3

Dado arquivo prompts/readme.md em orphan_exceptions
Quando scan() for chamado
Então AllPrompts não contém "prompts/readme.md"
— exceção excluída antes de retornar

Dado arquivo em 00_nucleo/prompts/rules/v3.md
Quando scan() for chamado
Então AllPrompts contém PromptEntry {
    relative_path: "prompts/rules/v3.md"
}
— path relativo ao nucleo_root, não ao projeto

Dado 00_nucleo/ com permissão de leitura negada
Quando scan() for chamado
Então retorna Err(PromptScanError::NucleoUnreadable)
— nunca silencia erro de acesso ao nucleo

Dado arquivo não-.md em 00_nucleo/prompts/ (ex: .toml)
Quando scan() for chamado
Então não aparece em AllPrompts
— apenas .md são prompts

Dado AllPrompts construído por scan()
Quando passado como referência imutável ao pipeline rayon
Então acesso concorrente é seguro — AllPrompts não muta
após construção
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | prompt_walker.rs |
