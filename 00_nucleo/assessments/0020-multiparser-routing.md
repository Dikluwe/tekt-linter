# Assessment 0020 — roteamento MultiParser

**Estado:** PREFLIGHT — SPEC-GAP congelado; produção ainda não confrontada  
**Data:** 2026-08-24  
**Passo:** P0091  
**Baseline:** `13180b1`  
**Commit do protocolo no branch:** `b54c58c`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `2e5da1cfb9d1f66e5015cf67bf2f6fb9e8992ad29c59d05d5acef4ea2705f8d3` |
| contrato parser | `00_nucleo/prompts/contracts/language-parser.md` | `203ed423a5149331525a6c7bc1662e74b87e6bcab7a5ee5337cda7581523791b` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceFile/Language | `00_nucleo/prompts/contracts/file-provider.md` | `1574ce788513573901376fc80933464cca5e7b6bc17acf5af8bfcd28e4d7335d` |
| isolamento multilíngue | `00_nucleo/adr/0009-isolamento- de-parsers-por-linguagem.md` | `fbfeb007115f2464ece7e1f0e2a5615bb06b459e7bb7446bbd2957a06ee67452` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0091 | `00_nucleo/tekt-linter-passo-0091-auditoria-roteamento-multiparser.md` | `1c9bec8b72e4fd91c0c16f6c09fd5ede55c64f47ee58ce9c285a2d25cc1a0825` |

## Alegações candidatas

1. Cada linguagem suportada seleciona exatamente um slot nominalmente correto.
2. `Language::Unknown` não chama parser e produz `UnsupportedLanguage` com path e
   linguagem preservados.
3. Somente `file.language` participa da escolha.
4. O parser selecionado recebe por empréstimo o mesmo `SourceFile`.
5. `Ok` e `Err` retornam sem tradução, fallback ou segunda tentativa.
6. Exatamente uma chamada ocorre e os demais slots permanecem intocados.
7. A decisão é determinística e independente da ordem/disponibilidade dos adapters.
8. A seam não acessa filesystem, configuração, ambiente, relógio, rede ou processo.

## Evidência normativa presente

- `linter-core.md` atribui a L4 a seleção do parser por `file.language` e declara L4 sem
  lógica de negócio.
- O exemplo executável de `linter-core.md` decide somente Rust, TypeScript e Python;
  qualquer outra variante cai em `UnsupportedLanguage`.
- ADR-0009 registra `MultiParser` como composição implementada, mas sua decisão formal e
  sua tabela de implementação cobrem TypeScript/Python sobre a base Rust.
- `LanguageParser` fixa empréstimo `&'a SourceFile` e resultado
  `Result<ParsedFile<'a>, ParseError>`.
- `ParseError` fixa a forma de `UnsupportedLanguage`.

## SPEC-GAPs congelados

### G1 — universo de linguagens divergente

O passo candidato supõe Rust, TypeScript, Python, C, C++, Zig, Go, Java e Elixir. Os L0
autorizados para o roteamento publicam apenas Rust, TypeScript, Python e `Unknown`.
Referências a linguagens adicionais em prompts de regras não autorizam por si mesmas o
destino de composição. A matriz de nove slots não pode ser usada como oráculo ainda.

### G2 — ownership causal da política

`linter-core.md` simultaneamente diz que L4 seleciona por linguagem e que L4 contém zero
lógica de negócio. Não existe tipo público fechado nem função pura L1 para representar a
decisão `Language -> slot`. O gate B1 não possui API black-box autorizada.

### G3 — seam de composição

O contrato não define composição genérica/injetável com nove spies, precedência quando um
adapter está indisponível, nem mecanismo público para provar chamada única e propagação
exata sem expor o wiring privado. O gate B2 não pode inventar essa API.

### G4 — irrelevância e ausência de efeitos

O L0 atribui a escolha a `file.language`, mas não explicita nominalmente a irrelevância de
path, content, layer e `has_adjacent_test`, nem proíbe I/O na própria política separada. A
restrição geral de L1 puro só se torna aplicável depois de decidir que a política pertence
a L1.

## Decisão de preflight

Não construir gate nem ler `04_wiring/main.rs` até que G1–G4 sejam saneados e os hashes
resselados. A decisão recomendada, sujeita ao adversário A, é:

- publicar em L1 um enum fechado `ParserSlot` e função pura `parser_slot(Language)`;
- manter parsers concretos em L3 e `MultiParser` privado em L4;
- publicar no L0 a matriz completa e a semântica de `Unknown`;
- definir uma seam de encaminhamento testável por spies sem tipos L3 em L1;
- declarar dependência exclusiva de `language`, chamada única, propagação exata e zero
  efeitos na política.

## Papéis e precedência

- A: lê somente este Assessment e L0 hash-pinned; valida ou agrava os gaps.
- B1/B2: só começam após saneamento e resselamento.
- C: só lê produção após gates congelados.
- D: fecha causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge ou push.
