# Relatório P0098 — reconciliação do horizonte da auditoria

**Data:** 2026-08-25
**Branch:** `codex/reconcile-audit-exit-criteria`
**Baseline:** `7e358cff39ba24d5bba26de2fa0a3ba86ff7b379`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Resultado

A campanha agora possui horizonte finito: **13 lotes máximos**, dos quais cinco saneiam
L0 e oito confrontam comportamento obrigatório. Um lote pode ser eliminado por decisão
normativa explícita, mas nenhum lote adicional entra sem gatilho formal de reabertura.

| Destino | Quantidade | Lotes |
|---|---:|---|
| `L0-BLOCKED` | 5 | F01, F02, F04, F09, F11 |
| `MANDATORY` | 8 | F03, F05, F06, F07, F08, F10, F12, F13 |
| `CLOSED` sem lote | 1 seam | S3 — manifesto, recibo e selo |
| sub-seam `REOPENED` | 1 | Rust `NegativeLiteral` sob macro, absorvida por F12 |

## Reconciliação S1–S6

- S1 permanece `L0-BLOCKED`: writer e schema do extractor precisam decisões antes do
  gate funcional.
- S2 permanece `L0-BLOCKED`: prompt, ADR e documentação pública divergem sobre a vigência
  de Git/`refine-revisions`.
- S3 permanece `CLOSED`; não houve causa concreta de reabertura.
- S4 é `MANDATORY`, decomposto por comando e saída observável.
- S5 é `MANDATORY`, decomposto por linguagem, consumidor e classe de fato.
- S6 permanece `L0-BLOCKED` até existir matriz normativa única de precedência e exits.

P0097 fechou somente as células numéricas Rust efetivamente confrontadas. Nenhum outro
campo, parser, consumidor ou camada herdou fechamento por associação.

## RED adversarial

D1 encontrou uma omissão material: o ramo negativo do parser Rust não suprime
`macro_invocation` como o ramo positivo, portanto `emit!(-5)` pode produzir
`NegativeLiteral`. Isso contradiz a exclusão publicada por P0097 e não poderia permanecer
como residual.

O backlog foi corrigido sem ampliar sua contagem: a célula foi reaberta e absorvida por
F12. Também foram acrescentados risco/confiança por lote, F13 à dependência pertinente de
F06, fronteiras distintas para F03/L3 e F08/L4, condicionalidade F08→F05 e matrizes
nominais hash-pinned para F12/F13. D2 aprovou o resselamento. D3 confirmou a redação
final e o hash C `c829befc0df2addb431406d3592c88499a2c47d70d0178ed3d25bef7369b1314`, sem novo
`RED`, `SPEC-GAP` ou `GATE-DEFECT`.

P0098 não corrigiu o parser: sua natureza é documental. O RED agora está preservado e
endereçado por um lote obrigatório futuro, em vez de oculto como auditoria residual.

## Condição de saída

A campanha termina quando F01–F13 estiverem `CLOSED` ou eliminados por decisão L0
hash-pinned, não restarem `SPEC-GAP` sobre comportamento público vigente e cada promessa
mantida possuir cadeia proporcional entrada→IR→decisão→diagnóstico/efeito→exit. Gates,
regressões, hashes, auto-lint e fechamento adversarial devem estar verdes.

Após essa condição, auditoria nova só começa por mudança de contrato, produtor,
consumidor ou dependência; novo requisito público; incidente ou entrada hostil
reproduzível; `GATE-DEFECT`; ou invalidação de hash/evidência. Possibilidade abstrata de
casos adicionais não reabre o backlog.

## Próximo candidato

P0099 deve executar somente F04: sanear, sem produção, a vigência, autorização e promessa
pública de Git/`refine-revisions`. A decisão desbloqueia F05, F09 e a composição F08.
Nenhum segundo candidato foi selecionado.

## Segregação e validação

- A: cobertura histórica pós-P0097, sem produção;
- B1: delta estrutural, sem autoridade derivada da implementação;
- B2: promessas e `SPEC-GAP`, sem produção;
- C: reconciliação somente dos três pareceres hash-pinned;
- D1: `BLOCKED`; D2: `READY WITH RESIDUAL AUDIT`;
- nenhum arquivo fora de `00_nucleo` foi alterado por P0098;
- arquitetura Tekt L1/L2/L3/L4 preservada.

Nenhum push foi realizado. O merge do P0098 não faz parte deste passo.
