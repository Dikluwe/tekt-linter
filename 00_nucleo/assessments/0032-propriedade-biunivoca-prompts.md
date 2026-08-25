# Assessment 0032 — propriedade biunívoca de prompts

**Estado:** BLOCKED — C verde; D requer decisão semântica para 13 L0 compartilhados
**Data:** 2026-08-25
**Passo:** P0104
**Baseline funcional:** `84fa3006ad6557722cfbe4d10c78c7d0de6b4195`
**Envelope operacional:** `4901345260915a0f7628c79ca85ed096cf9d0d00`

## Decisão normativa humana

`@prompt` representa propriedade exclusiva. Cada código produtivo possui exatamente um
prompt proprietário e cada prompt proprietário possui exatamente um código. Relação
compartilhada, herança e metadata plural não são autorizadas por P0104.

## Insumos conferidos

| Unidade | SHA-256 |
|---|---|
| P0104 | `30b4db584b6ca73ee518471f25b8ff25207c0212bd48dd21884d56dff22be0e4` |
| contrato V15 | `81ba0f080eac8c2db78f27f04f206ff746eecdd358fdb55b146523192704f053` |
| produção V15 | `123f2ab3da2c130ae47d624b731327ec857d8b752399c1b744a8d48d7d86a400` |
| contrato fix-hashes | `d6cc361ed70301c002717b6e80a6c166a0ba1f149084c0f3000c373ba5d1daf9` |
| produção fix-hashes | `26252ef696b1026168568e992a10b41a25b9848bd94e1ab0fa01403288ea3278` |
| arquitetura Tekt | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| índice atual | `9bf8d5e772761347c52f628d9a0cde57d1a4dbd931dcb5e66968e6558e62aa91` |
| wiring atual | `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` |
| diagnóstico P1179 | `3fee15dbe9c3610a2104f4523cac79039c368737a8f7cb8aafd9c5adc5d95e60` |
| manifesto P1179 | `67f3ec296c9e8bf54891b1c1a32cd323d8509c6957767a3f87e0135622315a6a` |

Todos os hashes conferem e o worktree estava limpo. O único delta do baseline é o passo
P0104. Divergência futura bloqueia; não há resselamento automático.

## Hipóteses RED

| ID | Hipótese |
|---|---|
| R1 | V15 atual não observa dois códigos apontando ao mesmo prompt |
| R2 | resultado global pode depender de ordem/partição se agregado sem índice canônico |
| R3 | fix-hashes planeja por consumer e sobrescreve metadata de prompt compartilhado |
| R4 | fix-hashes escreve source antes de descobrir metadata ausente, produzindo parcial |
| R5 | segunda passagem/V5 direta declara falso fechamento sem validar o vínculo reverso |
| R6 | o próprio tekt-linter contém ownership compartilhado e ficará vermelho sob a regra correta |

## Segregação

A é somente leitura. B1/B2/B3 recebem L0 hash-pinned e interfaces mínimas, sem ler
produção correspondente. B4 lê apenas o diagnóstico/manifesto externo e estado Git do
Typst Crystalline; não escreve. C só abre após gates e REDs congelados. D saneia o
próprio linter depois da regra, sem exceções. E é somente leitura.

O Typst Crystalline permanece fora da superfície de escrita. Sem merge, push, release ou
instalação neste passo.

## A — inventário integral

O inventário somente leitura está congelado em
`00_nucleo/assessments/0032-a-inventario-ownership-prompts.md` (SHA-256
`fde7ac62afc69a6aaa14dbb0aaf334d1441e0d19ea9bd51482ec36187a909e66`, commit
`81d2cf1`). Ele encontrou 76 consumers produtivos, 45 prompts distintos, 32 relações
1:1 e 13 prompts compartilhados por 44 consumers. A individualização mínima exigirá 31
novos prompts proprietários.

Não há multi-`@prompt` local nem órfão após as exceções vigentes. A assimetria dos
parsers foi congelada: todos publicam `prompt_header`, mas somente Rust preserva todas
as referências; Go, Java e Elixir preservam a canônica; os demais deixam `prompt_refs`
vazio. A seam autorizada preserva `(prompt, source_path)` no índice local, reduz em L4 e
decide puramente em L1.

Os 13 compartilhamentos constituem `SPEC-GAP` apenas para a individualização semântica
do próprio repositório. Eles não bloqueiam implementar nem testar a regra global; bloqueiam
declarar o auto-lint `READY` sem D.

## B — gates congelados antes de produção

