# Relatório P0099 — saneamento da vigência de `refine-revisions`

**Data:** 2026-08-25
**Branch:** `codex/reconcile-refine-revisions-authority`
**Baseline:** `6fe51ce80e953e17b990cc400fb487b119aab034`
**Resultado:** `READY WITH RESIDUAL AUDIT` / `PASS-CONFIRMED`

## Decisão

`refine-revisions` está confirmado como capacidade normativa vigente, somente dentro do
envelope B2 aprovado em 2026-08-24: Git local, OIDs imutáveis, leitura de blobs regulares,
sem shell, rede/fetch, protocolos externos, checkout, mutação, build, hooks, filtros,
LFS, travessia de symlink/submódulo ou temporários diagnósticos.

A aprovação humana explícita da adenda B2 é posterior ao Gate A/B1 que proibia Git e não
foi revogada. O Gate antigo permanece preservado como histórico, não como limite vigente
da adenda.

## Segregação

- A reconstruiu a cronologia normativa sem ler produção ou documentação pública;
- B analisou README/USAGE sem ler produção, prompt, ADR ou A;
- C recebeu A/B hash-pinned, decidiu `CONFIRMED` e congelou patch nominal;
- executor separado aplicou somente o saneamento autorizado;
- D1 bloqueou omissões remanescentes; D2 aprovou o resselamento final.

## REDs e correções

O executor detectou que alterar `refinement-validator.md` provocaria drift V5 em cinco
consumidores. Como o prompt já confirmava B2, as clarificações provisórias foram
descartadas e seu hash permaneceu `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97`.

D1 encontrou:

- `linter-core.md` ainda apresentava Git como globalmente fora de escopo;
- `00_nucleo/README.md` repetia essa contradição antes de registrar B2;
- README/USAGE omitiam build, protocolos externos e temporários diagnósticos.

Os textos públicos e o índice foram corrigidos. Como `linter-core.md` é causal para
produção, sua correção exigiu autorização adicional do usuário. O reparador oficial
alterou exatamente dez headers `@prompt-hash`, de `10f02f10` para `1b09018a`; a inspeção
do diff confirmou ausência de qualquer mudança comportamental.

## Arquitetura Tekt

- L1 conserva fatos, contratos e comparação puros, sem tipos Git;
- L2 conserva argumentos e apresentação; exits globais permanecem em F09;
- L3 é a fronteira do adapter Git e contenção do subprocesso;
- L4 resolve OIDs e compõe extração/comparação.

P0099 apenas reconciliou autoridade e linhagem. Ele não confrontou a implementação Git,
não decidiu precedência CLI e não auditou composição do pipeline.

## Efeito no backlog

- F04: `CLOSED` como `PASS-CONFIRMED`;
- F05: desbloqueado para auditoria funcional Git/B2;
- F09: inclui `refine-revisions` na futura matriz de exits/precedência;
- F08: recebe o comando como vigente, mantendo as demais dependências.

Resíduos aceitos: conformidade funcional Git/B2 pertence a F05; códigos e precedência
globais pertencem a F09; documentos históricos podem preservar estados anteriores quando
identificados como históricos e não causais.

## Validação

- 630 testes unitários e todas as integrações: PASS;
- V5/V6/V7/V12: nenhuma violação;
- reparador V5 dry-run: `Nothing to fix`;
- `git diff --check`: PASS;
- D2: `READY WITH RESIDUAL AUDIT`;
- nenhum push realizado.

O merge do P0099 não faz parte deste passo.
