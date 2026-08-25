# Relatório P0092 — auditoria do planejamento update-snapshot

**Data:** 2026-08-24
**Branch:** `codex/audit-update-snapshot-planning`
**Baseline pós-P0091:** `aee1344`
**Resultado provisório:** `BLOCKED`

## Escopo

O lote auditou somente o caso de uso L2 `update_snapshot`: construir plano a partir de
violações V6 e executar esse plano pelo port. Writer L3, serialização canônica, parsing de
snapshot, V6 e CLI ficaram fora do oráculo funcional.

## SPEC-GAP e saneamento

O L0 alternava `SnapshotWriter`/`SnapshotRewriter`, não publicava tipos de plano/resultado
e não decidia duplicatas, unreadable ou dry-run. Antes de ler produção, o contrato passou
a fixar:

- port L2 único com serialização e escrita separadas;
- enums `Ready/Unreadable` e `DryRun/Written/WriteFailed/Unreadable`;
- uma entrada por ocorrência V6, ordem e duplicatas preservadas;
- primeiro `ParsedFile` de path integralmente igual;
- uma serialização por `Ready` e nenhum efeito nos demais;
- um resultado por entrada, escrita única, erro exato e continuação.

## RED e correção

Os gates cegos falharam porque os enums não existiam. O confronto confirmou, além disso,
que `execute` apagava entradas não acionáveis via `filter_map`. O commit `e106a38`
substituiu estados opcionais por enums e `filter_map` por transformação total.

## Matriz causal

| Autoridade/ação | Camada | Evidência |
|---|---|---|
| tipos V6 e `ParsedFile` | L1 | insumo somente leitura |
| planejamento, estados e port | L2 | `update_snapshot.rs` |
| serialização/escrita externa | L3 | implementação do port, fora do oráculo |
| instanciação e ciclo | L4 | consumidor sem decisão duplicada |
| planejamento black-box | B1 | 3/3 PASS |
| execução black-box | B2 | 2/2 PASS |

## Validação

- 628 testes unitários e todos os gates de integração: PASS;
- 83 fixtures: PASS;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- reparador de hashes: `Nothing to fix`;
- `rustfmt` dirigido e `git diff --check`: PASS;
- busca em L2: nenhum filesystem, ambiente, relógio, rede, processo ou import L3.

O primeiro adversário D confirmou causalidade e arquitetura Tekt, mas encontrou RED na
apresentação real do dry-run: `format_plan` omite o snapshot/interface e
`format_results` chamaria dry-run de atualização concluída. O fechamento aguarda gate,
correção e repetição D. Não houve merge ou push.
