# Passo operacional 0073 — saneamento segregado de config e walker

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0005, commit `737aac5`

## Objetivo

Fechar os quatro REDs de configuração/descoberta antes de continuar a triagem, pois
essas fronteiras definem o universo analisado e podem contaminar qualquer gate posterior.

## Decisões congeladas

1. `[layers]` aceita apenas `L0`, `L1`, `L2`, `L3`, `L4`, `lab` e `Lab`; `lab` e `Lab`
   são aliases da mesma chave e não podem coexistir. Diretório vazio, absoluto, com
   `.`/`..`, separador ou repetido entre layers bloqueia `CrystallineConfig::load`.
2. O walker emite `SourceFile` em ordem canônica crescente por path relativo,
   independente da ordem de criação e da enumeração do filesystem.
3. Teste adjacente só conta quando o candidato é arquivo regular local. Diretório,
   symlink, FIFO, socket ou device não constituem cobertura.
4. Erro produzido por `WalkDir` não é descartado. Deve virar `SourceError::Unreadable`
   com path e motivo, mantendo os demais resultados observáveis.
5. Walker não segue symlink de arquivo ou diretório e não lê conteúdo fora da raiz.
6. Exclusões, linguagens, convenções de teste e `Layer::Unknown` que passaram no
   assessment permanecem inalteradas.

## Segregação

- Agente A lê este passo, prompts e produção; não lê assessment/teste adversarial.
- Agente B lê este passo e `tests/config_walker_assessment.rs`; não lê produção.
- Agente C revisa a implementação após o primeiro gate; não lê testes de B.
- O orquestrador transmite apenas sintomas mínimos e classifica divergências.

## Gate

1. Ativar os quatro REDs do assessment 0005 sem relaxá-los.
2. Acrescentar fixtures para alias `lab/Lab`, paths inválidos de layer, FIFO Unix e
   symlink interno/externo como falso teste.
3. Criar seam ou fixture determinístico para erro de travessia; se a plataforma não
   permitir, manter teste condicional explícito, nunca alegação de prova universal.
4. Rodar `cargo test --workspace`, auto-lint V1/V5/V7 e `git diff --check`.
5. Revisão adversarial deve permutar TOML/criação e tentar tipos especiais de arquivo.

## Parada

Relatar em `00_nucleo/relatorio-p0073-saneamento-config-walker.md`, manter worktree
limpo e não fazer merge, instalação ou release. Após o gate, retomar a triagem integral.
