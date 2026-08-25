# Assessment 0026 — extração estrutural Rust de `SourceConstant`

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-25
**Passo:** P0097
**Baseline:** `3a5ffbec3968230f8fda29dff329c476fa73be39`

## Insumos L0 hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| protocolo P0097 | `00_nucleo/tekt-linter-passo-0097-auditoria-extracao-source-constant-rust.md` | `3aa2bee69002528787837583a2c97f279793dc1fa1f682e9050da7b4dc29a921` |
| contrato V21 | `00_nucleo/prompts/unsourced-constant.md` | `9560ecbcdc3a5f5eec14e0cabe96062b274504f92e1f009188c6dbc2f59fa174` |
| traits/IR | `00_nucleo/prompts/contracts/rule-traits.md` | `cdba18365badfb56288480f683451914d88b0df07201acc43ee8334d22289ba3` |
| source file | `00_nucleo/prompts/contracts/file-provider.md` | `1574ce788513573901376fc80933464cca5e7b6bc17acf5af8bfcd28e4d7335d` |
| prompt reader | `00_nucleo/prompts/contracts/prompt-reader.md` | `5ded333b4ef0da943355962da5de202f7a5b8a4aa6d885236f215e3a3884f219` |
| snapshot reader | `00_nucleo/prompts/contracts/prompt-snapshot-reader.md` | `80b6f7ab9fbb0f97fa085d7a34802792eb6fce4834ac204775b47749c77985be` |
| crate registry | `00_nucleo/prompts/crate-registry.md` | `2eae38e14e797f21b7f217403f6a421c9eb2ceedf871936df2c4630499d06116` |
| parser Rust | `00_nucleo/prompts/parsers/rust.md` | `80d1bb090717719befe293aba04b3ff22496f15caa5db1820827843c2fea796d` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment V21 | `00_nucleo/assessments/0017-hardcoded-contextual-value-v21.md` | `fb3024c255789d409b73d2d8e5e138c753c8a01e9c986190ff727147081e584b` |
| fechamento V21 | `00_nucleo/relatorio-p0088-triagem-v21.md` | `2b5d0c09078dddb9d7dcab43b14e6585c40ea89253cdf020d438e88435524f40` |
| Assessment de risco | `00_nucleo/assessments/0025-inventario-risco-residual.md` | `4d9a7fa75def17dfcd5f5e552210b825d8b64ea98e64f8e9fdd430eb0fc74e2a` |
| reconciliação | `00_nucleo/assessments/0025-c-reconciliacao-risco.md` | `f713ec185c8c4e878da8c5cc609846271a6a1af3bd3a58e69b45e6667b1c7ede` |
| fechamento P0096 | `00_nucleo/relatorio-p0096-inventario-risco-residual.md` | `b653185723e46790ac32098cc8781787c8220247114d39301453be9c42750037` |

## Fronteira congelada

Entrada: bytes de fonte Rust fornecidos à API pública de `RustParser`. Saída observável:
`ParsedFile.constants: Vec<SourceConstant>`. Campos autorizados: kind, snippet,
linha/coluna, origem de teste, return type, scaling, context var, geometric sink e
data-table, além de ordem/multiplicidade.

`citation` é opaco e fora do oráculo. V21 e V22 são consumidores de regressão, nunca
oráculos. Filesystem, config, wiring, apresentação e exit estão fora.

## SPEC-GAPs candidatos para A

1. taxonomia completa de literals/constantes e significado exato de snippet;
2. coluna byte versus caractere diante de Unicode;
3. contextos que tornam `is_test_origin` verdadeiro;
4. associação de `function_return_type`, especialmente closures e funções complexas;
5. estrutura exata de scaling, nesting e ordem dos operandos;
6. gramática/identidade/desempate de `context_var` e `geometric_sink`;
7. definição e limiar de `is_in_data_table`;
8. ordem, multiplicidade e fonte sintaticamente inválida;
9. API pública mínima disponível aos gates;
10. separação efetiva entre campos estruturais e citação compartilhada.

## Protocolo

- A lê somente este Assessment e os doze insumos; classifica G1–G10.
- Qualquer saneamento altera apenas L0 estrutural e exige resselamento.
- B1/B2 começam somente após A e usam arquivos/fixtures independentes.
- Produção permanece proibida até ambos os gates congelados e RED inicial registrado.
- C corrige somente a seam autorizada; D fecha causalidade, consumidores e arquitetura.

Resultados: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge/push.

## Parecer A e saneamento

A validou os doze hashes iniciais. G1–G9 eram `SPEC-GAP`; G10 foi `PASS` apenas como separação de
escopo. O L0 foi reduzido e saneado para literais numéricos dentro de `function_item`,
kind positivo/negativo, snippet byte-exato, linha/coluna 1-based em bytes UTF-8, preorder,
multiplicidade e erro sintático sem IR parcial.

