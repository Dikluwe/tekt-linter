# Assessment 0028 — vigência de `refine-revisions`

**Estado:** PROTOCOLO CONGELADO — A/B autorizados
**Data:** 2026-08-25
**Passo:** P0099 / F04
**Baseline:** `6fe51ce80e953e17b990cc400fb487b119aab034`

## Insumos hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| protocolo P0099 | `00_nucleo/tekt-linter-passo-0099-saneamento-vigencia-refine-revisions.md` | `611b6403daf81d0bd4437478c9a112bd921a685351291653c1ef7dd04a353cae` |
| contrato de refinamento | `00_nucleo/prompts/refinement-validator.md` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` |
| ADR de refinamento | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` |
| README público | `README.md` | `3ff67521214cff672b54941e1d4392b2ab933c51ed69ecc9cf5e55e8989716d6` |
| guia público | `USAGE.md` | `245bc38db11a29467e7e72514f488fcb69fd471d401fa7eb1b6823355fa8d4f1` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment P0098 | `00_nucleo/assessments/0027-horizonte-finito-auditoria.md` | `dbfd5755641962132cafc967951ba1ae8bc8197370152af426a01eab7e1389f7` |
| backlog P0098 | `00_nucleo/assessments/0027-c-backlog-finito.md` | `c829befc0df2addb431406d3592c88499a2c47d70d0178ed3d25bef7369b1314` |
| fechamento P0098 | `00_nucleo/relatorio-p0098-reconciliacao-horizonte-auditoria.md` | `9e8d3acd399088c18bbfabf232cb307863fe28f03a3472e5f8f59dfc05c76a6d` |

## Contradição e decisão exigida

O prompt registra B2 como aprovada/vigente. O ADR contém simultaneamente escopo/gate que
proíbem Git e adenda posterior declarada aceita. README diferencia apenas `refine` como
sem Git; README/USAGE não publicam claramente `refine-revisions` e seus efeitos.

O resultado deve ser exatamente `CONFIRMED` ou `REVOKED`. A decisão deriva de autoridade
humana explícita, cronologia normativa e coerência arquitetural, nunca de produção,
testes ou help gerado.

## Segregação

- A lê este Assessment, prompt, ADR e registros históricos L0 autorizados; não lê
  produção, README/USAGE nem parecer B.
- B lê este Assessment, README, USAGE e documentação de CLI autorizada; não lê produção,
  prompt/ADR causal além da contradição aqui congelada nem parecer A.
- C só começa após A/B hash-pinned e propõe decisão/patch nominal.
- executor separado aplica apenas o saneamento documental aprovado.
- D confronta a redação final, hashes, arquitetura e ausência de expansão.

## Limites

Podem mudar somente `00_nucleo`, `README.md` e `USAGE.md`. Produção, testes executáveis,
fixtures, configuração e dependências são proibidos. Política global de precedência e
exits continua em F09. P0099 não executa F05, F08 ou F09.

## Saídas segregadas

- `00_nucleo/assessments/0028-a-cronologia-autoridade.md`;
- `00_nucleo/assessments/0028-b-superficie-publica.md`;
- `00_nucleo/assessments/0028-c-decisao-saneamento.md`;
- `00_nucleo/relatorio-p0099-saneamento-vigencia-refine-revisions.md`.

Classificações: `RED`, `SPEC-GAP`, `GATE-DEFECT`, `PASS-CONFIRMED` e `PASS-REVOKED`.
Fechamento: `READY WITH RESIDUAL AUDIT` ou `BLOCKED`. Sem merge ou push.
