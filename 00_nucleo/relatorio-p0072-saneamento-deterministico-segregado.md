# Relatório P0072 — saneamento determinístico segregado

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Estado:** gate local concluído; sem merge ou instalação

## Linha de evidência

- L0 congelado: `939a99e` (`docs: define P0072 deterministic sanitation`);
- REDs legados: `a20647e`, `f46e20e`, `420273a` e `fc6b54d`;
- implementação e gates ativos: `0bfda5f`;
- nenhum código legado é reclassificado retroativamente como `sealed`.

Agente A recebeu P0072, prompts e produção, sem acesso aos assessments e relatórios
adversariais. Agente B recebeu L0 e gates, sem ler produção modificada. Agente C recebeu
L0 e produção somente após o primeiro gate, sem ler os testes de B. O orquestrador foi
o único papel a classificar e encaminhar recibos entre produtores.

## Resultado D1–D6

| Decisão | Materialização | Gate final | Limite residual |
|---|---|---|---|
| D1 — aliens canônicos | ordenação e deduplicação em merge local/global | PASS sob permutação e partição | nenhum no escopo finito testado |
| D2 — ordem total | severidade, path, linha, coluna, regra e mensagem | PASS em 24 permutações | comparação usa ordem nativa de `Path` |
| D3 — registry falível | normalização central, dedup semântico e rejeição de conflitos | PASS, inclusive rota direta e TOML inválido | projeto não Cargo continua registry vazio por contrato |
| D4 — location V22 | menor path do módulo | PASS sob inversão de arquivos | percentual continua com uma casa decimal |
| D5 — object DB local | ambiente limpo e preflight de gitdir/objects/alternates | PASS para alternates e `.git` symlink | worktree vinculada, bare repo e object pool ficam fechados |
| D6 — paths lossless | escape humano e URI ASCII percent-encoded | PASS Unix para byte inválido e `%` | caso Windows possui unit test condicional, não foi executado neste host |

## Divergências intermediárias

O primeiro gate funcional passou D1–D6, mas o auto-lint encontrou V1 no novo módulo
`path_encoding`. O agente A adicionou linhagem para `sarif-formatter.md`, atualizou o
hash pelo fluxo do linter e eliminou V1/V5/V7.

O agente C então reabriu o gate:

1. `.git` como symlink para outro repositório ainda permitia `PRESERVED`;
2. `CrateRegistry::from_members` ainda aceitava colisões internas normalizadas em deps
   e renames, embora a rota TOML já as rejeitasse.

O agente B reproduziu ambos como RED antes da correção. O agente A confinou gitdir e
objects e centralizou a normalização do registry. B reexecutou os gates em verde. C
reexecutou seus probes próprios e encerrou com veredito **não reabrir**.

No gate final, o auto-lint detectou ainda uma construção Rust aceita pelo compilador,
mas não pelo tree-sitter vigente em `crate_registry.rs`. A sintaxe foi simplificada sem
alteração semântica; o PARSE e o falso prompt órfão consequente desapareceram.

## Gates finais

- 597/597 testes unitários;
- 83/83 fixtures gerais;
- assessment Git: 6/6;
- assessment registry/inventário: 7/7;
- assessment de entidades: 4/4;
- refinamento anterior: 10/10;
- selo segregado: 16/16;
- assessment de apresentação: 4/4;
- zero testes ignorados nos assessments;
- auto-lint focal V1/V5/V7: limpo;
- `git diff --check`: limpo.

Permanece um warning Rust preexistente para a função de teste `print_tree` em
`ts_parser.rs`; P0072 não o criou nem o alterou.

## Estado das alegações

As mudanças P0072 foram produzidas sob materialização segregada e possuem gates ativos.
Os módulos legados ao redor continuam apenas `assessed`: o processo aumenta confiança
nas propriedades D1–D6, mas não certifica sua origem histórica nem correção completa.

## Recomendação

Retomar a triagem de baixo risco, agora procurando a mesma família em walkers,
configuração e coletores: toda agregação oriunda de filesystem, TOML ou Rayon deve
possuir desempate canônico ou rejeição explícita de ambiguidade. Antes de integração,
executar a matriz Windows/macOS para D6 e decidir se worktrees vinculadas/bare repos
merecem suporte em um contrato separado.

## Parada cumprida

Não houve merge em `master`, instalação de binário, release ou marcação retroativa do
legado como `sealed`.
