# Assessment 0034 — individualização semântica de prompts

**Estado:** A CONGELADO — materialização proibida até B1–B3
**Data:** 2026-08-25
**Passo:** P0106
**Baseline:** `add583d`

## Inventário confirmado

- 13 prompts compartilhados;
- 44 consumers afetados;
- 32 pares históricos já únicos;
- mínimo estrutural de 31 novos prompts;
- 44 owners distintos projetados no manifesto;
- zero Núcleos Tekt propostos nesta migração: nenhuma claim comum exigiu novo artefato;
- V15 projetada após os lotes: 13 → 6 → 2 → 0.

## Decisão semântica

Os compartilhamentos observados são agrupamentos históricos de fluxo, família ou fachada.
As relações comuns já são expressas por entidades, portas e ADRs; transformá-las em claims
`.tekt` duplicaria contratos existentes. Portanto P0106 individualiza prompts sem criar
núcleos artificiais. Isso não invalida P0105: o tipo permanece disponível quando surgir uma
claim compartilhada autônoma.

## Insumos

- manifesto: `00_nucleo/assessments/0034-manifest-individualizacao.tsv`;
- 13 classificadores: `00_nucleo/assessments/0034-groups/*.md`;
- P0106 SHA-256 `5310ff9ea92af4bc699a50a14b771fbc78fec2374b1042c84c883cfc91ac6d85`;
- inventário causal 0032-A e ADR-0022 permanecem autoridades.

## Hipóteses RED

| ID | Hipótese |
|---|---|
| R1 | manifesto pode omitir/duplicar consumer ou owner |
| R2 | prompts novos podem carregar responsabilidade de outro código |
| R3 | reescrever apenas headers pode deixar prompts amplos semanticamente falsos |
| R4 | V15 pode cair por exclusão em vez de individualização |
| R5 | reparador pode ser executado antes de V15=0 |
| R6 | resselo pode tocar bytes funcionais além de headers/metadata |

## Segregação

B1 valida o manifesto sem produção. B2 confronta classificadores e prompts propostos. B3
projeta V7/V15/V26 antes de writes. Cada lote é congelado antes do seguinte; hashes somente
no resselo global final. Projetos externos permanecem fora da superfície de escrita.
