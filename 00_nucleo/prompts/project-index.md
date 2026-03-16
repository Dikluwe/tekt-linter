# Prompt: ProjectIndex (project-index)

**Camada**: L1 (Core — Entities)
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 01_core/entities/project_index.rs + test

---

## Contexto

V7 (Orphan Prompt) e V8 (Alien File) não podem ser verificadas
por arquivo — dependem de visão global do projeto. Exigem saber
o conjunto completo de prompts referenciados e o conjunto completo
de arquivos fora da topologia.

O pipeline paralelo (rayon, ADR-0004) impede acumulação via
mutação compartilhada. A solução é Map-Reduce:

- **Map**: cada thread produz um `LocalIndex` com os dados do
  seu arquivo
- **Reduce**: os `LocalIndex` são fundidos num `ProjectIndex`
  global via operação associativa e comutativa
- **Verify**: V7 e V8 rodam uma única vez sobre o índice global

`ProjectIndex` e `LocalIndex` são entidades puras de L1 —
zero I/O, construídas e fundidas sem estado compartilhado.

---

## Estruturas

### `LocalIndex<'a>` — produzido por thread
```rust
/// Contribuição de um único arquivo para o índice global.
/// Produzido durante a fase Map do pipeline paralelo.
/// Deve ser barato de construir e de fundir.
#[derive(Debug, Clone)]
pub struct LocalIndex<'a> {
    /// prompt_path referenciado pelo @prompt header deste arquivo.
    /// None se arquivo não tem header (V1 já cobre esse caso).
    pub referenced_prompt: Option<&'a str>,

    /// Se este arquivo tem Layer::Unknown e não está em excluídos.
    /// None se layer é conhecido. Some(path) se é alien.
    pub alien_file: Option<&'a Path>,
}

impl<'a> LocalIndex<'a> {
    pub fn empty() -> Self {
        Self { referenced_prompt: None, alien_file: None }
    }

    pub fn from_parsed(file: &ParsedFile<'a>) -> Self {
        Self {
            referenced_prompt: file.prompt_header
                .as_ref()
                .map(|h| h.prompt_path),
            alien_file: None, // arquivo parseado tem layer conhecida
        }
    }

    pub fn from_alien(path: &'a Path) -> Self {
        Self {
            referenced_prompt: None,
            alien_file: Some(path),
        }
    }

    pub fn from_source_error() -> Self {
        Self::empty() // V0 já cobre, não contribui para o índice
    }
}
```

### `ProjectIndex<'a>` — produzido pela fase Reduce
```rust
/// Índice global construído por fusão de todos os LocalIndex.
/// Entregue a V7 e V8 após o pipeline paralelo completar.
#[derive(Debug, Default)]
pub struct ProjectIndex<'a> {
    /// Todos os prompt_paths referenciados por @prompt headers
    /// em arquivos válidos de L1–L4.
    pub referenced_prompts: HashSet<&'a str>,

    /// Arquivos com Layer::Unknown fora de diretórios excluídos.
    pub alien_files: Vec<&'a Path>,
}

impl<'a> ProjectIndex<'a> {
    pub fn new() -> Self {
        Self {
            referenced_prompts: HashSet::new(),
            alien_files: Vec::new(),
        }
    }

    /// Absorve um LocalIndex — operação da fase Reduce.
    /// Associativa e comutativa — segura para rayon::fold.
    pub fn merge_local(&mut self, local: LocalIndex<'a>) {
        if let Some(prompt) = local.referenced_prompt {
            self.referenced_prompts.insert(prompt);
        }
        if let Some(path) = local.alien_file {
            self.alien_files.push(path);
        }
    }

    /// Funde dois ProjectIndex — para rayon::reduce.
    pub fn merge(mut self, other: ProjectIndex<'a>) -> ProjectIndex<'a> {
        self.referenced_prompts.extend(other.referenced_prompts);
        self.alien_files.extend(other.alien_files);
        self
    }
}
```

