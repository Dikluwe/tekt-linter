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
