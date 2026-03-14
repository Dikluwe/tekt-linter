# Prompt: Contract - Language Parser (language-parser)

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
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | language_parser.rs |
| 2026-03-14 | ADR-0004: parse() recebe &'a SourceFile, retorna ParsedFile<'a>, duas fases de parse documentadas (Symbol Tracking + FQN), restrições zero-copy | language_parser.rs |
