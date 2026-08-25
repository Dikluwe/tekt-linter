# Prompt: Contract - Language Parser (language-parser)
Hash do Código: cbe2c3dc

**Camada**: L1 (Core — Contracts)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-14 (ADR-0004)
**Arquivos gerados**:
  - 01_core/contracts/language_parser.rs

---

## Contexto

Este contrato define a fronteira de tradução entre texto bruto e
a Representação Intermediária Cristalina. L1 não conhece tree-sitter
nem regras sintáticas específicas de nenhuma linguagem.

**Diretiva Zero-Copy (ADR-0004):** O parser recebe `&'a SourceFile`
e devolve `ParsedFile<'a>` que referencia fatias do conteúdo original.
Nenhuma string do código-fonte é copiada — o compilador Rust garante
via lifetime que `ParsedFile` não pode sobreviver ao `SourceFile` que
o originou.

**Diretiva FQN (ADR-0004):** Antes de extrair `call_expression` do
AST, o parser constrói uma tabela de aliases local ao arquivo a partir
dos `use_declaration`. Todos os tokens entregues a L1 contêm Fully
Qualified Names resolvidos. L1 nunca vê aliases.

---

## Contrato (Trait)
```rust
use crate::contracts::parse_error::ParseError;
use crate::contracts::file_provider::SourceFile;
use crate::entities::parsed_file::ParsedFile;

pub trait LanguageParser {
    /// Traduz SourceFile em ParsedFile<'a>.
    ///
    /// O lifetime <'a> garante que ParsedFile não pode sobreviver
    /// ao SourceFile original — zero dangling pointers.
    ///
    /// O parser executa duas fases internas antes de retornar:
    ///   Fase 1: constrói tabela de aliases a partir de use_declaration
    ///   Fase 2: extrai tokens, resolvendo aliases para FQN
    ///
    /// Erros de gramática → ParseError::SyntaxError
    /// Linguagem sem grammar → ParseError::UnsupportedLanguage
    /// Conteúdo vazio → ParseError::EmptySource
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError>;
}
```

## Política pura de seleção e composição

O universo de slots é fechado e não contém tipos concretos de L3:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserSlot {
    Rust,
    TypeScript,
    Python,
    C,
    Cpp,
    Zig,
    Go,
    Java,
    Elixir,
}

pub fn parser_slot(language: &Language) -> Option<ParserSlot>;
```

A função é total e usa exclusivamente `language`:

| `Language` | resultado |
|---|---|
| `Rust` | `Some(ParserSlot::Rust)` |
| `TypeScript` | `Some(ParserSlot::TypeScript)` |
| `Python` | `Some(ParserSlot::Python)` |
| `C` | `Some(ParserSlot::C)` |
| `Cpp` | `Some(ParserSlot::Cpp)` |
| `Zig` | `Some(ParserSlot::Zig)` |
| `Go` | `Some(ParserSlot::Go)` |
| `Java` | `Some(ParserSlot::Java)` |
| `Elixir` | `Some(ParserSlot::Elixir)` |
| `Unknown` | `None` |

L1 também publica uma composição sobre ports, sem conhecer adapters concretos:

```rust
pub struct ParserSet<'p> {
    pub rust: &'p dyn LanguageParser,
    pub typescript: &'p dyn LanguageParser,
    pub python: &'p dyn LanguageParser,
    pub c: &'p dyn LanguageParser,
    pub cpp: &'p dyn LanguageParser,
    pub zig: &'p dyn LanguageParser,
    pub go: &'p dyn LanguageParser,
    pub java: &'p dyn LanguageParser,
    pub elixir: &'p dyn LanguageParser,
}

