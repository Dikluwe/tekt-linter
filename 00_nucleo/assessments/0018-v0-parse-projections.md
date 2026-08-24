# Assessment 0018 — projeções V0/PARSE

**Estado:** READY WITH RESIDUAL AUDIT
**Data:** 2026-08-24
**Passo:** P0089
**Baseline:** `cc1924b52f2079034c7814241ade88e0ca8f7583`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `70ed01e7dd64b9da727d35b0341ee67712f0434d75543ac05697b111acee864e` |
| diagnósticos | `00_nucleo/prompts/violation-types.md` | `b50d90505e311a1aa99d3c80988f3f7996fe7974d71579543f86c0553a4dc314` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceError | `00_nucleo/prompts/contracts/file-provider.md` | `f5ed3805807f730576bd3af99d850eacfad49b9c2c1708f10aacd04c0af2e9ce` |
| fail-fast | `00_nucleo/adr/0004-reformulação-do-motor-de-análise.md` | `25d0571e0621b207b59d79ffd4ce6dfd31008738812a06fd82d0ac95d8d7fe3d` |
| paths owned | `00_nucleo/adr/0005-location-owned-paths-e-cargo.toml-como-artefato-gerido.md` | `917f4a1194e3d7b2a6955b6182684ad55bf909705cbf2537b095145d22b78421` |
| protocolo | `00_nucleo/tekt-linter-passo-0089-auditoria-v0-parse.md` | `b0379c2bf5c0bca13d83ba3b5dbfea2dc1bb4c27030ff2c01e5d4805a7126ae3` |
| projetor puro | `00_nucleo/prompts/rules/infrastructure-error.md` | `bb2f8f5669ca264b205d03240a06d53c576b57847450388f44663f4f89cab119` |

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

## SPEC-GAPs congelados e decisão causal

O adversário A encontrou contradição entre “conversores em L4” e “L4 zero lógica de
negócio”, além de ausência de API pública black-box. Ambos foram fechados antes do gate:
projeção erro→violação é política pura L1, sob
`prompts/rules/infrastructure-error.md`; L4 somente encaminha. `0:0` significa posição
indisponível apenas nas modalidades explicitamente normatizadas.

Após materialização, o reparador oficial atualizou somente `Hash do Código` em
`linter-core.md` e `file-provider.md`; os hashes acima resselam essa metadata sem mudar
as expectativas usadas pelo gate cego.

## Papéis segregados

- **A:** adversário L0, sem produção/testes/histórico/relatórios;
- **B:** verificador cego, após API completa, sem produção;
- **C:** confronto pelo orquestrador somente após gate congelado;
- **D:** adversário final de causalidade, gravidade, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge.

## Fechamento

- SPEC-GAP camada/API: fechado por projetor puro L1 e encaminhamento L4.
- Gate cego: 7/7 PASS; SHA-256
  `7cf459d6251533df903019795b036ff43345731f14d9b9eb636e6773b5bbb7db`.
- RED API ausente e RED `SourceError` sem derives: fechados em `f1f8486`.
- Gate final de whitespace: fechado em `ec43f05`.
- Suíte workspace, fixtures, hashes e auto-lint arquitetural: PASS.
- Adversário D: `READY WITH RESIDUAL AUDIT`.
