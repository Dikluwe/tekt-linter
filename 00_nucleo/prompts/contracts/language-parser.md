# Prompt: LanguageParser (language-parser)

**Camada**: L1 (Core — Contracts)
**Criado em**: 2025-03-13
**Arquivos gerados**: 01_core/contracts/language_parser.rs

---

## Contexto

L1 precisa receber `ParsedFile` para julgar violações, mas não pode
invocar tree-sitter — isso é I/O de gramática, responsabilidade de L3.
Este contrato define a fronteira: L3 implementa, L1 consome.

## Instrução

Declarar a trait `LanguageParser` usando `ParseError` do contrato irmão.
```rust
pub trait LanguageParser {
    fn parse(&self, file: SourceFile) -> Result<ParsedFile, ParseError>;
}
```

`SourceFile` vem de `crate::contracts::file_provider`.
`ParsedFile` vem de `crate::entities::parsed_file`.
`ParseError` vem de `crate::contracts::parse_error`.

A trait recebe `SourceFile` inteiro — não apenas `&str` — porque
o parser precisa do `path` para determinar a `Layer` e a `Language`
antes de invocar a gramática correta.

## Restrições

- L1: zero tree-sitter, zero I/O
- Erros de gramática viram `ParseError` — erros de disco nunca chegam aqui
- L3 decide qual gramática usar baseado em `SourceFile.language`

## Critérios de Verificação
```
Dado um SourceFile com content Rust válido
Quando parse() for chamado numa implementação mock
Então retorna Ok(ParsedFile) com imports e tokens populados

Dado um SourceFile com content sintaticamente inválido
Quando parse() for chamado
Então retorna Err(ParseError) com localização do erro
```

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | language_parser.rs |
