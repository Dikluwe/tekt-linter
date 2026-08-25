# Relatório P0100 — auditoria funcional Git de `refine-revisions`

**Data:** 2026-08-25
**Branch:** `codex/audit-git-refinement-functional`
**Baseline:** `8c28cc01ea7cdb47aa9e8e582597085304a7ece4`
**Resultado:** `BLOCKED`
**Backlog:** F05 permanece aberto

## Resultado

P0100 materializou uma seam Git hostil testável e tornou onze gates novos verdes, mas o
adversário demonstrou que a evidência ainda não cobre o caminho produtivo completo nem
algumas propriedades críticas de contenção. O lote não está pronto para merge.

## Preflight e saneamento

A1 encontrou oito `SPEC-GAP` e uma contradição sobre alternates. O L0 foi saneado com
autocontenção, API pública L3, transcript, ambiente, framing, budgets, lifecycle e
taxonomia fechada. A2 terminou 12/12 e 13/13 `PASS`, liberando B1/B2.

B1/B2 congelaram RED de compilação pela ausência da API. Três `GATE-DEFECT` foram
preservados e corrigidos:

- B2 identificava o subcomando pelo último argv;
- B1 usava shell que sintetizava `PATH` após `env_clear()`;
- B2 não lia `.scenario` sem newline e não exercitava os cenários hostis.

As fixtures finais são independentes: B1 usa binário Rust nativo; B2 usa fixture própria
para status, saída parcial, timeout e descendente.

## Implementação parcial

`03_infra/git_refinement.rs` recebeu tipos públicos, `load_revision_with_git`, validação
pre-spawn, ambiente/argv controlados, OID opaco, pathspec literal, framing, tipos,
budgets, caps, grupo Unix e taxonomia. Headers causais foram atualizados pelo reparador
oficial.

Resultados:

- B1: 7/7 PASS;
- B2: 4/4 PASS, incluindo deadline real de 10 segundos;
- gate Git histórico: 6/6 PASS;
- CLI refinement: 10/10 PASS;
- V5/V6/V7/V12: PASS;
- reparador V5: `Nothing to fix`;
- `git diff --check`: PASS.

## REDs adversariais bloqueantes

1. A rota produtiva ainda usa o adapter histórico e não a seam confrontada.
2. Windows não implementa Job Object/encerramento de descendentes.
3. Oversized declarado com pipe aberto termina em timeout, não budget imediato.
4. Symlinks em loose objects/packs acessados não são excluídos recursivamente.
5. Líder encerrado com descendente segurando pipes pode bloquear joins sem deadline;
   esperar apenas o líder também não comprova reap de todos os membros do grupo.

Os gates também precisam de casos específicos para os itens 2–5. Testes verdes atuais
não podem ser promovidos a prova implícita desses comportamentos.

O RED documental do pin final foi fechado após D: `refinement-validator.md` está
registrado em `9ab972915e8f21e6c0fc323686d507fb2cb4b590de6d987b454e05642f167818`.

## Próxima retomada de F05

A retomada deve começar por resselar o Assessment e acrescentar gates, antes de tocar
produção, para:

- oversized com stdout mantido aberto;
- líder encerrado e descendente segurando stdout/stderr;
- symlink em loose object e pack acessível;
- contenção Windows por Job Object, ou saneamento normativo explícito de plataforma;
- rota produtiva usando a mesma seam e projetando `GitRevisionContent` ao extrator.

Somente depois devem ser refatorados o parser incremental, o watchdog que cobre pipes,
a autocontenção e o wiring mínimo. F09 e F08 continuam fora, salvo a ligação estritamente
necessária para eliminar a duplicação de adapter.

Nenhum push foi realizado e o branch P0100 não foi integrado.
