# Relatório — investigação de refinamento entre revisões

**Data:** 2026-08-24  
**Escopo:** somente Fase 0; nenhuma alteração em L1–L4 e nenhuma dependência adicionada

## Resultado

Recomenda-se um processo Git local, endurecido e somente leitura, usando
`cat-file --batch-command --buffer`. A recomendação está registrada como adenda
**proposta** no ADR-0019 e aguarda aprovação humana. A exportação explícita B1 continua
sendo o caminho sem autoridade Git.

## Estado e ensaio de imutabilidade

Ambiente medido: Git 2.43.0, Rust 1.92.0 e Cargo 1.92.0, no branch
`codex/refinement-validator`. HEAD e revisão posterior eram
`0f4e5df9b8848d81d495f0b9ee0aad62bb13fe20`; a revisão anterior era
`18a9b6ef82d2a982af85ea4c969ef674d095f2e0`.

O working tree já estava deliberadamente sujo: `00_nucleo/README.md` modificado e o
passo desta investigação não rastreado. Antes e depois da resolução dos refs,
`ls-tree` e leitura batch, permaneceram iguais:

| Estado | Antes | Depois |
|---|---|---|
| HEAD | `0f4e5df9…` | `0f4e5df9…` |
| branch | `codex/refinement-validator` | `codex/refinement-validator` |
| hash do diff do índice | `e3b0c442…` | `e3b0c442…` |
| hash do diff rastreado | `c46c6ca0…` | `c46c6ca0…` |
| hash do status NUL | `4f9ed064…` | `4f9ed064…` |
| hash da lista de stash | `e3b0c442…` | `e3b0c442…` |

Os trees resolvidos foram `50d882b2…` e `9dc6292e…`; o mesmo path lógico resultou
nos blobs `7de43afe…` e `d166b810…`. Uma entrada parecida com opção (`--help`) colocada
depois de `--end-of-options` falhou como revisão (exit 128), sem ser interpretada como
flag.

## Custo e dependências

Ensaio local curto, 200 leituras do mesmo arquivo após aquecimento:

| Estratégia | Tempo observado |
|---|---:|
| um `git show` por arquivo | 416 ms |
| um processo `cat-file` batch | 14 ms |

O resultado é aproximadamente 27 vezes menor neste caso e justifica processo único
por operação, não processo por arquivo. É microbenchmark de decisão, não promessa de
performance.

O baseline possui 91 linhas únicas na árvore `cargo tree --locked` e binário release
local de 14.729.984 bytes. A opção A adiciona zero crates e zero bytes de biblioteca
Git; passa a exigir um executável Git compatível. `gix` é implementação Git em Rust com
modelo explícito de confiança, mas adicionaria uma árvore ampla de features. `git2` é
binding para libgit2 e pode trazer compilação/linkagem C vendorizada. Não foram
baixadas nem compiladas dependências durante esta investigação, portanto não se
inventam números de tamanho para B.

## Comparação

| Critério | A: Git batch | B: biblioteca | C: exportação externa |
|---|---|---|---|
| nova dependência no binário | nenhuma | alta/variável | nenhuma |
| autoridade adicional | processo Git local | parser Git embutido | nenhuma |
| ergonomia | alta | alta | baixa |
| compatibilidade de objetos | Git instalado | versão da crate/libgit2 | responsabilidade do utilizador |
| risco principal | processo/config/protocolo | supply chain e parser | snapshot incorreto/manual |
| escolha | **recomendada** | adiada | fallback permanente |

## Casos inconclusivos e limites

Objeto prometido porém ausente, timeout, blob/path acima do orçamento, symlink e
submódulo não podem produzir `PRESERVED`. Devem resultar em razão tipada ou erro de
entrada. A proposta limita 512 paths, 4 MiB por blob, 32 MiB por revisão e 10 segundos
por operação. Esses valores precisam de fixtures antes de se tornarem contrato.

Não foram testados nesta máquina Windows, macOS, repositório SHA-256, partial clone,
alternates, LFS, symlink ou submódulo; todos permanecem critérios RED da eventual
materialização. A ausência desses ensaios é razão para o gate, não licença para inferir
suporte.

## Fontes primárias

- Git, [`git-cat-file`](https://git-scm.com/docs/git-cat-file): protocolo batch,
  buffering e filtros opt-in.
- [`gix`](https://docs.rs/gix/latest/gix/): implementação Rust e modelo de confiança.
- [`git2-rs`](https://github.com/rust-lang/git2-rs): bindings, features de rede e
  libgit2 vendorizada.
- [`git2::Repository`](https://docs.rs/git2/latest/git2/struct.Repository.html): APIs
  de commit, tree e blob.

## Gate

A Fase 0 terminou sem código de produto. Em seguida o humano aprovou a adenda e a B2
foi materializada no branch dedicado: `refine-revisions` resolve OIDs uma vez, lê
trees/blobs por Git batch, aplica budgets e reutiliza o extrator/comparador existentes.
Nenhuma dependência foi adicionada.

O ensaio real `18a9b6e → 0f4e5df` retornou `PRESERVED` (exit 0); o controle histórico
`f8a0dae → 0f4e5df` retornou `UNKNOWN(missing-observable)` (exit 2). A suíte fechou com
579 testes unitários, 83 fixtures gerais e 7 testes CLI; auto-lint e hashes causais
passaram. Revisão versus working tree, SMT e análise interprocedural continuam adiados.
