# Assessment 0001 — leitura imutável de revisões Git

**Estado:** CONGELADO PARA INVESTIGAÇÃO
**Data:** 2026-08-24
**Alvo:** `03_infra/git_refinement.rs`
**Prompt histórico:** `00_nucleo/prompts/refinement-validator.md`, etapa B2

## Natureza da evidência

Este é um `assessed` retroativo, não um `sealed`. O código já existia antes desta
investigação; portanto não há alegação de independência histórica entre seu autor e os
testes existentes. Agentes independentes podem aumentar a confiança presente, mas não
reescrever a proveniência passada.

## Alegações sob teste

1. Refs são resolvidas para commits e somente os OIDs resolvidos alimentam a leitura.
2. Somente blobs regulares são interpretados; symlink, gitlink e objeto inesperado não
   viram ausência conhecida.
3. A leitura não faz checkout, fetch, execução de hook, filtro, LFS ou submódulo.
4. Working tree, índice, HEAD, refs, stash e entradas permanecem byte-identicamente
   inalterados.
5. Framing inválido, objeto ausente, estouro de orçamento e timeout não se convertem em
   `PRESERVED`.
6. Paths e refs hostis não são reinterpretados como opções, pathspec mágico ou comandos.
7. Um processo Git que excede o orçamento é encerrado sem deixar descendentes ou
   leitores bloqueados.

## Gate

Um agente adversarial produz uma matriz de ataques sem editar produção ou testes. Um
agente verificador, sem ler a produção, transforma os ataques prioritários em testes
black-box. Qualquer RED deve ser reproduzível e classificado como defeito do código,
do teste ou desta especificação antes de autorizar correção.

Se nenhum RED surgir, o resultado aumenta confiança somente nessas alegações. Se um
RED legítimo surgir, ele justifica assessments graduais nos demais adaptadores de
entrada e nos pontos capazes de silenciar `UNKNOWN`.
