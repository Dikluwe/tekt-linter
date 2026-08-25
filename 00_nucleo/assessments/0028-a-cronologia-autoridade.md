# Assessment 0028/A — cronologia e autoridade de `refine-revisions`

**Papel:** A — cronologia e autoridade segregadas  
**Resultado:** evidência suficiente para `CONFIRMED`  
**Produção, testes, README, USAGE e parecer B lidos:** não

## Identidade dos insumos

| Unidade | SHA-256 esperado | SHA-256 recalculado | Resultado |
|---|---|---|---|
| protocolo P0099 | `611b6403daf81d0bd4437478c9a112bd921a685351291653c1ef7dd04a353cae` | `611b6403daf81d0bd4437478c9a112bd921a685351291653c1ef7dd04a353cae` | `PASS` |
| contrato de refinamento | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` | `PASS` |
| ADR-0019 | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` | `PASS` |
| Assessment P0098 | `dbfd5755641962132cafc967951ba1ae8bc8197370152af426a01eab7e1389f7` | `dbfd5755641962132cafc967951ba1ae8bc8197370152af426a01eab7e1389f7` | `PASS` |
| backlog P0098 | `c829befc0df2addb431406d3592c88499a2c47d70d0178ed3d25bef7369b1314` | `c829befc0df2addb431406d3592c88499a2c47d70d0178ed3d25bef7369b1314` | `PASS` |
| fechamento P0098 | `9e8d3acd399088c18bbfabf232cb307863fe28f03a3472e5f8f59dfc05c76a6d` | `9e8d3acd399088c18bbfabf232cb307863fe28f03a3472e5f8f59dfc05c76a6d` | `PASS` |

O Assessment 0028 foi lido como protocolo corrente. Os registros P0098 foram lidos
somente porque P0099 os identifica nominalmente como origem causal de F04. Nenhuma
implementação ou superfície pública participou da classificação.

## Linha do tempo normativa

| Data | Natureza | Evidência autorizada | Efeito sobre Git/B2 |
|---|---|---|---|
| 2026-08-23 | proposta | ADR-0019 propõe a capacidade de refinamento e divide a entrega em A e uma Etapa B posterior. | Não autoriza Git. |
| 2026-08-23 | aprovação | Cabeçalho, interface e Gate do ADR-0019 registram aprovação humana da Etapa A. | Git, wrapper e leitura de revisões permanecem expressamente não autorizados nesse escopo. |
| 2026-08-24 | aprovação incremental | A seção “Etapa B1 aprovada” do ADR-0019 e o histórico do prompt registram autorização humana para snapshot de diretório explícito, sem Git. | Não autoriza Git; preserva a proibição anterior para B1. |
| 2026-08-24 | proposta B2 | A adenda do ADR-0019 descreve backend, ameaça, efeitos, budgets, camadas, portabilidade e condição para aprovação. | A proposta isolada ainda não bastaria para autorizar Git. |
| 2026-08-24 | aprovação B2 | O estado da própria adenda é `ACEITA — aprovada pelo humano ...`; ela afirma que a materialização está autorizada em branch dedicado. O prompt registra B2 como `Vigente`, inclui a interface aprovada e repete a aprovação humana no histórico. | Autoriza `refine-revisions` somente no envelope local, imutável e limitado da adenda. |
| 2026-08-25 | reconciliação | P0098 classifica a divergência documental como F04 `L0-BLOCKED` e exige que P0099 confirme ou revogue a capacidade antes de F05/F09/F08. | Não revoga B2; suspende a auditoria funcional até o saneamento da contradição documental. |

## Separação de estados de autoridade

### Propostas

- A capacidade geral de refinamento e a Etapa B futura foram inicialmente propostas em
  2026-08-23.
- O conteúdo técnico da adenda B2 nasceu como proposta condicionada à aprovação.
- Orçamentos, requisito externo e escolhas de backend permanecem limitados ao que a
  adenda efetivamente aprovou; esta análise não amplia esse envelope.

### Aprovações

- Etapa A: aprovação humana explícita em 2026-08-23.
- Etapa B1: aprovação humana explícita em 2026-08-24, sem Git.
- Etapa B2: aprovação humana explícita em 2026-08-24, posterior ao Gate antigo. A adenda
  se autodeclara aceita e materializável; o prompt vigente confirma a mesma decisão de
  forma independente e datada.

### Revogações

Não há, no corpus autorizado, decisão humana explícita que revogue B2. O cabeçalho, o
escopo e o Gate anteriores do ADR-0019 são evidência de um limite previamente vigente,
mas antecedem a adenda aceita e não declaram revogá-la. P0098 também não revoga: trata
a contradição como bloqueio documental a sanear.

## Confronto da contradição

O ADR-0019 conserva simultaneamente dois estratos:

1. o escopo A/B1 e o Gate anterior, nos quais Git continua não autorizado;
2. a adenda B2 posterior, explicitamente aceita, que autoriza uma exceção estreita para
   leitura Git local e imutável.

Lidos sem cronologia, os estratos são contraditórios e constituem o `SPEC-GAP` já
congelado por F04. Lidos segundo a ordem de autoridade exigida por P0099, a adenda B2 é
a seção normativa mais recente, contém aprovação humana inequívoca e amplia o escopo
anterior de maneira delimitada. O prompt vigente confirma esse mesmo resultado. A
inconsistência que resta é de consolidação documental, não falta de autoridade para a
decisão binária.

## Classificação

**Evidência suficiente para `CONFIRMED`.**

Fundamento causal:

1. existe aprovação humana explícita e datada de B2;
2. essa aprovação é posterior ao Gate que proibia Git;
3. a adenda declara materialização autorizada e define um envelope estreito;
4. o prompt vigente registra B2 e seu subcomando como aprovados;
5. não existe revogação no corpus autorizado;
6. P0098 apenas exige reconciliação e não altera a decisão normativa.

Esta classificação confirma somente a vigência normativa de `refine-revisions` no
envelope B2. Ela não prova implementação, conformidade da superfície pública, política
global de exits nem atendimento dos gates funcionais futuros. Também não recomenda
redação para o saneamento.
