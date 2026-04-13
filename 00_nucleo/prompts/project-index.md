# Prompt: ProjectIndex (project-index)
Hash do Código: d6b0a29e

**Camada**: L1 (Core — Entities)
**Criado em**: 2026-03-14 (ADR-0006)
**Revisado em**: 2026-03-16 (ADR-0007: declared_traits e implemented_traits para V11)
**Arquivos gerados**:
  - 01_core/entities/project_index.rs + test

---

## Contexto

V7 (Orphan Prompt), V8 (Alien File) e V11 (Dangling Contract) não
podem ser verificadas por arquivo — dependem de visão global do
projeto. Exigem saber, respectivamente, o conjunto completo de
prompts referenciados, o conjunto completo de arquivos fora da
topologia, e o conjunto completo de traits declaradas em
`01_core/contracts/` versus traits implementadas em L2/L3.

O pipeline paralelo (rayon, ADR-0004) impede acumulação via
mutação compartilhada. A solução é Map-Reduce:

- **Map**: cada thread produz um `LocalIndex` com os dados do
  seu arquivo
- **Reduce**: os `LocalIndex` são fundidos num `ProjectIndex`
  global via operação associativa e comutativa
- **Verify**: V7, V8 e V11 rodam uma única vez sobre o índice
  global

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
    /// None se layer é conhecida. Some(path) se é alien.
    pub alien_file: Option<&'a Path>,

    /// Traits públicas declaradas neste arquivo em L1/contracts/.
    /// Vazio para arquivos fora de L1 ou fora de subdir "contracts".
    /// Populado pelo RustParser a partir de nós `trait_item` com `pub`.
    /// Usado por V11 para detectar contratos sem implementação.
    pub declared_traits: Vec<&'a str>,

    /// Traits implementadas neste arquivo via `impl Trait for Type`.
    /// Vazio para arquivos fora de L2 e L3.
    /// Populado pelo RustParser a partir de nós `impl_item` com trait.
    /// Usado por V11 para fechar o circuito contrato → implementação.
    pub implemented_traits: Vec<&'a str>,
}

impl<'a> LocalIndex<'a> {
    pub fn empty() -> Self {
        Self {
            referenced_prompt: None,
            alien_file: None,
            declared_traits: vec![],
            implemented_traits: vec![],
        }
    }

    /// Constrói LocalIndex a partir de um ParsedFile.
    ///
    /// Detecta aliens internamente: se layer == Layer::Unknown,
    /// popula alien_file com o path do arquivo.
    ///
    /// `declared_traits` e `implemented_traits` são lidos dos
    /// campos homônimos de ParsedFile, populados por RustParser.
    /// from_parsed não os deriva — apenas os transporta.
    pub fn from_parsed(file: &ParsedFile<'a>) -> Self {
        Self {
            referenced_prompt: file.prompt_header
                .as_ref()
                .map(|h| h.prompt_path),
            alien_file: if file.layer == Layer::Unknown {
                Some(file.path)
            } else {
                None
            },
            declared_traits: file.declared_traits.clone(),
            implemented_traits: file.implemented_traits.clone(),
        }
    }

    /// Constrói LocalIndex para arquivo que falhou no parse.
    /// Não é alien — arquivo tem layer conhecida mas conteúdo inválido.
    pub fn from_parse_error() -> Self {
        Self::empty()
    }

    pub fn from_source_error() -> Self {
        Self::empty() // V0 já cobre, não contribui para o índice
    }
}
```

**Por que `declared_traits` e `implemented_traits` vivem em
`ParsedFile` e não são extraídos diretamente pelo `LocalIndex`:**
`LanguageParser::parse()` retorna `Result<ParsedFile<'a>, ParseError>`.
Alterar a assinatura para retornar também um `LocalIndex` parcial
quebraria o contrato e exporia L1 à lógica de indexação. A solução
mais limpa é que `ParsedFile` carregue os campos — `from_parsed`
os lê sem derivar nada. Os campos ficam em `ParsedFile` mas nenhuma
regra de L1 os acessa diretamente, apenas o `LocalIndex`.