### `AllPrompts<'a>` — entregue por L3
```rust
/// Conjunto de todos os prompts existentes em 00_nucleo/prompts/,
/// excluindo as exceções declaradas em [orphan_exceptions].
/// Construído por L3 (PromptWalker) antes do pipeline paralelo.
/// Passado a V7 junto com o ProjectIndex.
pub struct AllPrompts<'a> {
    pub paths: HashSet<&'a str>,
}
```

`AllPrompts` é construído uma única vez por L3 antes do pipeline
paralelo iniciar — varredura sequencial de `00_nucleo/prompts/`.
Não participa do Map-Reduce porque não depende dos arquivos de
código.

---

## Pipeline Map-Reduce em L4
```rust
// Fase Map+Reduce paralela (rayon)
// Cada thread retorna (Vec<Violation>, LocalIndex)
let (all_violations, project_index): (Vec<Violation>, ProjectIndex) =
    walker
        .files()
        .par_bridge()
        .map(|result| -> (Vec<Violation>, LocalIndex) {
            match result {
                Ok(source) => match parser.parse(&source) {
                    Ok(parsed) => {
                        let violations = run_checks(&parsed, &enabled, &l1_ports);
                        let local = LocalIndex::from_parsed(&parsed);
                        (violations, local)
                    }
                    Err(err) => (
                        vec![parse_error_to_violation(err)],
                        LocalIndex::empty(),
                    ),
                },
                Err(err) => {
                    let local = LocalIndex::from_alien(err.path());
                    (vec![source_error_to_violation(err)], local)
                }
            }
        })
        .fold(
            || (Vec::new(), ProjectIndex::new()),
            |(mut viols, mut idx), (v, local)| {
                viols.extend(v);
                idx.merge_local(local);
                (viols, idx)
            },
        )
        .reduce(
            || (Vec::new(), ProjectIndex::new()),
            |(mut viols_a, idx_a), (viols_b, idx_b)| {
                viols_a.extend(viols_b);
                (viols_a, idx_a.merge(idx_b))
            },
        );

// Fase global — V7 e V8 sobre o índice completo
if enabled.v7 {
    all_violations.extend(check_orphans(&project_index, &all_prompts));
}
if enabled.v8 {
    all_violations.extend(check_aliens(&project_index));
}
```

**Por que é seguro:** cada thread trabalha em seu `LocalIndex`
local sem compartilhar estado. A fusão ocorre via `fold` e
`reduce` — operações funcionais puras que rayon garante sem
locks. `ProjectIndex::merge` é associativa e comutativa —
a ordem de fusão não afeta o resultado.

---

## Restrições (L1 Pura)

- `LocalIndex` e `ProjectIndex` são structs de dados puras —
  zero I/O, zero tree-sitter
- `merge_local` e `merge` são funções puras — sem mutação
  compartilhada, sem locks
- `AllPrompts` é construído por L3 antes do pipeline —
  não participa do Map-Reduce
- V7 e V8 recebem referências imutáveis ao índice final —
  nunca modificam o índice

---

## Critérios de Verificação
```
Dado dois LocalIndex com referenced_prompts distintos
Quando merge() for chamado
Então ProjectIndex.referenced_prompts contém a união dos dois

Dado LocalIndex::from_alien(path)
Quando merge_local() for chamado
Então ProjectIndex.alien_files contém o path

Dado LocalIndex::from_parsed(parsed_file com prompt_header)
Quando merge_local() for chamado
Então ProjectIndex.referenced_prompts contém o prompt_path

Dado LocalIndex::empty()
Quando merge_local() for chamado
Então ProjectIndex não muda

Dado três LocalIndex fundidos em sequências diferentes
Quando merge() for chamado em ordens distintas
Então ProjectIndex resultante é idêntico — comutatividade

Dado pipeline rayon com 100 arquivos
Quando fold + reduce completar
Então ProjectIndex contém exatamente os prompts e aliens
de todos os 100 arquivos — sem perda, sem duplicação indevida
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006): LocalIndex, ProjectIndex, AllPrompts, padrão Map-Reduce documentado | project_index.rs |