impl ParserSet<'_> {
    pub fn parse<'a>(&self, file: &'a SourceFile)
        -> Result<ParsedFile<'a>, ParseError>;
}
```

`ParserSet::parse` consulta `parser_slot` uma vez. Para um slot suportado, chama
exatamente o port correspondente uma vez com o mesmo `&SourceFile` e devolve o mesmo
`Ok` ou `Err`, sem tradução, fallback ou segunda tentativa. Os demais ports não são
consultados. Para `Unknown`, nenhum port é consultado e o retorno direto é
`ParseError::UnsupportedLanguage { path: file.path.clone(), language:
file.language.clone() }`.

Os nove ports são obrigatórios na construção: ausência de adapter não é estado de
runtime e não compartilha a semântica de `Unknown`. Ordem dos campos não define
precedência. A política depende apenas de `Language`; path, content, layer e
`has_adjacent_test` não alteram o slot. Ela não acessa filesystem, configuração,
ambiente, relógio, rede ou processo.

L4 instancia os nove adapters L3 e constrói `ParserSet`; não repete o `match`, não decide
fallback e apenas inicia `ParserSet::parse`. Um wrapper `MultiParser` privado em L4 é
permitido somente se for estruturalmente transparente e não reimplementar a decisão.

---

## Contrato de duas fases (para implementadores em L3)

O parse deve ser executado em duas fases distintas e ordenadas:

**Fase 1 — Symbol Tracking (tabela de aliases):**
Percorre todos os `use_declaration` do arquivo e constrói um mapa
local `alias → FQN`:
```
use std::fs as f      →  aliases["f"]   = "std::fs"
use tokio::io as tio  →  aliases["tio"] = "tokio::io"
use std::fs           →  aliases["fs"]  = "std::fs"  (último segmento)
```

A tabela é local ao arquivo — não compartilhada entre threads.
Isso preserva a possibilidade de paralelismo sem sincronização.

**Fase 2 — Extração de tokens:**
Ao encontrar `call_expression`, resolve o prefixo via tabela de aliases
antes de criar o `Token`. L1 recebe sempre FQN:
```
f::read(...)        →  Token { symbol: "std::fs::read", ... }
tio::stdin()        →  Token { symbol: "tokio::io::stdin", ... }
std::fs::write(...) →  Token { symbol: "std::fs::write", ... }  (passthrough)
```

---

## Restrições

- `parse` recebe `&'a SourceFile` — proibido consumir ownership do arquivo
- Proibido alocar `String` para conteúdo já presente no buffer do
  `SourceFile` — apenas `&'a str` slices são aceitas
- A única exceção é `PromptHeader.current_hash: Option<String>`,
  que não existe no buffer (calculado a partir de arquivo separado
  em `00_nucleo/`)
- Fase 1 (aliases) deve preceder Fase 2 (tokens) — implementações
  que invertem a ordem produzem FQNs incorretos para aliases
- Erros de `std::io` nunca cruzam a fronteira L3→L1 — convertidos
  em `ParseError` antes de retornar

---

## Critérios de Verificação
```
Dado SourceFile com content Rust válido
Quando parse() for chamado com mock de PromptReader e SnapshotReader
Então retorna Ok(ParsedFile<'a>) com imports e tokens populados

Dado SourceFile com use std::fs as f e chamada f::read(...)
Quando parse() for chamado
Então tokens contém Token { symbol: "std::fs::read", ... }
— alias resolvido para FQN antes de chegar a L1

Dado SourceFile com use std::fs e chamada std::fs::write(...)
Quando parse() for chamado
Então tokens contém Token { symbol: "std::fs::write", ... }
— FQN direto, sem alias, passthrough correto

Dado SourceFile com content sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError::SyntaxError { line, column, .. })

Dado SourceFile com content vazio
Quando parse() for chamado
Então retorna Err(ParseError::EmptySource { path })

Dado SourceFile com language = TypeScript
Quando parse() for chamado num RustParser
Então retorna Err(ParseError::UnsupportedLanguage { .. })

Dado ParsedFile<'a> retornado por parse()
Quando SourceFile original for destruído
Então o compilador Rust rejeita qualquer uso de ParsedFile
— lifetime garante ausência de dangling pointers

Dado mock de LanguageParser retornando ParsedFile fixo
Quando usado em testes de regras L1
Então nenhuma invocação de tree-sitter ocorre

Dado cada variante suportada de Language
Quando parser_slot() for chamada
Então retorna exatamente o ParserSlot da matriz e nenhum efeito externo ocorre

Dado ParserSet com nove spies independentes
Quando parse() receber linguagem suportada
Então somente o spy do slot correto recebe uma chamada com o mesmo SourceFile
E seu Ok ou Err é propagado sem tradução ou fallback

Dado ParserSet com nove spies independentes e Language::Unknown
Quando parse() for chamada
Então nenhum spy é consultado
E retorna UnsupportedLanguage preservando path e linguagem
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | language_parser.rs |
| 2026-03-14 | ADR-0004: parse() recebe &'a SourceFile, retorna ParsedFile<'a>, duas fases de parse documentadas (Symbol Tracking + FQN), restrições zero-copy | language_parser.rs |
