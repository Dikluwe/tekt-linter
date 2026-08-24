# Relatório P0073 — saneamento segregado de config e walker

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Evidência

- L0: `e9ea719`;
- REDs do assessment 0005: `737aac5`;
- implementação, gate ativo e revisão adversarial: `61cf043`.

Agente A implementou sem ler assessment/gate; B ativou e ampliou o gate sem ler
produção; C revisou a produção após o primeiro gate sem ler os testes de B.

## Fechamento

- `[layers]` possui vocabulário fechado, aliases controlados, paths simples e
  diretórios únicos;
- walker retorna resultados em ordem canônica por path;
- erro `WalkDir` é preservado como `SourceError::Unreadable`;
- symlinks não são seguidos como sources nem como testes;
- somente arquivo regular local prova teste adjacente;
- diretório, FIFO e symlink interno/externo não fabricam cobertura;
- exclusões, linguagens e `Layer::Unknown` mantiveram a semântica anterior.

## Gates

- assessment config/walker: 6/6, zero ignorados;
- testes unitários: 600/600;
- fixtures gerais: 83/83;
- todos os demais assessments e integrações: verdes;
- auto-lint V1/V5/V7: limpo;
- `git diff --check`: limpo;
- revisão adversarial: **NÃO REABRIR**.

O probe dinâmico de socket Unix ficou `SKIP` porque o sandbox recusou `bind` com EPERM.
A implementação usa o mesmo predicado `symlink_metadata().file_type().is_file()` que
rejeitou diretório e FIFO, mas o relatório não eleva inspeção a prova dinâmica.

## Limite residual

Erro real de travessia por EACCES não é portável sob processo privilegiado. O adapter
possui seam unitário determinístico que prova retenção e ordenação conjunta de Ok/Err;
uma matriz sob usuário restrito pode complementar a evidência futuramente.

## Parada

Nenhum merge, instalação ou release foi realizado. A triagem integral pode continuar
sem carregar os quatro REDs de config/walker para os próximos lotes.
