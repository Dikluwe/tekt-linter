# Passo operacional 0074 — saneamento segregado de prompt I/O e hashes

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0006, commit `525e42c`

## Objetivo

Fechar os cinco REDs que afetam V5/V6/V7 antes de continuar a triagem. Leitura, hash,
snapshot e escrita devem compartilhar uma fronteira confinada e byte-exata.

## Decisões congeladas

1. Prompt paths são relativos, não vazios e compostos apenas por componentes normais.
   O arquivo final e todos os ancestrais canônicos permanecem dentro da raiz. Symlink,
   absoluto, `.` e `..` bloqueiam. `exists` só aceita arquivo regular local.
2. Hash opera sobre bytes, com limite aplicado à mesma captura. Remove exatamente uma
   linha meta canônica (`Hash do Código: <8hex>` em prompt ou
   `//! @prompt-hash <8hex>` em source), preservando todos os demais bytes e line endings.
   Meta ausente segue a semântica da operação; meta duplicada/malformada bloqueia.
3. Digest escrito possui exatamente oito hexadecimais minúsculos. Escrita modifica
   somente a linha autorizada, preserva newline, BOM, conteúdo e permissões restantes.
4. Escrita atômica usa temporário irmão exclusivo (`create_new`), permissões do destino,
   `sync_all`, rename e limpeza em erro. Nome não pode ser apenas PID compartilhado.
5. Prompt walker exige `00_nucleo/prompts` como diretório local regular, não segue
   symlinks, ordena paths e transforma erro interno de travessia em `PromptScanError`.
   O comportamento histórico de saltar entrada inacessível é revogado por fail-closed.
6. Snapshot exige exatamente um marcador de linha canônico, fora de texto/fence por
   construção sintática, e schema fechado com todos os campos. Duplicata, campo extra,
   path externo ou JSON parcial retorna `None`.
7. `CachedPromptReader` é snapshot por execução: primeira leitura, inclusive `None`, é
   estável. Frescura após mutação pertence a uma nova instância, não ao mesmo cache.

## Segregação e gate

- A implementa sem ler assessment/gate/lab.
- B ativa os cinco REDs e adiciona concorrência determinística quando possível, sem ler
  produção.
- C revisa após o primeiro gate, sem ler testes de B.
- Executar suíte completa, assessments sem ignore, auto-lint e `git diff --check`.
- Atualizar hashes de linhagem apenas após o L0 final; registrar a migração causada pela
  troca de hash canônico byte-exato.

## Parada

Criar `00_nucleo/relatorio-p0074-saneamento-prompt-io-hash.md`, manter branch/worktree
limpos e não fazer merge, instalação ou release. Retomar a triagem depois do gate.
