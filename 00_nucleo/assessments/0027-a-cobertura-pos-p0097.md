# Assessment 0027/A — cobertura histórica pós-P0097

**Papel:** A, somente Assessment 0027 e nove insumos L0 hash-pinned  
**Resultado:** PASS  
**Produção/testes lidos:** não  
**Recomendação de próximo passo:** não realizada por segregação

## Validação dos insumos

Os nove SHA-256 recalculados coincidem byte a byte com o pacote congelado no Assessment
0027.

| Unidade | SHA-256 validado |
|---|---|
| protocolo P0098 | `d4dd1b52d181cb1f092e339669a9b8e2990c2c4300658785679c01a59063e4ce` |
| arquitetura Tekt | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| inventário P0096 | `4d9a7fa75def17dfcd5f5e552210b825d8b64ea98e64f8e9fdd430eb0fc74e2a` |
| reconciliação P0096 | `f713ec185c8c4e878da8c5cc609846271a6a1af3bd3a58e69b45e6667b1c7ede` |
| fechamento P0096 | `b653185723e46790ac32098cc8781787c8220247114d39301453be9c42750037` |
| Assessment P0097 | `26d94721dc5a0e6787f407859c27bf15e26e35b47b34611dd788e0cb3d4f30da` |
| fechamento P0097 | `fc3d130fb3794e47fbe7ed387c7d0160305faf098952fe0366c033d90b181057` |

## Reconciliação histórica S1–S6

| Seam | Estado em P0096 | Delta documental comprovado por P0097 | Cobertura histórica pós-P0097 |
|---|---|---|---|
| S1 — extractor/escritor de snapshot | `PARTIAL` | Nenhum; loader fechado em P0095 não abrangia extractor/escritor, e P0097 excluiu filesystem e escrita. | `PARTIAL` |
| S2 — refinamento Git/subprocesso | `INDETERMINATE` | Nenhum; P0097 não leu Git, não executou subprocessos e não resolveu a contradição L0 registrada. | `INDETERMINATE` |
| S3 — manifesto, recibo e selo | `CLOSED_WITH_RESIDUAL` | Nenhuma causa concreta de reabertura. O fechamento histórico 16/16 permanece; P0097 não tocou essa seam. | `CLOSED_WITH_RESIDUAL` |
| S4 — pipeline principal | `PARTIAL` | Nenhum; wiring, configuração, apresentação e exit foram excluídos de P0097. | `PARTIAL` |
| S5 — parsers concretos por linguagem/característica | `PARTIAL` | Fechamento estreito somente da projeção numérica Rust descrita abaixo. Nenhum fechamento do parser Rust inteiro, de outros parsers ou de todos os campos de `SourceConstant`. | `PARTIAL`, com subprojeção P0097 `CLOSED` |
| S6 — preflight/precedência CLI ampliada | `PARTIAL` | Nenhum; CLI, precedência e exit foram excluídos de P0097. | `PARTIAL` |

Esta reprodução não converte automaticamente os rótulos históricos de P0096 na taxonomia
de destino de P0098. Em especial, `PARTIAL`, `INDETERMINATE` e
`CLOSED_WITH_RESIDUAL` são estados de cobertura herdados; a decisão entre `MANDATORY`,
`L0-BLOCKED`, `ACCEPTED-RESIDUAL`, `CLOSED` ou `REOPENED` pertence à reconciliação C.

## Recorte exato fechado por P0097 em S5

P0097 fechou somente a transformação fonte Rust → `ParsedFile.constants` para a
**projeção numérica** dentro de `function_item`, observada pelos dois kinds
`FunctionNumberLiteral` e `NegativeLiteral`:

- positivos são `FunctionNumberLiteral` e negativos unários são `NegativeLiteral`;
- o snippet é byte-exato, inclui sinal e sufixo;
- linha e coluna são 1-based, com coluna em bytes UTF-8;
- ordem lexical/preorder e multiplicidade são preservadas, sem deduplicação;
- numerais em patterns, ranges, macros e fora de função são excluídos;
- erro sintático não produz IR parcial;
- outros kinds podem coexistir, mas permanecem opacos;
- V21 e V22 foram confrontados apenas como regressões, nunca como oráculos.

O fechamento possui dois gates independentes resselados, ambos verdes em 3/3, com RED
causal preservado e dois `GATE-DEFECT` corrigidos antes do veredito final. A alteração foi
restrita à extração L3; tipos e decisões L1, apresentação L2 e coordenação L4 não foram
alterados para acomodar os gates.

## Fronteira que P0097 não fechou

Continuam fora do fechamento, sem herança por associação:

- origem de teste, `function_return_type`, scaling, `context_var`, `geometric_sink` e
  data-table;
- associação, janela e semântica de citações compartilhadas, além da agregação V22;
- todos os kinds não numéricos de `SourceConstant`;
- variantes de gramática de macro não enumeradas, especialmente tokens negativos;
- `DecisionExpr` e wiring V16;
- demais características do parser Rust e todos os outros parsers concretos;
- frescura filesystem, configuração global, pipeline, apresentação, precedência CLI e
  exit status.

Os campos estruturais removidos no preflight e a associação de citações permanecem
resíduos expressamente nomeados; o corpus A não fornece causa para tratá-los como
fechados. Também não fornece causa concreta para reabrir S3.

## Conclusão do papel A

`PASS`: o histórico pós-P0097 é reproduzível sem leitura de produção ou testes. Das seis
seams, somente S5 recebeu delta de cobertura, e esse delta é estritamente a projeção
numérica Rust acima. S1, S2, S4 e S6 conservam os estados históricos; S3 conserva seu
fechamento com residual; S5 permanece parcial no agregado. A não pontua risco, não aplica
o destino final e não escolhe P0099.