### `ProjectIndex<'a>` — produzido pela fase Reduce
```rust
/// Índice global construído por fusão de todos os LocalIndex.
/// Entregue a V7, V8 e V11 após o pipeline paralelo completar.
#[derive(Debug, Default)]
pub struct ProjectIndex<'a> {
    /// Todos os prompt_paths referenciados por @prompt headers
    /// em arquivos válidos de L1–L4.
    pub referenced_prompts: HashSet<&'a str>,

    /// Arquivos com Layer::Unknown fora de diretórios excluídos.
    pub alien_files: Vec<&'a Path>,

    /// Todas as traits públicas declaradas em L1/contracts/.
    /// Agregado de LocalIndex.declared_traits de todos os arquivos L1.
    /// Usado por V11 para detectar contratos sem implementação.
    pub all_declared_traits: HashSet<&'a str>,

    /// Todas as traits implementadas em L2 ou L3.
    /// Agregado de LocalIndex.implemented_traits de todos os arquivos L2/L3.
    /// Usado por V11 para fechar o circuito contrato → implementação.
    pub all_implemented_traits: HashSet<&'a str>,
}

impl<'a> ProjectIndex<'a> {
    pub fn new() -> Self {
        Self {
            referenced_prompts: HashSet::new(),
            alien_files: Vec::new(),
            all_declared_traits: HashSet::new(),
            all_implemented_traits: HashSet::new(),
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
        self.all_declared_traits.extend(local.declared_traits);
        self.all_implemented_traits.extend(local.implemented_traits);
    }

    /// Funde dois ProjectIndex — para rayon::reduce.
    pub fn merge(mut self, other: ProjectIndex<'a>) -> ProjectIndex<'a> {
        self.referenced_prompts.extend(other.referenced_prompts);
        self.alien_files.extend(other.alien_files);
        self.all_declared_traits.extend(other.all_declared_traits);
        self.all_implemented_traits.extend(other.all_implemented_traits);
        self
    }
}
```

### `AllPrompts<'a>` — entregue por L3
```rust
/// Conjunto de todos os prompts existentes em 00_nucleo/prompts/,
/// excluindo as exceções declaradas em [orphan_exceptions].
/// Construído por L3 (FsPromptWalker) antes do pipeline paralelo.
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
                        let violations = run_checks(&parsed, &enabled, &l1_ports, &wiring_config);
                        // from_parsed detecta Layer::Unknown e transporta
                        // declared_traits/implemented_traits internamente
                        let local = LocalIndex::from_parsed(&parsed);
                        (violations, local)
                    }
                    Err(err) => (
                        vec![parse_error_to_violation(err)],
                        LocalIndex::from_parse_error(),
                    ),
                },
                Err(err) => (
                    vec![source_error_to_violation(&err)],
                    LocalIndex::from_source_error(),
                ),
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

// Fase global — V7, V8 e V11 sobre o índice completo
if enabled.v7 {
    all_violations.extend(check_orphans(&project_index, &all_prompts));
}
if enabled.v8 {
    all_violations.extend(check_aliens(&project_index));
}
if enabled.v11 {
    all_violations.extend(check_dangling_contracts(&project_index));
}
```

**Garantias de segurança:**
- Cada thread opera sobre `LocalIndex` próprio — sem estado
  compartilhado
- `fold` acumula por thread, `reduce` funde threads — ambos
  funcionais puros
- `ProjectIndex::merge` é associativa e comutativa — ordem de
  fusão não afeta resultado
- `AllPrompts` é imutável durante todo o pipeline paralelo

---

## Restrições (L1 Pura)

- `LocalIndex` e `ProjectIndex` são structs de dados puras —
  zero I/O, zero tree-sitter
- `from_parsed` detecta `Layer::Unknown` internamente e
  transporta `declared_traits`/`implemented_traits` de `ParsedFile`
  — não os deriva
