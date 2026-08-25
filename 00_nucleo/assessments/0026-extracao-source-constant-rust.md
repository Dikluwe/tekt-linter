# Assessment 0026 — extração estrutural Rust de `SourceConstant`

**Estado:** PREFLIGHT SANEADO — B1/B2 autorizados; produção proibida
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
| parser Rust | `00_nucleo/prompts/parsers/rust.md` | `a661c8f226849b55fb83f3f50e5d8ea37c082852b21432c5fb24917e272c4aac` |
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
`CrystallineConfig::default()`. V21/V22 continuam apenas regressões.

B1 e B2 recusaram começar porque `SourceFile`, `PromptReader` e `PromptSnapshotReader`
eram apenas referenciados, não autorizados. Os três contratos nominais foram adicionados
com hash e passam a integrar o pacote L0 resselado. Eles não ampliam o oráculo funcional.
