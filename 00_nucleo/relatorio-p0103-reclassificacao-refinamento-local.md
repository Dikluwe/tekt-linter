# Relatório P0103 — reclassificação do refinamento local

**Data:** 2026-08-25  
**Decisão:** opção C / ADR-0021  
**Resultado:** `READY WITH RESIDUAL AUDIT` para o modo local

## Decisão

O linter continua focado em melhoria de código e preservação da arquitetura Tekt.
`refine-revisions` é uma conveniência defensiva que confia no Git instalado, no usuário
local e na estabilidade do repositório durante a operação.

Certificação contra executável adulterado, mutação sincronizada e fuga deliberada de
processos não pertence ao produto atual. Um modo selado poderá ser criado futuramente,
como projeto/comando separado, se surgir caso de uso que justifique sandbox, budgets e
matriz de plataforma próprios.

## Avanços integrados

- seam Git L3 única consumida pela rota produtiva;
- resolução única de refs e uso posterior somente de OIDs;
- ambiente mínimo, sem hooks, filtros, prompts, lazy fetch ou configuração global;
- rejeição de alternates e symlinks persistentes;
- framing e caps incrementais;
- budget de blob/revisão antes de publicação;
- timeout e cleanup da contenção normal;
- projeção única `GitRevisionContent` → `ArtifactFacts`;
- regressões históricas e CLI preservadas.

## Evidência futura preservada

Os gates P0102 de corrida transitória, `setsid` e Job Object permanecem no repositório
como testes `ignored`, com motivo explícito. Eles não são apresentados como PASS e não
fazem parte da suíte normal. SG-1, SG-2 e seus `GATE-DEFECTs` permanecem documentados.

## Fechamento

R1 e R3 são `PASS`. R2/R4/R5 foram reclassificados, por decisão humana de produto, para
o futuro modo selado. F05 fica `CLOSED` para o modelo local e `OPEN/FUTURE` para
certificação hostil.

Integração em `master` é autorizada após suíte completa, auto-lint, hashes, smoke test do
binário instalado e confirmação de worktree limpo.
