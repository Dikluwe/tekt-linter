# Passo operacional 0085 — triagem segregada V17–V20

> **Natureza:** envelope operacional temporário; não é regra arquitetural  
> **Estado:** contrato congelado; triagem em execução  
> **Branch:** `codex/audit-low-risk-v17-v20`  
> **Base:** `master@cea1e70`  
> **Assessment:** 0014

## Objetivo

Continuar a auditoria residual em lote pequeno e de baixo risco, cobrindo os quatro
classificadores mecânicos V17–V20 sem reabrir os assessments 0001–0013.

## Protocolo segregado

1. O orquestrador congela assessment, baseline e L0 autorizado por SHA-256.
2. Um adversário, sem editar arquivos, deriva ataques apenas do pacote normativo.
3. Um verificador diferente materializa gate black-box sem ler produção ou testes
   existentes até o gate estar congelado em commit próprio.
4. Só depois o orquestrador confronta gate, produção e relatório adversarial.
5. Achados usam exclusivamente `RED`, `SPEC-GAP` ou `GATE-DEFECT`; toda correção exige
   achado previamente congelado.
6. Um adversário final revisa o delta e o fechamento antes de qualquer recomendação de
   merge.

Identidade nominal dos produtores registra segregação operacional, não prova sandbox.

## Insumo normativo do gate

- `00_nucleo/assessments/0014-decision-metrics-v17-v20.md`;
- `00_nucleo/prompts/rules/wildcard-saturation.md`, SHA-256
  `5941adf0c444a65e101224dacfdb1fea0cbafebf46a5a9ac6be5bed25063cc08`.

Qualquer outro L0 necessário deve ser autorizado por adendo congelado antes do gate.

## Saída e fechamento

O lote deve produzir gate de integração independente, relatório adversarial, matriz de
resultados e atualização do assessment 0014. O estado final será `BLOCKED` ou
`READY WITH RESIDUAL AUDIT`; não haverá merge, instalação nem release neste passo.