Origem teste, return type, scaling, context var, geometric sink, data-table e citation
foram removidos do oráculo. B1 cobre casos positivos/identidade; B2 cobre exclusões e erro.
Ambos usam somente `RustParser::new` + `LanguageParser::parse`, mocks locais dos readers e
`CrystallineConfig::default()` e `CrateRegistry::default()`. V21/V22 continuam apenas
regressões.

B1 e B2 recusaram começar porque `SourceFile`, `PromptReader` e `PromptSnapshotReader`
eram apenas referenciados, não autorizados. Os três contratos nominais foram adicionados
com hash e passam a integrar o pacote L0 resselado. Eles não ampliam o oráculo funcional.

B1 então revelou que o construtor vigente possui quarto argumento `CrateRegistry`, omitido
no contrato do parser. `crate-registry.md` foi adicionado ao pacote e o construtor L0 foi
corrigido para `CrateRegistry::default()`. O RED de compilação provisório não conta como
RED funcional; B1/B2 devem revalidar seus arquivos após este resselamento.

## Gates congelados e RED inicial

| Gate | Arquivo | SHA-256 | RED |
|---|---|---|---|
| B1 — identidade | `tests/rust_source_constant_identity_assessment.rs` | `cc596d876bcfbcacbf3688bd9a1aa1b875928bcb53f77abe1544af5100fd3dcb` | RED histórico 0/3; GREEN 3/3 |
| B2 — exclusões | `tests/rust_source_constant_context_assessment.rs` | `dc50ebb0b3913a108c09a0b5e2dc81d6918ac622d7ab8a4159340a1c818237dd` | RED histórico 1/3; GREEN 3/3 |

B1 observou colunas zero-based e emissão de strings/não numéricos. B2 confirmou colunas
zero-based, emissão de contextos excluídos e `SyntaxError` sem IR parcial já conforme.
O harness provisório B2 usava `Box::leak`; foi classificado `GATE-DEFECT` e removido antes
do congelamento. Os gates finais não importam V21/V22, não observam campos excluídos e
mantêm `SourceFile` vivo lexicalmente.

## GATE-DEFECT de projeção no confronto C

A primeira formulação exigia coleção inteira vazia para formas não numéricas. Ao ficar
verde, ela removeu kinds históricos e quebrou regressão direta do parser, invadindo
citações/V22 fora do lote. O L0 foi corrigido: B1/B2 devem filtrar somente
`FunctionNumberLiteral` e `NegativeLiteral`; outros kinds coexistem opacos. Os hashes dos
gates acima ficam invalidados até nova identidade e novo RED.

Os gates foram resselados com projeção numérica. O RED histórico continua causal para as
mesmas observações autorizadas: coluna zero-based e numeral dentro de macro. A correção C
passou a usar coluna 1-based e não desce em `macro_invocation`, preservando os demais kinds
históricos. B1/B2 finais estão 3/3 cada.

## Fechamento D

O adversário final confirmou os hashes L0 e dos dois gates, a causalidade RED→GREEN e a
separação arquitetural. A mudança de produção ficou restrita a
`03_infra/rs_parser.rs`, em L3: a projeção numérica usa coluna 1-based, limita-se a
`function_item`, exclui patterns e macros e preserva o snippet byte-exato, a ordem e a
multiplicidade. Tipos L1, consumidores, wiring, configuração, apresentação e semântica de
citações não foram alterados.

Os dois `GATE-DEFECT` foram fechados antes do parecer:

1. o harness B2 deixou de usar `Box::leak` e mantém `SourceFile` vivo lexicalmente;
2. B1/B2 passaram a observar somente a projeção `FunctionNumberLiteral` /
   `NegativeLiteral`, sem exigir a remoção de kinds históricos opacos.

Validação final:

- B1: 3/3 e B2: 3/3 nos hashes congelados acima;
- regressões V21: 9/9; V22 e teste dirigido do parser Rust: PASS;
- suíte do workspace: 630 unitários e todas as integrações/fixtures: PASS;
- V5/V6/V7/V12: nenhuma violação; reparador V5 dry-run: `Nothing to fix`;
- `git diff --check`: PASS;
- gates sem imports/chamadas a V21/V22 e sem asserts de `citation`.

Resíduos aceitos: variantes adicionais da gramática de macros, especialmente tokens
negativos dentro de macros, não integram a matriz explícita; associação de citações e os
demais campos estruturais removidos no preflight continuam fora de P0097. A evidência RED
intermediária permanece registrada na história Git e neste Assessment, sem artefato
executável separado.

**Veredito:** `READY WITH RESIDUAL AUDIT`. Nenhum merge ou push é autorizado por este
fechamento.
