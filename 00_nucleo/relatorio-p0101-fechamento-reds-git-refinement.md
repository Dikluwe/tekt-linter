# Relatório P0101 — fechamento dos REDs Git

**Data:** 2026-08-25  
**Branch:** `codex/audit-git-refinement-functional`  
**HEAD confrontado:** `0b1e4a9`  
**Resultado:** `BLOCKED`  
**Backlog:** F05 permanece aberto

## Resultado executivo

P0101 reduziu o buraco funcional, mas não o fechou. A rota produtiva agora atravessa a
seam controlada de L3 e o overflow de blob é detectado incrementalmente. Esses resultados
fecham R1 e R3 no Unix.

Três bloqueios materiais permanecem: Job Object Windows não foi implementado nem
executado; a contenção do object database usa varreduras antes/depois e conserva uma
janela TOCTOU; e process group Unix não contém descendente que execute `setsid`, podendo
deixar readers bloqueados.

## Matriz final

| ID | Resultado | Evidência |
|---|---|---|
| R1 rota paralela | `PASS` | B3 3/3; seis operações controladas e duas resoluções únicas |
| R2 Job Object | `RED / BLOCKED` | B4 0 testes; APIs Job Object ausentes da produção |
| R3 oversized/EOF | `PASS` Unix | B1 4/4; `BudgetExhausted` antes do deadline |
| R4 symlink/TOCTOU | `RED` | B2 7/7 não cobre troca transitória restaurada |
| R5 lifecycle integral | `RED / BLOCKED` | escape de process group Unix; runtime Windows ausente |

## Arquitetura Tekt

L1 e L2 permanecem sem processo/filesystem. Processo, framing, budgets, object database e
projeção de conteúdo ficam em L3. L4 faz somente seleção do executável e composição da
rota publicada. Não houve mudança de parsing CLI, exits/F09, contrato L0 ou dependência.

A varredura recursiva de `.git/objects` é registrada como correção parcial, não como nova
regra arquitetural: além do custo proporcional ao banco, ela não oferece contenção forte
contra TOCTOU.

## Validação

- B1 4/4; B2 7/7; B3 3/3; B4 0/0 `NOT RUN`;
- gates P0100 7/7 + 4/4;
- Git histórico 6/6 e CLI refinement 10/10;
- suíte workspace completa `PASS`;
- V5/V6/V7/V12 `PASS`; reparador V5 dry-run sem alterações;
- hashes L0 e gates conferidos;
- `rustfmt --check` dirigido e `git diff --check` limpos.

## Fechamento

Classificação final: `BLOCKED`, sem `SPEC-GAP`. Há `GATE-DEFECT` em B1 e B2 por cobertura
insuficiente dos escapes descritos acima. O branch não deve ser integrado, enviado ou
usado para declarar F05 fechado.

O próximo passo é finito: substituir a inspeção pathname por contenção resistente a
TOCTOU, confrontar `setsid`/readers fora do grupo e implementar/executar Job Object em
Windows real. Só então repetir D e considerar integração.
