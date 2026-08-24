# Passo operacional 0089 — auditoria segregada das projeções V0/PARSE

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** fechado — `READY WITH RESIDUAL AUDIT`
> **Branch prevista:** `codex/audit-v0-parse-projections`
> **Pré-condição:** P0088 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0088

## Objetivo

Auditar o componente ainda não coberto que aparenta menor risco: a projeção fechada de
`SourceError` e `ParseError` em `Violation`, hoje materializada em
`04_wiring/main.rs`. O lote verifica V0/PARSE, sem reauditar walkers, parsers, formatters
ou o pipeline paralelo.

A escolha é deliberadamente pequena: dois enums fechados, quatro modalidades e nenhuma
dependência externa nova. O risco arquitetural, porém, deve ser confrontado antes de
ratificar a produção: L4 pode compor e injetar, mas não deve acumular política de domínio
ou apresentação sem causalidade explícita.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `ed44ffdda0a323df26a25cef40c0acb46bd692db6fdaef861a20a509adeb7029` |
| diagnósticos | `00_nucleo/prompts/violation-types.md` | `b50d90505e311a1aa99d3c80988f3f7996fe7974d71579543f86c0553a4dc314` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceError | `00_nucleo/prompts/contracts/file-provider.md` | `f00e05231f34b29256692d7b0e9f2f17db82417c1e3c3d93f922028f36fc189e` |
| fail-fast | `00_nucleo/adr/0004-reformulação-do-motor-de-análise.md` | `beea8faffd2a446ff5744bd1b5d5b6a148d86f53fc9508a31f98fd039634fcff` |
| paths owned | `00_nucleo/adr/0005-location-owned-paths-e-cargo.toml-como-artefato-gerido.md` | `917f4a1194e3d7b2a6955b6182684ad55bf909705cbf2537b095145d22b78421` |

Qualquer alteração nesses arquivos invalida o Assessment 0018 até novo resselamento.

## Alegações candidatas

1. `SourceError::Unreadable` produz exatamente uma violação V0 `Fatal`.
2. V0 preserva path e razão, usa linha/coluna zero e `Cow::Owned`.
3. `ParseError::SyntaxError` produz PARSE `Error`, preservando path, linha, coluna e
   mensagem causal.
4. `UnsupportedLanguage` produz PARSE `Warning`, preservando linguagem e path, com
   posição zero.
5. `EmptySource` produz PARSE `Warning`, preservando path e posição zero.
6. Nenhuma modalidade vira silêncio, V0/PARSE trocam de ID ou perdem evidência.
7. Entradas clonadas e paths Unicode/hostis geram resultados determinísticos e não
   modificam o erro recebido nem o filesystem.
8. A projeção não lê configuração, filesystem, ambiente, relógio, rede ou processo.

## Preflight arquitetural obrigatório

Antes de criar um gate executável, o Assessment 0018 deve classificar:

- se a transformação erro→violação é política pura pertencente a L1, apresentação L2
  ou composição permitida em L4;
- se as mensagens em português/inglês são normativas ou drift histórico;
- se `UnsupportedLanguage` e `EmptySource` realmente devem ser Warning;
- se linha/coluna zero é representação normativa de posição ausente;
- qual API pública permite gate black-box sem importar o binário L4 nem copiar a
  implementação.

Ausência de decisão é `SPEC-GAP`. É proibido tornar funções L4 públicas ou movê-las de
camada apenas para facilitar teste antes desse fechamento L0.

## Protocolo segregado

### A — Assessment e adversário cego

1. Criar `00_nucleo/assessments/0018-v0-parse-projections.md` com baseline pós-merge do
   P0088, alegações, papéis e estes hashes.
2. Um adversário recebe somente Assessment 0018 e L0 autorizado; não lê produção,
   testes, histórico ou relatórios.
3. Congelar toda divergência como `SPEC-GAP`, `GATE-DEFECT` ou alegação executável.
4. Sanear L0 e resselar hashes antes de materializar qualquer API ou teste.

### B — Gate cego

Um verificador novo materializa a matriz completa de quatro modalidades usando apenas a
API pública autorizada. O gate cobre severidade, ID, mensagem/evidência, ownership do
path, posições zero e não zero, Unicode, strings vazias/hostis, determinismo e ausência
de efeitos. Ele não pode executar parser/walker como oráculo nem ler a função existente.

### C — Confronto e correção

Somente após o gate congelado, confrontar `04_wiring/main.rs` e materializações causais.
Correção funcional exige RED prévio. Mudança de camada exige decisão L0 explícita e deve
preservar a gravidade Tekt:

- L1: entidades e transformação pura, se normatizada ali;
- L2: apresentação, somente se o contrato decidir que texto pertence ao shell;
- L3: nenhuma política V0/PARSE;
- L4: apenas composição, injeção e encaminhamento.

Não tocar parsers, walker, SARIF, CLI ou ordenação salvo RED pertencente ao lote.

### D — Adversário final

Outro confronto verifica causalidade L0→código→gate, ausência de lógica indevida em L4,
gravidade, preservação de `Fatal`, fail-closed, ausência de delta escondido e regressão
dos assessments 0001–0017.

## Classificações

- `RED`: produção contradiz alegação executável congelada;
- `SPEC-GAP`: L0 não decide camada, API ou comportamento necessário;
- `GATE-DEFECT`: teste inventa ou compartilha oráculo com a produção;
- `PASS`: alegação confrontada sem divergência;
- `READY WITH RESIDUAL AUDIT`: todos os bloqueios do lote fechados.

## Validação mínima de fechamento

1. gate segregado V0/PARSE;
2. testes unitários dos contratos `file_provider` e `parse_error`;
3. fixtures V0/PARSE existentes, sem usá-las como fonte normativa;
4. `cargo test --workspace --quiet`;
5. auto-lint V4/V5/V11/V12 nas materializações tocadas;
6. `cargo run --quiet -- . --fix-hashes --dry-run`;
7. `rustfmt --check` somente nos arquivos funcionais do lote;
8. `git diff --check` contra o baseline;
9. busca mecânica por I/O/configuração nas funções puras;
10. adversário final e worktree limpo após commit de fechamento.

## Saídas esperadas

- Assessment 0018;
- gate black-box V0/PARSE;
- L0 saneado, apenas se houver SPEC-GAP;
- correção mínima, apenas se houver RED;
- relatório `00_nucleo/relatorio-p0089-auditoria-v0-parse.md`;
- veredito `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

P0089 não autoriza merge, push, instalação ou release. Se a pré-condição de integração
do P0088 não estiver satisfeita, a execução deve parar sem criar branch concorrente.

## Fechamento

Executado em 2026-08-24 após integração de P0088 em `master@cc1924b`. Os SPEC-GAPs de
camada/API, os REDs de produção e o gate documental final foram fechados. Evidência e
residuais constam em `relatorio-p0089-auditoria-v0-parse.md`.
