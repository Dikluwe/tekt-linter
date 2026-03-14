# Prompt: ParseError (parse-error)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2025-03-13
**Arquivos gerados**: 01_core/contracts/parse_error.rs

---

## Contexto

Quando L3 tenta parsear um arquivo com tree-sitter e falha — gramática
não reconhece o conteúdo, arquivo vazio, encoding inválido — esse erro
precisa ter uma representação em L1 para que `LanguageParser` possa
retornar `Result<ParsedFile, ParseError>`.

`ParseError` é um erro de domínio do linter, não um erro de sistema.
`std::io::Error` nunca atravessa a fronteira L3→L1.

## Instrução
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// tree-sitter reconheceu o arquivo mas encontrou nó ERROR na árvore
    SyntaxError {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },
    /// Linguagem do arquivo não tem grammar registrada
    UnsupportedLanguage {
        path: PathBuf,
        language: Language,
    },
    /// Conteúdo vazio — nada a parsear
    EmptySource {
        path: PathBuf,
    },
}
```

## Restrições

- L1: sem dependências externas
- `ParseError` é `Clone + PartialEq` — testável sem mocks
- Não carrega `std::io::Error` — L3 converte antes de retornar

## Critérios de Verificação
```
Dado ParseError::SyntaxError com line=5 column=3
Quando comparado com outro idêntico
Então PartialEq retorna true

Dado ParseError::UnsupportedLanguage { language: Language::Unknown }
Quando formatado com Debug
Então mensagem é legível sem panic
```

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | parse_error.rs |