| Gate | Evidência | Resultado inicial |
|---|---|---|
| B1 bijeção in-memory | `tests/prompt_ownership_bijection_assessment.rs` — `d0c9850fda426cf7210dc5617f3f48e4f554f0145823deb21f076632312d9eaf` | compile-RED causal: seam L1 global ausente |
| B2 wiring real | `tests/prompt_ownership_wiring_assessment.rs` — `26003301e74b978a9d3efe75d84d47316e95a652987f7a230c72d99242fde021` | RED funcional esperado sob o binário atual |
| B2 fixture prompt | `tests/fixtures/prompt_ownership_wiring/00_nucleo/prompts/shared.md` — `ac750ae97b9ea93cd20db34e234c92531d9b7663d5e961d79eeee8141ffd13b8` | entrada exclusiva do gate |
| B2 fixture config | `tests/fixtures/prompt_ownership_wiring/crystalline.toml` — `31a381d682231e9f8a22cfe3d292d20b68c3a879ed57ac7ea4866c54fca45618` | correção do oráculo de exclusão |
| B3 transação | `tests/fix_hashes_bijection_assessment.rs` — `7d7ed0691656517c91a846d7f442d9d506e2e3f1b3dcd968a008ef15d32ac3d2` | compile-RED causal: seam transacional ausente |

B1 cobre oito propriedades normativas. B2 executa o binário real, incluindo todos os
parsers declarados. B3 cobre preflight integral, colisões determinísticas, rollback,
falha de rollback, validação reversa e reprodução mínima do falso fechamento P1179.
`rustfmt --check` dirigido e `git diff --check` passaram antes de C.

### B4 — Typst Crystalline somente leitura

O manifesto foi revalidado sem alteração externa: 421 consumers, 336 prompts, 22 prompts
compartilhados por 107 consumers, 314 prompts únicos e 78 consumers sem metadata. O
dry-run atual emitiu 421 linhas, SHA-256
`518825da010e5124649a6287eaaf39f5f6d8c0b30c7cbfcf0d4dc8d762581c78`.
O hash do status Git antes/depois permaneceu
`1c3ff45d79b2eb82706490f155ed919b9d3e759f73fb843d09adb20642750548` e o hash do
diff binário dos seis paths rastreados permaneceu
`102831d5574f5d3580797670a67742875a63517701e4f984f601eb31a92854be`.

## Classificação causal antes de C

- `R1`: RED de produção confirmado pelos gates B1/B2.
- `R2`: risco congelado; B1 exige bytes invariantes por permutação.
- `R3`–`R5`: RED contratual/estrutural confirmado por B3 e P1179.
- `R6`: confirmado pelo inventário A; saneamento D permanece `SPEC-GAP` semântico.

Nenhuma expectativa dos gates pode ser alterada durante C.

## C — implementação

Commit `d0fb12a` implementou:

- entidade L1 owned e função pura `check_prompt_ownership`;
- diagnóstico V15 global determinístico após a redução L4, usando a referência canônica
  publicada por todos os parsers;
- bloqueio incondicional de `--fix-hashes` diante de ownership não biunívoco;
- seam L2 `TransactionalHashRewriter`, plano integral, preflight, rollback e validação
  bidirecional;
- adapter L4/L3 que prepara bytes antes da aplicação, restaura o par que falhou e restaura
  os pares anteriores;
- isolamento explícito das fixtures históricas que não são oráculo de ownership; B2 é o
  gate real multi-parser dessa propriedade.

B1 passou 8/8, B2 passou 7/7 e B3 passou 8/8. A suíte integral passou com 630 testes
unitários, 83 fixtures e todos os integration gates. `git diff --check` passou. O warning
preexistente de `print_tree` permanece.

Durante B2 houve uma corrida no congelamento: o agente fortaleceu o gate e a fixture
enquanto o commit `1e7ef49` era criado, além de formatar arquivos fora do escopo. O conteúdo
efetivamente congelado no commit foi revisado e seus hashes reais são os acima; ele não leu
a produção V15. A formatação fora do escopo foi descartada integralmente. A única correção
posterior do oráculo foi adicionar o `crystalline.toml` normativo para a prova de exclusão
configurada. A ocorrência fica como ressalva de proveniência, não escondida como execução
segregada perfeita.

## D — bloqueio reproduzível

O auto-lint V15 agora encontra exatamente os 13 prompts compartilhados previstos por A,
por 44 consumers. O reparador dry-run aborta com exit 2 antes de apresentar/aplicar writes:
`--fix-hashes blocked by 13 V15 ownership collision(s)`. O hash do estado Git antes e
depois permaneceu `b70311b08af10d2460939647d622b766a33f8ac2bcc1fd81f86d119a40bac107`.

Logo, R6 foi confirmado sem exceção escondida. O fluxo oficial também não pode resselar o
novo contrato V15 enquanto os compartilhamentos permanecerem, pois o bloqueio integral é
deliberado. Individualizar os 31 prompts necessários sem inventar contratos é a decisão
semântica que P0104 proíbe automatizar. Classificação: `SPEC-GAP`; fechamento: `BLOCKED`.
