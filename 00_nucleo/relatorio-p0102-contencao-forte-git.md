# Relatório P0102 — contenção forte do Git

**Data:** 2026-08-25  
**Branch:** `codex/audit-git-refinement-functional`  
**HEAD confrontado:** `f2ebbfb`  
**Resultado:** `BLOCKED`  
**Backlog:** F05 permanece aberto

## Resultado executivo

P0102 executou feasibility e materializou os estímulos que faltavam, mas não reabriu
produção. A análise encontrou duas decisões arquiteturais inevitáveis ainda ausentes:

- SG-1: mecanismo e política Unix para conter descendentes depois de `setsid`;
- SG-2: mecanismo resistente a TOCTOU para a visão do object database.

Esses `SPEC-GAPs` envolvem portabilidade, privilégio, cleanup e/ou budget. Escolher uma
implementação em C teria transformado detalhe operacional em regra arquitetural sem L0.

## Evidência congelada

| Gate | Resultado |
|---|---|
| B1 corrida transitória | 3 controles PASS; 3 REDs publicando sentinela externa |
| B2 escape de sessão | 0/4; três watchdogs e um `ProcessFailure` indevido |
| B3 Windows v2 | 0 testes Linux; compile-RED Windows por seam ausente |
| B4 regressão | rota 3/3, protocolo 7/7, stream 4/4 |

B1 prova R4 mesmo quando o symlink é restaurado antes do pós-check. B2 prova que
`setsid` escapa da contenção vigente. B3 não é prova Windows e permanece bloqueado pela
infraestrutura.

## GATE-DEFECT

- B2: o watchdog não governa cleanup + `worker.join()` e a cadeia não publica todos os
  PIDs. O RED é válido, mas a cobertura integral de lifecycle ainda não está pronta.
- B3: `compile_error!` fixa a seam ausente, porém não implementa os seis cenários runtime
  exigidos. Deve ser substituído depois da seam e executado em Windows real.

## Matriz final

| ID | Resultado |
|---|---|
| R1 rota única | `PASS` |
| R2 Job Object | `RED / BLOCKED` |
| R3 overflow incremental | `PASS` Unix |
| R4 object database TOCTOU | `RED` |
| R5 lifecycle integral | `RED` Unix / `BLOCKED` Windows |

## Arquitetura e regressão

Não houve alteração em produção, manifests ou dependências. As fronteiras Tekt foram
preservadas. Passaram 630 testes de biblioteca, Git histórico 6/6, gate de objetos P0101
7/7, timeout 4/4, CLI 10/10, V5/V6/V7/V12 e reparador V5 dry-run. A suíte workspace
completa fica intencionalmente vermelha pelos gates P0102 congelados.

## Fechamento

P0102 termina `BLOCKED`, sem merge/push. O próximo passo deve ser decisório: fechar SG-1
e SG-2 no L0 com matriz explícita de plataforma, privilégio, budget e cleanup. Depois,
corrigir os `GATE-DEFECTs` B2/B3, recongelar RED e somente então autorizar produção.
