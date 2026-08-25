# Assessment 0020 — roteamento MultiParser

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-24
**Passo:** P0091
**Baseline:** `13180b1`
**Commit do protocolo no branch:** `b54c58c`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| contrato parser | `00_nucleo/prompts/contracts/language-parser.md` | `5d8a5db677dfba32be5228e643e1c1184905a0def86379aef40bab7640fa9588` |
| tipos de IR/Language | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| tipos de fatos/sentinelas | `00_nucleo/prompts/contracts/rule-traits.md` | `cdba18365badfb56288480f683451914d88b0df07201acc43ee8334d22289ba3` |
| tipos de decisão/sentinelas | `00_nucleo/prompts/rules/wildcard-saturation.md` | `19f79428f1e7c9740ae7f2466f03bc82c22a5632a2388e5b2c587a3fa2588609` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceFile/Language | `00_nucleo/prompts/contracts/file-provider.md` | `1574ce788513573901376fc80933464cca5e7b6bc17acf5af8bfcd28e4d7335d` |
| isolamento multilíngue | `00_nucleo/adr/0009-isolamento- de-parsers-por-linguagem.md` | `fbfeb007115f2464ece7e1f0e2a5615bb06b459e7bb7446bbd2957a06ee67452` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0091 | `00_nucleo/tekt-linter-passo-0091-auditoria-roteamento-multiparser.md` | `a177a2bb119d5f7d907a3921597eb67ca9e6f9d62cc799e91150e696dda85fe3` |

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

## Gate B resselado — RED isolado

Após reconciliar no L0 os sete campos omitidos de `ParsedFile`, o mesmo verificador
revalidou os nove hashes e alterou exclusivamente o fixture `Ok`. O oráculo de roteamento
permaneceu byte a byte inalterado em suas expectativas.

- gate SHA-256:
  `4e0df518f6eca5918247bec058793d01d63501a34b1a7ffe59897930a7018ffa`;
- `rustfmt --check` dirigido: PASS;
- único erro restante: imports ausentes de `parser_slot`, `ParserSlot` e `ParserSet`;
- classificação congelada: `RED` de produção, sem `SPEC-GAP` residual no gate.

O confronto C da produção está autorizado a partir deste ponto.

## Correção requerida pelo primeiro adversário D

O primeiro confronto D aprovou a produção, mas bloqueou o fechamento por dois defeitos de
evidência: B1 e B2 compartilharam a mesma identidade e o `Ok` de B2 não continha
sentinelas em todos os campos. Os dois L0 adicionais acima ficam autorizados somente para
construir valores não vazios dos tipos já referenciados por `ParsedFile`; não alteram a
matriz nem a seam de roteamento.

Um verificador novo, sem contexto herdado, deve criar B2 em arquivo separado, sem ler B1,
produção ou testes existentes. Cada campo de `ParsedFile` deve carregar sentinela
observável e o retorno deve ser comparado integralmente por `PartialEq`.

## B2 independente resselado

Um agente novo sem contexto herdado validou os onze hashes L0 e criou somente
`tests/multiparser_composition_assessment.rs`, sem ler produção, B1 ou testes existentes.

- SHA-256:
  `a8c8985933272cdd9d7ab5c99e29b77ab9f5baea1a884a25041ef1d3b4800e2b`;
- nove spies independentes e nove rotas `Ok` + nove rotas `Err`;
- sentinelas não vazias em todos os campos de `ParsedFile`, comparadas integralmente por
  `PartialEq`;
- mesma identidade de `SourceFile`, chamada única e zero chamadas em `Unknown`;
- parsers reais, filesystem e implementação produtiva não foram usados como oráculo;
- `cargo test --test multiparser_composition_assessment`: 3/3 PASS;
- `rustfmt --check` dirigido: PASS.

Os dois `GATE-DEFECTs` apontados pelo primeiro D estão fechados. O fechamento ainda exige
repetição adversarial D sobre o novo delta e a regressão global posterior.

## Fechamento

- adversário A: `SPEC-GAP / BLOCKED`; G1–G4 confirmados e depois saneados em L0;
- B1/B2 inicial SHA-256
  `4e0df518f6eca5918247bec058793d01d63501a34b1a7ffe59897930a7018ffa`:
  3/3 PASS após correção, preservado como gate de política e primeira composição;
- RED congelado: APIs `parser_slot`, `ParserSlot` e `ParserSet` ausentes e decisão em L4;
- correção causal: `74de284`, política/ports em L1 e wrapper L4 transparente;
- B2 independente SHA-256
  `a8c8985933272cdd9d7ab5c99e29b77ab9f5baea1a884a25041ef1d3b4800e2b`:
  3/3 PASS, com nove `Ok`, nove `Err` e `Unknown`;
- suíte global: 628 unitários, 83 fixtures e todos os gates de integração PASS;
- auto-lint V4/V5/V11/V12: nenhuma violação;
- hashes: `Nothing to fix`; `rustfmt` dirigido e `git diff --check`: PASS;
- adversário D repetido: `READY WITH RESIDUAL AUDIT`.

Residual: a ausência de leitura do verificador cego não é provável somente pelo autor
Git genérico; a segregação registrada e o conteúdo do B2 não mostram oráculo compartilhado
ou contaminação. O B2 corretivo ocorreu depois da materialização, por exigência explícita
do primeiro D, sem alteração posterior de produção ou B1.

## Papéis e precedência

- A: lê somente este Assessment e L0 hash-pinned; valida ou agrava os gaps.
- B1/B2: só começam após saneamento e resselamento.
- C: só lê produção após gates congelados.
- D: fecha causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge ou push.
