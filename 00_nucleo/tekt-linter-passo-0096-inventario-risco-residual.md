# Passo operacional 0096 — inventário segregado de risco residual

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-residual-risk-inventory`
> **Pré-condição:** P0095 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0095

## Objetivo

Construir um inventário verificável dos componentes ainda não auditados ou cobertos
apenas parcialmente e classificá-los por risco antes de escolher qualquer novo lote
funcional. P0096 é somente leitura sobre produção: não cria gate executável, não altera
Rust, configuração, prompt normativo, ADR, fixture ou comportamento do linter.

O resultado deve responder quais seams permanecem, por que não são mais consideradas de
baixo risco, quais autoridades e consumidores cada uma atravessa e qual delas pode ser
proposta como P0097. A seleção final é documental e não autoriza sua execução.

## Hipótese e risco

Após os Assessments 0001–0024, componentes aparentemente pequenos podem esconder
acoplamento entre parser, filesystem, Git, configuração, apresentação e coordenação. O
risco deste passo é produzir uma lista por nome de arquivo, confundir quantidade de linhas
com risco ou declarar cobertura por associação sem rastrear alegação, gate e consumidor.

O inventário deve preferir seams comportamentais. Arquivo já tocado não equivale a seam
auditada; teste existente não equivale a gate independente; relatório histórico não
equivale a fechamento vigente.

## Insumos L0 hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| fechamento inicial | `00_nucleo/assessments/0013-fechamento-pre-merge.md` | `3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9` |
| último Assessment | `00_nucleo/assessments/0024-refinement-snapshot-loader.md` | `6e32d9d8b798928c438bca3959b5de35bbd899a4dfed646860e82ffd4f51bb3c` |
| fechamento P0094 | `00_nucleo/relatorio-p0094-auditoria-n16-summary.md` | `5631381eabae71e090f663d5ce00093a2f524bddcd9eb4bd744afd08699da4b6` |
| fechamento P0095 | `00_nucleo/relatorio-p0095-auditoria-loader-snapshots-refinamento.md` | `33c7ae9cf9608daf1f99e289c1af8eb9a2be78e80515e7acf8d5709033106b01` |
| método de insumo cego | `00_nucleo/relatorio-p0081-fechamento-insumo-normativo-cego.md` | `25177c76c5a3005daccac54dea4c085b04a7a58aed268b50fbbfc8763273c33d` |

Na execução, o Assessment 0025 deve acrescentar hashes dos relatórios 0072–0093 e dos
Assessments 0001–0023 que realmente sustentarem uma classificação. Não copiar hashes sem
recalculá-los. Divergência de qualquer insumo é `RED` documental e interrompe a triagem
até resselamento.

## Universo e unidade de análise

O universo inicial inclui produção em `01_core`, `02_shell`, `03_infra` e `04_forge`,
seus comandos em `main.rs`/CLI, configuração, fixtures e consumidores externos presentes
no workspace. Diretórios gerados, dependências vendorizadas e artefatos de build ficam
fora.

Cada linha do inventário representa uma **seam comportamental**, com:

1. identificador estável e responsabilidade observável;
2. arquivos produtores e consumidores;
3. camada proprietária e travessias L1/L2/L3/L4;
4. autoridade L0 exata ou `SPEC-GAP` explícito;
5. Assessment/relatório anterior que cobre a seam, se houver;
6. gates existentes e aquilo que eles não demonstram;
7. superfícies externas: filesystem, ambiente, processo, Git, parser ou serialização;
8. efeitos possíveis: leitura, escrita, mutação, execução, diagnóstico e exit status;
9. raio de regressão e dependências compartilhadas;
10. risco, confiança da classificação e próximo tratamento recomendado.

## Taxonomia de cobertura

- `CLOSED`: Assessment fechado, gates independentes e consumidor confrontado;
- `CLOSED_WITH_RESIDUAL`: fechado, mas com resíduos nomeados ainda relevantes;
- `PARTIAL`: apenas parte da seam, variante, linguagem ou consumidor foi confrontada;
- `HISTORICAL_ONLY`: há teste/relatório anterior, sem fechamento segregado suficiente;
- `UNAUDITED`: nenhuma evidência específica localizada;
- `INDETERMINATE`: fronteira ou autoridade não pode ser decidida sem novo L0.

Uma seam `CLOSED` só pode ser reaberta por evidência concreta: novo consumidor, mudança
posterior ao fechamento, hash divergente, residual que entrou no escopo ou contradição
entre autoridade e produção. A ausência de leitura exaustiva não autoriza reabrir tudo.

## Classificação de risco

Pontuar cada dimensão de 0 a 3 com evidência:

| Dimensão | 0 | 1 | 2 | 3 |
|---|---|---|---|---|
| camadas | uma | fronteira simples | três camadas | L1–L4/política global |
| efeitos | puro | leitura confinada | escrita/mutação | processo/Git/efeito externo |
| entrada | tipada | schema fechado | parser/config/paths | bytes hostis/ambiente/revisões |
| consumidores | isolado | um direto | múltiplos | CLI e consumidores cruzados |
| L0 | total | residual pequeno | gaps materiais | contraditório/ausente |
| gates | independentes | boa cobertura parcial | históricos/acoplados | ausentes |
| regressão | local | módulo | subsistema | workspace/compatibilidade |

Somar apenas depois de justificar cada dimensão:

- 0–5: baixo;
- 6–10: médio;
- 11–15: alto;
- 16–21: crítico.

Um único fator pode elevar o risco mínimo: escrita destrutiva, execução de processo,
dependência de ambiente hostil, mutação Git, autoridade contraditória ou consumidor que
decide exit global impede classificação `baixo`. Incerteza não reduz pontuação; vira
`INDETERMINATE` ou aumenta a faixa com confiança baixa.

## Protocolo segregado

### A — mapa de cobertura histórica

A lê somente o Assessment 0025 e os insumos resselados. Produz a matriz de Assessments
0001–0024 para alegações/seams, separando fechado, residual, parcial e não coberto. A não
lê produção e não recomenda P0097.

### B1 — inventário estrutural de produção

B1 lê produção e testes, mas não os pareceres de A. Enumera seams, produtores,
consumidores, camadas, efeitos externos e gates existentes. Não classifica conformidade a
partir do comportamento atual e não altera arquivos.

### B2 — inventário normativo independente

B2 lê prompts/ADRs e índices de passos, sem ler o inventário B1. Mapeia autoridades,
áreas com L0 suficiente, contradições e `SPEC-GAP`. Não inspeciona implementação para
transformá-la em autoridade.

### C — reconciliação e ranking

C recebe somente os três artefatos congelados de A/B1/B2. Une linhas por seam, registra
discordâncias, calcula a matriz de risco e identifica no máximo três candidatos para
P0097. Nenhuma discordância pode ser resolvida por média silenciosa.

### D — adversário final

D confronta o inventário consolidado com hashes, produção e consumidores. Procura seams
omitidas, cobertura superestimada, componentes reabertos sem causa, pontuação manipulada,
travessia arquitetural incorreta e candidato escolhido por conveniência. D não edita o
inventário.

## Regras de seleção do P0097

O relatório pode recomendar um candidato somente se:

1. a seam estiver `PARTIAL`, `HISTORICAL_ONLY`, `UNAUDITED` ou `INDETERMINATE`;
2. houver fronteira delimitável e consumidor identificável;
3. pré-condições normativas forem enumeráveis;
4. gates cegos puderem ser separados da produção;
5. o lote não misturar saneamento L0, implementação e integração externa sem pontos de
   parada;
6. a escolha minimizar risco entre candidatos igualmente valiosos, sem alegar baixo
   risco quando a pontuação indicar médio ou maior.

Se nenhum candidato satisfizer os critérios, o resultado correto é `BLOCKED` com uma
etapa de saneamento L0 proposta, não uma auditoria funcional artificial.

## Classificações e fechamento

- `RED`: hash divergente, cobertura alegada sem evidência ou seam relevante omitida;
- `SPEC-GAP`: autoridade insuficiente para definir ou separar a seam;
- `GATE-DEFECT`: teste histórico/acoplado foi tratado como gate independente;
- `PASS`: classificação reproduzível a partir das evidências congeladas.

P0096 fecha somente como:

- `READY WITH RESIDUAL AUDIT`: inventário reproduzível, discordâncias registradas e
  candidato P0097 justificável; ou
- `BLOCKED`: universo incompleto, hashes inválidos, arquitetura indeterminada ou nenhuma
  próxima seam delimitável.

## Validação mínima

1. baseline e todos os insumos efetivamente usados hash-pinned;
2. cobertura explícita de `01_core`, `02_shell`, `03_infra`, `04_forge` e CLI;
3. busca reversa produtor→consumidor e consumidor→produtor;
4. reconciliação de cada Assessment 0001–0024 com pelo menos uma seam ou justificativa de
   ausência;
5. matriz completa com pontuação, confiança e evidência por dimensão;
6. amostra adversarial de ao menos um item de cada classe de cobertura;
7. confirmação de que nenhum arquivo fora de `00_nucleo` mudou;
8. `cargo test --workspace --quiet` somente como baseline de regressão, nunca como prova
   de cobertura;
9. auto-lint V5/V6/V7/V12 e reparador de hashes em dry-run;
10. `git diff --check`, adversário D e worktree limpo no fechamento.

## Saídas esperadas

- `00_nucleo/assessments/0025-inventario-risco-residual.md`;
- três artefatos segregados A/B1/B2, incorporados ou anexados com hashes;
- matriz seam→L0→camadas→consumidores→gates→risco;
- ranking de no máximo três candidatos;
- recomendação única para P0097 ou bloqueio justificado;
- `00_nucleo/relatorio-p0096-inventario-risco-residual.md`;
- veredito final.

P0096 não autoriza alteração de produção, criação de gate executável, correção funcional,
merge, push, instalação ou release. Sua execução deve parar se P0095 ainda não estiver
integrado em `master`.
