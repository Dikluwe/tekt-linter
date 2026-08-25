# Assessment 0032 — propriedade biunívoca de prompts

**Estado:** PROTOCOLO CONGELADO — produção e gates proibidos até A
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
