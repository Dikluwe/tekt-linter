# Assessment 0024 — loader de snapshots e contratos de refinamento

**Estado:** PREFLIGHT — produção proibida
**Data:** 2026-08-25
**Passo:** P0095
**Baseline:** `4408649`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| ADR refinamento | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` |
| contrato refinamento | `00_nucleo/prompts/refinement-validator.md` | `a3a1eb935f5c79e698e0b4a792f36ec70f67c53c9db65c345b27e347c2bcba7d` |
| arquitetura | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| diagnósticos | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Git fechado | `00_nucleo/assessments/0001-git-refinement.md` | `5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17` |
| fechamento anterior | `00_nucleo/assessments/0013-fechamento-pre-merge.md` | `3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9` |
| Etapa A histórica | `00_nucleo/tekt-linter-passo-validacao-de-refinamento.md` | `0364668e0adfa53d01f6d17cab9a7298839e9935c4286b4a412f04df559db9bd` |
| protocolo P0095 | `00_nucleo/tekt-linter-passo-0095-auditoria-loader-snapshots-refinamento.md` | `0fc61c8b295c1c0d585c6a8c385dbd6a371d14a17b5b7961e246ffad67555e14` |

## Alegações candidatas

1. Snapshot aceita somente schema/versionamento publicado e estados fechados.
2. Metadata/chaves/valores preservam identidade sem trim coercivo.
3. Campos e chaves duplicados/desconhecidos falham fechados.
4. Contrato aceita somente três relações e seus campos condicionais.
5. Listas/relações preservam ordem e multiplicidade autorizadas.
6. UTF-8, JSON/TOML e I/O permanecem classes de erro distinguíveis.
7. O loader é read-only e não decide veredito.
8. L1 tipa, L3 carrega/valida, L2 apresenta e L4 coordena.

## SPEC-GAPs candidatos

### G1 — fechamento do schema

As autoridades publicam exemplos e tipos, mas não decidem uniformemente campos JSON/TOML
desconhecidos, duplicados ou proibidos por variante.

### G2 — identidade textual

Não está decidido se ids, versões, chaves, source/target e `Known(value)` são apenas
validados com `trim` ou também normalizados; vazio/whitespace em chave/valor é ambíguo.

### G3 — listas e duplicatas semânticas

Ordem/multiplicidade de relações e `accepted_targets`, vazios, duplicatas e relações
contraditórias não possuem política total.

### G4 — limites

Não há orçamento publicado para bytes, profundidade, observáveis, relações, strings ou
listas. Um loader explícito ainda pode exaurir memória antes de produzir `UNKNOWN`.

### G5 — filesystem explícito

Os paths são argumentos do usuário, mas symlink, FIFO/diretório, troca concorrente e
arquivo especial não têm classificação. Confinamento de raiz pode ser inaplicável, mas
isso também deve ser dito.

### G6 — erros observáveis

Não está claro se mensagens completas são contrato ou somente classes `I/O`, `UTF-8`,
`JSON/TOML`, versão e schema; path/source hostil deve permanecer identificável sem
execução ou perda.

### G7 — API nominal

Os nomes históricos existem, mas o L0 não publica todas as assinaturas e caminhos
públicos necessários para B1/B2 cegos.

## Protocolo ativo

- A lê somente este Assessment e os dez insumos hash-pinned;
- B1/B2 começam apenas após saneamento e resselamento;
- C confronta produção somente após ambos os gates congelados;
- D fecha causalidade, consumidor, arquitetura e regressões 0001–0023.

Resultados: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge/push.
