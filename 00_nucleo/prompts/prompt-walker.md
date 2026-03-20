# Prompt: Prompt Walker (prompt-walker)

**Camada**: L3 (Infra)
**Padrão**: Filesystem Scanner
**Criado em**: 2026-03-14 (ADR-0006)
**Revisado em**: 2026-03-20 (propagação de erros granular — entrada inacessível é saltada)
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
passado como referência imutável ao longo de toda a execução.
Não participa do Map-Reduce porque não depende dos arquivos
de código.

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

**Propagação de erros:**
- Se `00_nucleo/prompts/` não puder ser lido (não existe ou
  sem permissão de leitura do directório raiz) →
  `PromptScanError::NucleoUnreadable` — correcto abortar
- Se uma entrada individual dentro de `prompts/` não for
  acessível (ficheiro ou subdirectório sem permissão) →
  entrada saltada silenciosamente, varredura continua.
  Análogo ao comportamento de `FileWalker` para `SourceError`.
- Se path contiver bytes inválidos UTF-8 →
  `PromptScanError::InvalidUtf8` para essa entrada

A distinção é importante: a ausência do directório raiz impede
qualquer varredura — o linter não pode garantir completude de V7
e deve falhar. Um ficheiro inacessível dentro de um directório
legível é um dado em falta, não uma falha de infra.

---

## Implementação
```rust
pub struct FsPromptWalker {
    pub project_root: PathBuf,
    pub orphan_exceptions: HashSet<String>,
    paths_buffer: std::cell::RefCell<Vec<Box<str>>>,
}

impl PromptProvider for FsPromptWalker {
    fn scan<'a>(&'a self) -> Result<AllPrompts<'a>, PromptScanError> {
        let prompts_dir = self.project_root.join("00_nucleo").join("prompts");

        if !prompts_dir.exists() {
            return Err(PromptScanError::NucleoUnreadable {
                path: prompts_dir.clone(),
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} não existe", prompts_dir.display()),
                ),
            });
        }

        let mut entries: HashSet<PromptEntry<'a>> = HashSet::new();

        for result in WalkDir::new(&prompts_dir) {
            // Entradas individuais inacessíveis são saltadas —
            // apenas a falha do directório raiz propaga como Err.
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let ext = entry.path().extension().and_then(|e| e.to_str());
            if ext != Some("md") {
                continue;
            }

            let relative = entry
                .path()
                .strip_prefix(&self.project_root)
                .map_err(|_| PromptScanError::NucleoUnreadable {
                    path: entry.path().to_path_buf(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "não foi possível calcular path relativo",
                    ),
                })?
                .to_str()
                .ok_or_else(|| PromptScanError::InvalidUtf8 {
                    path: entry.path().to_path_buf(),
                })?
                .to_string();

            if self.orphan_exceptions.contains(&relative) {
                continue;
            }

            let interned = self.intern(relative);
            entries.insert(PromptEntry { relative_path: interned });
        }

        Ok(AllPrompts { entries })
    }
}
```

---

## Restrições

- Implementa `PromptProvider` — retorna `Result<AllPrompts<'a>, PromptScanError>`
- `NucleoUnreadable` apenas para falha do directório raiz
- Entradas individuais inacessíveis dentro de `prompts/` são
  saltadas — não abortam a varredura
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

Dado arquivo em 00_nucleo/prompts/rules/v3.md
Quando scan() for chamado
Então AllPrompts contém PromptEntry {
    relative_path: "00_nucleo/prompts/rules/v3.md"
}

Dado 00_nucleo/ com permissão de leitura negada (directório raiz)
Quando scan() for chamado
Então retorna Err(PromptScanError::NucleoUnreadable)
— directório raiz inacessível → abortar

Dado uma entrada individual dentro de prompts/ inacessível
E o directório raiz 00_nucleo/prompts/ é legível
Quando scan() for chamado
Então a entrada é saltada silenciosamente
E as demais entradas são retornadas normalmente
— entrada individual inacessível não aborta a varredura

Dado arquivo não-.md em 00_nucleo/prompts/ (ex: .toml)
Quando scan() for chamado
Então não aparece em AllPrompts

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
| 2026-03-20 | Propagação de erros granular: entradas individuais inacessíveis são saltadas em vez de abortar; apenas falha do directório raiz retorna Err; critério adicionado | prompt_walker.rs |
