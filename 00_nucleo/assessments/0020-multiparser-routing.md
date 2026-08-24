# Assessment 0020 — roteamento MultiParser

**Estado:** PREFLIGHT — SPEC-GAP saneado; produção ainda não confrontada
**Data:** 2026-08-24  
**Passo:** P0091  
**Baseline:** `13180b1`  
**Commit do protocolo no branch:** `b54c58c`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `3e4e0c4f80cca0d139a145a7f17dde8b8decd61ff02dfab2de91d3667610ef7e` |
| contrato parser | `00_nucleo/prompts/contracts/language-parser.md` | `ffb5ef5658e3882dc518fe71e90eb0541ed8cd5083905f3b4bbbe3edaf9c87d5` |
| enum Language | `00_nucleo/prompts/violation-types.md` | `0979ed5856022466aad8d60a37da9858c3fe06263e3295c648422ae7f3e215e9` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceFile/Language | `00_nucleo/prompts/contracts/file-provider.md` | `1574ce788513573901376fc80933464cca5e7b6bc17acf5af8bfcd28e4d7335d` |
| isolamento multilíngue | `00_nucleo/adr/0009-isolamento- de-parsers-por-linguagem.md` | `fbfeb007115f2464ece7e1f0e2a5615bb06b459e7bb7446bbd2957a06ee67452` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0091 | `00_nucleo/tekt-linter-passo-0091-auditoria-roteamento-multiparser.md` | `53d347672d69bcb3c39c297bcfb10b7e77e4c2c6851a954cba1302e5b7cffdb3` |

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

## Parecer adversarial A e saneamento

O adversário A validou todos os hashes iniciais e concluiu `SPEC-GAP / BLOCKED`, com
G1–G4 confirmados. Classificou como `GATE-DEFECT` usar a matriz do passo operacional,
expor `MultiParser`, importar parsers reais ou testar internals antes do saneamento.

O L0 foi então saneado, antes de qualquer leitura da produção:

- `violation-types.md` enumera as nove linguagens e `Unknown`;
- `language-parser.md` publica `ParserSlot`, `parser_slot` e `ParserSet` sobre nove ports
  `LanguageParser` obrigatórios, sem tipos L3;
- a matriz, o caminho direto de `Unknown`, chamada única, mesmo empréstimo, propagação
  exata, irrelevância dos demais campos e ausência de efeitos ficaram explícitos;
- `linter-core.md` remove a decisão de L4: L4 constrói o registry total e apenas inicia a
  composição L1.

Com os hashes acima resselados, G1–G4 ficam normativamente fechados. B1 e B2 podem ser
materializados cegamente; a produção continua proibida até o congelamento desses gates.

## Gate B congelado — primeiro RED

O verificador B validou os nove hashes atuais e criou
`tests/multiparser_routing_assessment.rs`, SHA-256
`91a34c57415cfa6128f5c0257e63d97b75315b669cef7f62ed431a7175c21e3a`, sem ler
produção, testes anteriores, lab, relatórios ou histórico.

- `parser_slot`, `ParserSlot` e `ParserSet` ausentes: `RED` de produção congelado.
- construção do retorno `Ok(ParsedFile)`: `SPEC-GAP` adicional, pois o schema de
  `ParsedFile` no L0 autorizado omite sete campos exigidos pelo tipo materializado.
- parsers reais não foram usados; `rustfmt --check` dirigido passou.

O gate permanece congelado. O schema L0 deve ser reconciliado e resselado antes de
ajustar apenas o fixture `Ok`; nenhuma informação dessa reconciliação pode alterar o
oráculo de roteamento já materializado.

## Papéis e precedência

- A: lê somente este Assessment e L0 hash-pinned; valida ou agrava os gaps.
- B1/B2: só começam após saneamento e resselamento.
- C: só lê produção após gates congelados.
- D: fecha causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge ou push.
