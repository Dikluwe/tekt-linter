# ⚖️ ADR-0004: Reformulação do Motor de Análise

> **State Transformation Log**: Este documento registra a mutação
> estrutural do pipeline de linting, corrigindo quatro vetores de
> falha introduzidos na materialização inicial do motor.

---

## 💎 Formalism ($\mathcal{L}_{adr}$)

* **Estado Inicial**: Seja $M_0$ o motor atual — single-threaded,
  owned strings, erros de I/O suprimidos, símbolos não resolvidos.
* **Transformação**: $\Delta_{motor}$ define a transição para $M_1$
  preservando a topologia de camadas e os invariantes de L1.
* **Invariante de Conformidade**: $\neg\exists f \in Files :
  lint(f) = \emptyset \land \neg readable(f)$ — ausência de
  violações deve implicar conformidade real, não falha silenciosa
  de leitura.

---

## Status

`PROPOSTO`

## Data

2026-03-14

---

## Contexto

A separação conceitual de camadas está correta e validada. O que
falhou foi a execução do motor em L3 e L4 — quatro decisões de
implementação que comprometem as garantias fundamentais do linter.

### Falha 1 — Supressão silenciosa de erros de I/O

`FileWalker` ignora arquivos ilegíveis via `.ok()` e segue em
frente. Isso corrompe a premissa central de qualquer linter:

> **A ausência de violações deve garantir conformidade.**
> Atualmente pode apenas significar que o linter falhou em abrir o arquivo.

Um CI que passa com `exit 0` porque metade dos arquivos era
ilegível é mais perigoso que um CI que falha.

### Falha 2 — Pipeline single-threaded

O orquestrador em L4 processa arquivos sequencialmente —
leitura de disco, parse de AST e avaliação de regras num único
thread. Em bases de código com centenas de arquivos, isso é
inaceitável para uma ferramenta escrita em Rust, linguagem que
oferece paralelismo de dados sem custo de segurança.

### Falha 3 — Alocação descontrolada

`RustParser` clona strings da AST (`.to_string()`) para popular
`ParsedFile`. Cada arquivo analisado resulta em dezenas de
alocações heap desnecessárias. Isso destrói localidade de cache
e torna o linter mais lento que ferramentas equivalentes em
linguagens com GC.

### Falha 4 — Análise léxica disfarçada de semântica

V4 proíbe chamadas de I/O comparando símbolos crus extraídos do
código. Um alias simples desabilita a regra completamente:
```rust
use std::fs as f;
f::read("path")  // V4 não detecta — procura "std::fs", não "f"
```

Isso não é análise semântica — é regex sobre a AST. Um linter
arquitetural que pode ser burlado com um alias não oferece
garantias estruturais reais.

---

## Decisão

Reformular o pipeline em quatro frentes, preservando a topologia
de camadas e os contratos de L1.

### 1. Fail-Fast — Tolerância zero a erros de I/O

`FileProvider` passa a propagar erros de leitura em vez de
suprimi-los. O item do iterador muda de `SourceFile` para
`Result<SourceFile, SourceError>`:
```rust
pub enum SourceError {
    Unreadable { path: PathBuf, reason: String },
}

pub trait FileProvider {
    fn files(&self) -> impl Iterator<Item = Result<SourceFile, SourceError>>;
}
```

L4 converte `SourceError` em violação **bloqueante** `V0`:
```rust
pub enum ViolationLevel { Fatal, Error, Warning }
```

`V0 — UnreadableSource` tem nível `Fatal` — bloqueia CI
independentemente de `--fail-on`. Não há configuração que
permita ignorar um arquivo ilegível.

### 2. Pipeline concorrente — Paralelismo em L3/L4

O orquestrador em L4 usa `rayon` para transformar o iterador
de arquivos em `ParallelIterator`. Parse e avaliação de regras
ocorrem em paralelo:
```rust
// L4 — orquestração paralela
walker.files()
    .par_bridge()
    .map(|result| match result {
        Ok(source) => parser.parse(source)
            .map(|parsed| run_checks(&parsed, &enabled))
            .unwrap_or_else(|err| vec![parse_error_to_violation(err)]),
        Err(e) => vec![source_error_to_violation(e)],
    })
    .flatten()
    .collect()
```

`rayon` é dependência de L4 — nunca exposta a L1 ou L2.
L1 permanece completamente alheio ao modelo de execução.

### 3. Zero-copy — Referências em vez de alocações

`ParsedFile` e estruturas dependentes passam a referenciar
bytes carregados pelo walker em vez de cloná-los:
```rust
pub struct ParsedFile<'a> {
    pub path: &'a Path,
    pub layer: Layer,
    pub language: Language,
    pub prompt_header: Option<PromptHeader<'a>>,
    pub imports: Vec<Import<'a>>,
    pub tokens: Vec<Token<'a>>,
    pub public_interface: PublicInterface<'a>,
    pub prompt_snapshot: Option<PublicInterface<'a>>,
    // ...
}

pub struct Token<'a> {
    pub symbol: &'a str,
    pub line: usize,
    pub column: usize,
    pub kind: TokenKind,
}

pub struct Import<'a> {
    pub path: &'a str,
    pub line: usize,
    pub kind: ImportKind,
    pub target_layer: Layer,
}
```

