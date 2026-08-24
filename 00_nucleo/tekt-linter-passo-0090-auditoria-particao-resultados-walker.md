# Passo operacional 0090 — auditoria segregada da partição de resultados do walker

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-walker-result-partition`
> **Pré-condição:** P0089 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0089

## Objetivo

Auditar o próximo componente ainda não coberto que aparenta menor risco:
`collect_walker_results`, a partição pura de
`Iterator<Item = Result<SourceFile, SourceError>>` em vetores de sucessos e erros.

O lote não reabre `FileWalker`, descoberta de arquivos, configuração, parsers, rayon ou
V0. Ele verifica somente se a fronteira preserva integralmente cada item, ordem,
multiplicidade e separação, sem silenciar `SourceError`.

## Justificativa de baixo risco

- dois ramos fechados (`Ok` e `Err`);
- transformação determinística e sem I/O;
- nenhuma heurística, linguagem ou AST;
- nenhuma severidade ou mensagem;
- entrada e saída compostas apenas por tipos L1 já normatizados.

O risco arquitetural deve ser tratado explicitamente: embora hoje esteja em L4, uma
partição pura não é composição/injeção. O gate não pode legitimar lógica utilitária em
L4 apenas porque o corpo é pequeno.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/pipeline | `00_nucleo/prompts/linter-core.md` | `70ed01e7dd64b9da727d35b0341ee67712f0434d75543ac05697b111acee864e` |
| tipos/contrato | `00_nucleo/prompts/contracts/file-provider.md` | `f5ed3805807f730576bd3af99d850eacfad49b9c2c1708f10aacd04c0af2e9ce` |
| walker | `00_nucleo/prompts/file-walker.md` | `6deeec38a766c6ac16f8aa90944e75a6b6d22c91db1249f1d99fdf51c697a7c2` |
| motor/fail-fast | `00_nucleo/adr/0004-reformulação-do-motor-de-análise.md` | `25d0571e0621b207b59d79ffd4ce6dfd31008738812a06fd82d0ac95d8d7fe3d` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

Qualquer alteração invalida o Assessment 0019 até resselamento.

## Alegações candidatas

1. Entrada vazia produz dois vetores vazios.
2. Cada `Ok(SourceFile)` aparece exatamente uma vez no vetor de sucessos.
3. Cada `Err(SourceError)` aparece exatamente uma vez no vetor de erros.
4. A ordem relativa dos sucessos e a ordem relativa dos erros são estáveis e iguais às
   respectivas subsequências da entrada.
5. Intercalação, duplicatas e itens estruturalmente iguais preservam cardinalidade.
6. Um erro nunca encerra, descarta ou reclassifica itens posteriores.
7. Paths, conteúdo, linguagem, camada, adjacência e razões são preservados sem
   normalização ou inferência.
8. O iterador é consumido uma única vez e até EOF; não há clone, replay ou coleta oculta
   adicional do produtor.
9. A função não acessa filesystem, configuração, ambiente, relógio, rede ou processo.

## Preflight arquitetural obrigatório

Antes do gate, o Assessment 0019 e um adversário cego devem decidir:

- se a partição pertence a L1 como transformação pura ou se existe exceção causal para
  permanecer em L4;
- API pública black-box e nome do módulo/função;
- ownership exato de entrada/saída e aceitação de iteradores não-`Clone`;
- se estabilidade de ordem é normativa, apesar do pipeline posterior usar rayon;
- comportamento diante de `size_hint` hostil e iterador que registra número de `next`.

Ausência de decisão é `SPEC-GAP`. É proibido publicizar a função privada de L4 ou criar
um gate via binário antes do saneamento causal.

## Protocolo segregado

### A — Assessment e adversário L0

1. Criar `00_nucleo/assessments/0019-walker-result-partition.md`, fixando baseline
   pós-merge P0089 e todos os hashes.
2. O adversário lê somente Assessment 0019 e L0 autorizado; produção, testes, histórico
   e relatórios permanecem proibidos.
3. Congelar alegações executáveis e `SPEC-GAPs` antes de editar L0.
4. Sanear e resselar L0 antes de qualquer API ou gate.

### B — Gate cego

Um verificador novo materializa gate black-box usando iterador próprio instrumentado,
sem filesystem real. A matriz cobre vazio, somente Ok, somente Err, alternância,
duplicatas, Unicode/conteúdo hostil, ordem, cardinalidade, consumo único, EOF e
`size_hint` inexato. O gate não lê produção nem usa walker/pipeline como oráculo.

### C — Confronto e correção

Somente após o gate congelado, confrontar a função existente. Toda correção funcional
exige RED prévio. Se o L0 decidir por L1, L4 apenas chama a transformação; L3 continua
produzindo os resultados e L2 permanece fora do fluxo.

Não tocar enumeração do filesystem, exclusões, symlinks, adjacência, parser, V0,
`ProjectIndex`, rayon ou ordenação global salvo RED pertencente ao lote.

### D — Adversário final

Verificar causalidade L0→código→gate, gravidade Tekt, preservação fail-fast, ausência de
efeitos, inexistência de delta escondido e regressão dos assessments 0001–0018.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: L0 não decide camada/API/comportamento;
- `GATE-DEFECT`: teste inventa ou compartilha oráculo;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. gate segregado da partição;
2. testes dos contratos `file_provider`;
3. regressão dos gates config/walker, V0/PARSE e V0 fail-fast;
4. `cargo test --workspace --quiet`;
5. auto-lint V4/V5/V11/V12;
6. `cargo run --quiet -- . --fix-hashes --dry-run`;
7. `rustfmt --check` somente em arquivos funcionais do lote;
8. `git diff --check` contra baseline;
9. busca mecânica de I/O/configuração na transformação pura;
10. adversário final e worktree limpo após fechamento.

## Saídas esperadas

- Assessment 0019;
- gate black-box da partição;
- L0 saneado se houver `SPEC-GAP`;
- correção mínima somente após RED;
- relatório `00_nucleo/relatorio-p0090-auditoria-particao-walker.md`;
- veredito final e matriz L0→materialização→gate.

P0090 não autoriza merge, push, instalação ou release. Sem integração prévia do P0089,
a execução deve parar antes de criar branch concorrente.
