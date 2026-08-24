# Assessment 0018 — projeções V0/PARSE

**Estado:** CONGELADO PARA TRIAGEM SEGREGADA  
**Data:** 2026-08-24  
**Passo:** P0089  
**Baseline:** `cc1924b52f2079034c7814241ade88e0ca8f7583`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `ed44ffdda0a323df26a25cef40c0acb46bd692db6fdaef861a20a509adeb7029` |
| diagnósticos | `00_nucleo/prompts/violation-types.md` | `b50d90505e311a1aa99d3c80988f3f7996fe7974d71579543f86c0553a4dc314` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceError | `00_nucleo/prompts/contracts/file-provider.md` | `f00e05231f34b29256692d7b0e9f2f17db82417c1e3c3d93f922028f36fc189e` |
| fail-fast | `00_nucleo/adr/0004-reformulação-do-motor-de-análise.md` | `beea8faffd2a446ff5744bd1b5d5b6a148d86f53fc9508a31f98fd039634fcff` |
| paths owned | `00_nucleo/adr/0005-location-owned-paths-e-cargo.toml-como-artefato-gerido.md` | `917f4a1194e3d7b2a6955b6182684ad55bf909705cbf2537b095145d22b78421` |
| protocolo | `00_nucleo/tekt-linter-passo-0089-auditoria-v0-parse.md` | `f4bd91b4b438c0ee58e1e4d46968860d5c8f0fb3b1fc7167f5dccc8c7289d54f` |

## Alegações a congelar

1. `SourceError::Unreadable` projeta exatamente uma V0 `Fatal`, preservando path e
   razão, com linha/coluna zero e path owned.
2. `ParseError::SyntaxError` projeta exatamente uma PARSE `Error`, preservando path,
   linha, coluna e mensagem causal.
3. `UnsupportedLanguage` projeta uma PARSE `Warning`, preservando linguagem e path, com
   linha/coluna zero.
4. `EmptySource` projeta uma PARSE `Warning`, preservando path, com linha/coluna zero.
5. Nenhuma modalidade silencia, muda de ID ou compartilha cardinalidade com outra.
6. Unicode, strings vazias/hostis, clones e repetições permanecem determinísticos.
7. A projeção pura não acessa filesystem, config, ambiente, relógio, rede ou processo.
8. O gate observa `Cow::Owned` sem depender de parser/walker/CLI como oráculo.

## Questões arquiteturais bloqueantes

O adversário normativo deve decidir, sem consultar produção:

- camada causal da transformação erro→violação;
- natureza normativa das mensagens e severidades;
- semântica de posição zero;
- API pública black-box suficiente para o gate.

Qualquer item não decidido nos L0 autorizados é `SPEC-GAP`. O gate permanece fail-closed
até saneamento e resselamento. Não se cria API em L4 por conveniência de teste.

## Papéis segregados

- **A:** adversário L0, sem produção/testes/histórico/relatórios;
- **B:** verificador cego, após API completa, sem produção;
- **C:** confronto pelo orquestrador somente após gate congelado;
- **D:** adversário final de causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge.