O conteúdo do arquivo (`SourceFile.content: String`) vive no
walker e é emprestado por toda a vida do `ParsedFile`. Uma
única alocação por arquivo, zero cópias intermediárias.

**Consequência**: lifetimes `<'a>` se propagam para todos os
contratos que recebem ou produzem `ParsedFile`. Isso afeta
`LanguageParser`, `FileProvider` e todas as funções de regras
em L1. É a refatoração mais invasiva deste ADR — necessária
para consistência do modelo de memória.

### 4. Resolução de FQN — Semântica real em L3

`RustParser` constrói uma tabela de aliases durante o parse
dos `use_declaration` antes de processar `call_expression`:
```rust
// Fase 1 — construir tabela de aliases
// use std::fs as f;  →  aliases["f"] = "std::fs"
// use tokio::io as tio;  →  aliases["tio"] = "tokio::io"

// Fase 2 — resolver call_expressions
// f::read(...)  →  Token { symbol: "std::fs::read", ... }
// tio::stdin()  →  Token { symbol: "tokio::io::stdin", ... }
```

L1 recebe sempre Fully Qualified Names. V4 nunca vê aliases —
compara contra `FORBIDDEN_SYMBOLS` com a certeza de que os
símbolos já foram resolvidos.

A tabela de aliases é local ao arquivo — não há estado global
entre arquivos. Isso preserva a possibilidade de paralelismo
da decisão 2.

---

## Prompts Afetados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `violation-types.md` | Adicionar `ViolationLevel::Fatal`, `V0 — UnreadableSource`, lifetimes em todas as structs |
| `contracts/file-provider.md` | `files()` retorna `Iterator<Item = Result<SourceFile, SourceError>>` |
| `contracts/language-parser.md` | `parse()` recebe e retorna tipos com lifetime `<'a>` |
| `rs-parser.md` | Fase de resolução de aliases (FQN), zero-copy na extração de tokens |
| `file-walker.md` | Propagar `SourceError` em vez de silenciar `io::Error` |
| `linter-core.md` | Pipeline paralelo via rayon em L4, V0 como Fatal |
| `rules/impure-core.md` | Nota: símbolos já chegam resolvidos como FQN — sem mudança na lógica |

---

## Consequências

### ✅ Positivas

* **Garantia real de conformidade**: `exit 0` significa que todos
  os arquivos foram lidos e analisados — sem exceções silenciosas.
* **Performance em escala**: paralelismo automático via rayon sem
  mudança nos invariantes de L1.
* **Pressão de memória eliminada**: uma alocação por arquivo;
  tokens e imports são referências ao conteúdo original.
* **V4 inburlável**: aliases resolvidos em L3 antes de chegar
  ao núcleo — análise genuinamente semântica.

### ❌ Negativas

* **Infecção de lifetimes**: `<'a>` se propaga por L1 inteiro e
  pelos contratos. Refatoração pesada mas mecânica — o compilador
  Rust guia cada passo.
* **Complexidade do RustParser**: tabela de resolução de aliases
  aumenta significativamente a responsabilidade de L3. O parser
  passa a ter duas fases distintas por arquivo.
* **Reescrita de testes de L1**: todos os testes que constroem
  `ParsedFile` manualmente precisam ser atualizados para as
  novas assinaturas com lifetime.

### ⚙️ Neutras

* **rayon como dependência de L4**: não contamina L1, L2 ou L3.
  Pode ser removido ou substituído por tokio sem tocar nas regras.
* **V0 como nível Fatal**: extensão do enum `ViolationLevel` —
  não quebra lógica existente de V1–V6, apenas adiciona um nível
  acima de `Error`.
* **Aliases locais por arquivo**: a tabela não é compartilhada
  entre threads — sem necessidade de sincronização.

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Manter owned strings, usar arena allocator | Sem infecção de lifetimes | Complexidade de arena supera o ganho; lifetimes são mais idiomáticos em Rust |
| tokio em vez de rayon | Já familiar se o projeto evoluir para LSP/watch | Async desnecessário para I/O paralelo simples de linting em batch |
| Resolver FQN em L1 com tabela injetada | L1 mais poderoso | Viola pureza de L1 — tabela de símbolos é estado derivado de I/O |
| Logar ilegíveis como warning e continuar | Menos disruptivo | Mantém a garantia falsa — `exit 0` continuaria enganoso |

---

## Errata — 2026-03-14

**Seção 3 (Zero-Copy):** A diretiva original declarou `Token.symbol`
como `&'a str`. Isso é impossível quando o símbolo é construído por
resolução de alias — a string concatenada não existe no buffer original.

**Correção:** `Token.symbol` usa `Cow<'a, str>`:
- `Cow::Borrowed` para símbolos presentes literalmente no buffer
- `Cow::Owned` para FQNs construídos por resolução de alias

Todos os outros campos de `Token` e demais structs permanecem
`&'a str` — a contradição era localizada apenas em `Token.symbol`.

---

## Referências

* ADR-0001: Tree-sitter Intermediate Representation
* ADR-0002: Code-to-Prompt Feedback Direction
* `00_nucleo/prompts/contracts/file-provider.md`
* `00_nucleo/prompts/rules/impure-core.md`
* Rust Reference: Lifetime elision rules
* rayon crate documentation: `ParallelIterator`