- `merge_local` e `merge` são funções puras — sem mutação
  compartilhada, sem locks
- `AllPrompts` é construído por L3 antes do pipeline —
  não participa do Map-Reduce
- V7, V8 e V11 recebem referências imutáveis ao índice final —
  nunca modificam o índice
- V11 compara por nome simples de trait — limitação declarada
  em `dangling-contract.md`

---

## Critérios de Verificação
```
Dado dois LocalIndex com referenced_prompts distintos
Quando merge() for chamado
Então ProjectIndex.referenced_prompts contém a união dos dois

Dado LocalIndex construído via from_parsed com layer = Layer::Unknown
Quando merge_local() for chamado
Então ProjectIndex.alien_files contém o path

Dado LocalIndex construído via from_parsed com layer = Layer::L1
Quando merge_local() for chamado
Então ProjectIndex.alien_files permanece vazio
— layer conhecida não é alien

Dado LocalIndex::from_parsed(parsed_file com prompt_header)
Quando merge_local() for chamado
Então ProjectIndex.referenced_prompts contém o prompt_path

Dado LocalIndex::empty()
Quando merge_local() for chamado
Então ProjectIndex não muda

Dado três LocalIndex fundidos em sequências diferentes
Quando merge() for chamado em ordens distintas
Então ProjectIndex resultante é idêntico — comutatividade

Dado pipeline rayon com 100 arquivos onde 3 têm Layer::Unknown
Quando fold + reduce completar
Então ProjectIndex.alien_files contém exatamente 3 paths
E ProjectIndex.referenced_prompts contém todos os prompts
dos 97 arquivos com layer conhecida

Dado ParsedFile com layer = L1, subdir = "contracts"
E declared_traits = ["FileProvider", "LanguageParser"]
Quando LocalIndex::from_parsed() for chamado
Então local.declared_traits == ["FileProvider", "LanguageParser"]

Dado LocalIndex com declared_traits = ["FileProvider"]
E outro LocalIndex com declared_traits = ["LanguageParser"]
Quando merge_local() for chamado em sequência
Então ProjectIndex.all_declared_traits contém ambas as traits

Dado LocalIndex com implemented_traits = ["FileProvider"]
Quando merge_local() for chamado
Então ProjectIndex.all_implemented_traits contém "FileProvider"

Dado all_declared_traits = {"FileProvider", "LanguageParser"}
E all_implemented_traits = {"FileProvider"}
Quando check_dangling_contracts() for chamado
Então retorna uma violação V11 mencionando "LanguageParser"
E não retorna V11 para "FileProvider"

Dado LocalIndex::from_source_error()
Quando merge_local() for chamado
Então ProjectIndex não muda em nenhum campo
— erros de fonte não contribuem para o índice

Dado LocalIndex::from_parse_error()
Quando merge_local() for chamado
Então ProjectIndex não muda em nenhum campo
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006): LocalIndex, ProjectIndex, AllPrompts, padrão Map-Reduce documentado | project_index.rs |
| 2026-03-15 | from_parsed detecta Layer::Unknown internamente — from_alien() removido. Pipeline em L4 sempre chama from_parsed(). from_parse_error() adicionado para clareza. | project_index.rs |
| 2026-03-16 | ADR-0007: declared_traits e implemented_traits em LocalIndex; all_declared_traits e all_implemented_traits em ProjectIndex; merge_local e merge atualizados; from_parsed transporta os novos campos; decisão de design documentada (campos em ParsedFile, não retorno duplo de parse()); pipeline L4 atualizado com V11; critérios de V11 adicionados | project_index.rs |
| 2026-03-16 | Materialização ADR-0007: declared_traits e implemented_traits adicionados a LocalIndex; all_declared_traits e all_implemented_traits adicionados a ProjectIndex; empty(), from_parsed(), from_parse_error(), from_source_error(), merge_local() e merge() atualizados; 8 novos testes | project_index.rs |
