# Assessment 0015 — revisão adversarial segregada V23–V25

**Produtor:** `adversary/v23-v25/0015`
**Resultado inicial:** `SPEC-GAP`

Os três hashes L0 do assessment 0015 foram validados antes da análise. Nenhum L1–L4,
teste, histórico ou relatório foi consultado nesta rodada.

## SPEC-GAPs congelados

| ID | Regra | Lacuna normativa |
|---|---|---|
| SG1 | comum | API black-box, tipos/construtores, assinaturas e diagnósticos não publicados |
| SG2 | V23 | formas neutras não enumeradas; zero não pode ser inferido globalmente |
| SG3 | V23 | resolvedor neutro sem fluxo a sink e cardinalidade fan-out/fan-in ambíguos |
| SG4 | V23/V24 | chave completa de identidade entre contrato, language, scope, source e sink/slot ausente |
| SG5 | V24 | ausência comprovada, fato ausente, `None` explícito e opacidade não distinguíveis |
| SG6 | V24 | dependência direta/transitiva/condicional/morta/opaca sem fronteira executável |
| SG7 | V25 | implementação direta é proibida, mas não tem modalidade nem critério estrutural |
| SG8 | V25 | “owner efetivo”, duplicata idempotente e owners distintos não definidos |
| SG9 | V25 | inclusividade/ordem de `resolved_after` e composição com proxy não definidas |
| SG10 | comum | conjunto versus ocorrência, duplicatas, permutação e particionamento sem unidade normativa |
| SG11 | comum | location decisiva e evidência quando há múltiplas operações não definidas |
| SG12 | comum | templates mínimos de mensagem e campos obrigatórios não executáveis |

## Ataques congelados

- produto categoria × regra e contratos com mesmo id em language/scope diferentes;
- V23: argumento neutro/não neutro, absolute-only, fluxo direto/let, sem sink, contrato
  cruzado, fan-out/fan-in, mutação/alias/macro/interprocedural;
- V24: preserve/drop-to-default, dependência direta/transitiva/condicional/morta,
  ausência/opacidade, slot vizinho e contrato cruzado;
- V25: três modalidades nomeadas, implementação direta, owners iguais/distintos,
  proxy sozinho/owner sozinho/composição, canonicalizador antes/no/depois do marco;
- permutação, duplicação, multiplicidade por location, campos irrelevantes, vazio, Unicode
  e limites numéricos.

Nenhum RED de produção é alegado nesta fase. Converter esses casos em asserts antes do
saneamento seria GATE-DEFECT. Os L0 devem publicar semântica e API completas, ser
resselados por hash e entregues a um verificador novo.

## Fechamento final

O saneamento separou os fatos decididos por L3 do classificador L1 e publicou a API no L0
causal de `rule_traits.rs` antes da materialização. A quarta modalidade V25 recebeu kind
próprio, sem colapsar reimplementação direta em proxy ou canonicalizer.

O gate final SHA-256
`9d7bbda9cd97f164785e7e8f1dea406a4d9190148396452afea36839029dd1e6` passou 5/5. Ele
cobre a matriz completa sete kinds × três regras, quatro níveis, ordem/multiplicidade,
language irrelevante, Unicode/vazio/extremos e evidência integral por ocorrência. Após
um GATE-DEFECT intermediário ser fechado, o adversário declarou **NÃO REABRIR** para L1.

Os SG2–SG11 não foram apagados: foram corretamente roteados para o futuro assessment de
extração L3/integração. Não são RED residual destes três classificadores puros.
