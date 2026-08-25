# Passo operacional 0092 — auditoria segregada do planejamento update-snapshot

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-update-snapshot-planning`
> **Pré-condição:** P0091 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0091

## Objetivo

Auditar a seam L2 que transforma violações V6 e `ParsedFile`s em plano de atualização e
executa esse plano pelo port `SnapshotRewriter`, hoje materializada em
`02_shell/update_snapshot.rs`.

O lote não reabre parsing/serialização do snapshot, escrita atômica, confinamento de
paths, V6, CLI, wiring ou parsers. Essas responsabilidades permanecem nos contratos e
assessments próprios; L2 só planeja, chama o port e apresenta o resultado.

## Hipótese de baixo risco

`plan` e `execute` aparentam ser transformações mecânicas sobre slices. O risco causal é
silenciar uma entrada não acionável, associar uma violação ao arquivo errado, alterar
ordem/cardinalidade ou converter falha do port em sucesso.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| fluxo fix/update | `00_nucleo/prompts/fix-hashes.md` | `dd987d35beced5bd7fb6a0961f1e2cfa08d85d4c6f8a3702f797f7a6f32e8024` |
| contrato snapshot | `00_nucleo/prompts/contracts/prompt-snapshot-reader.md` | `80b6f7ab9fbb0f97fa085d7a34802792eb6fce4834ac204775b47749c77985be` |
| tipos V6/IR | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| apresentação/CLI | `00_nucleo/prompts/sarif-formatter.md` | `959d6e56785e6c32087fcae361300304d4a8197a2669f9df7f2b4809a4842605` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

Qualquer alteração invalida o Assessment 0021 até resselamento.

## Alegações candidatas

### Planejamento

1. Somente violações cujo `rule_id` é exatamente `V6` produzem entradas.
2. Cada V6 produz exatamente uma entrada, preservando ordem e duplicatas.
3. A associação usa igualdade integral do path, sem prefixo, normalização, canonicalização
   ou fallback para basename.
4. Ausência de `ParsedFile` e ausência de `PromptHeader` permanecem distinguíveis e
   observáveis; nenhuma delas chama serialização.
5. Para entrada acionável, source path, prompt path e interface pública vêm do mesmo
   `ParsedFile`, e a serialização ocorre exatamente uma vez.
6. Múltiplos `ParsedFile`s com o mesmo path têm precedência explícita e determinística;
   ausência dessa decisão no L0 é `SPEC-GAP`.

### Execução

7. `dry_run = true` nunca chama escrita e preserva uma resposta observável para cada
   entrada acionável.
8. Execução real chama o port exatamente uma vez por entrada acionável, na ordem do plano,
   com prompt path e snapshot inalterados.
9. `Ok` e `Err` do port são preservados por entrada; uma falha não interrompe, apaga ou
   muda entradas posteriores.
10. Entradas não acionáveis não podem desaparecer silenciosamente. O L0 deve decidir se
    viram `SnapshotResult` de falha ou se permanecem somente no plano/formatter.
11. L2 não lê filesystem, ambiente, relógio, rede ou processo; L3 permanece único dono da
    escrita.

## Preflight arquitetural obrigatório

Assessment 0021 e adversário A devem decidir antes do gate:

- contrato público completo de `SnapshotEntry` e `SnapshotResult`, incluindo igualdade
  ou campos observáveis suficientes ao black-box;
- precedência para `ParsedFile`s duplicados e cardinalidade por violação;
- semântica executável de entradas com `unreadable_reason`;
- se serialização pertence ao port L2 ou deve ser transformação pura separada;
- ordem e política de continuação após falha;
- distinção entre `dry-run` bem-sucedido e escrita realmente realizada.

Ausência de decisão é `SPEC-GAP`. É proibido ler L3 ou usar o writer real como oráculo do
gate.

## Protocolo segregado

### A — Assessment e adversário L0

1. Criar `00_nucleo/assessments/0021-update-snapshot-planning.md` com baseline pós-merge
   e hashes autorizados.
2. Adversário A lê somente Assessment/L0; produção, testes, lab, histórico e relatórios
   são proibidos.
3. Congelar alegações, precedências e `SPEC-GAPs`.
4. Sanear L0 e resselar antes de qualquer gate ou confronto.

### B1 — Gate cego de planejamento

Um verificador novo cobre mistura V6/não-V6, paths hostis, ausências, duplicatas,
permutação, ordem, multiplicidade e spies de serialização. O gate usa apenas tipos
construídos a partir do L0 hash-pinned.

### B2 — Gate cego de execução

Outro verificador, em arquivo separado, cobre dry-run, sequência de chamadas, Ok/Err
intercalados, continuação após falha e entradas não acionáveis. O port é um spy puro; L3,
filesystem e testes existentes são proibidos.

### C — Confronto e correção

Somente após B1/B2 congelados, confrontar `02_shell/update_snapshot.rs`. Correção
funcional exige RED. A solução deve respeitar:

- L2 possui o caso de uso e o port;
- L3 implementa serialização/escrita externa;
- L4 instancia o adapter e apenas orquestra o ciclo;
- L1 fornece tipos/diagnósticos, sem conhecer o comando de mutação.

Nenhum writer, reader, parser, regra, CLI ou wiring pode mudar salvo RED causal do lote ou
reparo mecânico de hash.

### D — Adversário final

Verificar causalidade, gravidade, gates realmente independentes, ausência de oráculo
compartilhado, arquitetura Tekt, delta escondido e regressão dos assessments 0001–0020.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: L0 não decide cardinalidade, precedência ou erro observável;
- `GATE-DEFECT`: gate compartilha implementação, L3 ou expectativa inventada;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. gates B1 e B2 em arquivos e identidades separadas;
2. testes dirigidos de `update_snapshot` e `prompt_snapshot_reader`;
3. regressão do P0074 sem reabrir seu oráculo;
4. `cargo test --workspace --quiet`;
5. auto-lint V5/V6/V7/V12;
6. `cargo run --quiet -- . --fix-hashes --dry-run`;
7. `rustfmt --check` dirigido e `git diff --check`;
8. busca mecânica por I/O/imports L3 em L2;
9. adversário final e worktree limpo.

## Saídas esperadas

- Assessment 0021;
- gates cegos B1/B2 separados;
- L0 saneado se necessário;
- correção mínima após RED;
- relatório `00_nucleo/relatorio-p0092-auditoria-update-snapshot.md`;
- matriz L0→L2→port→L3/L4→gates;
- veredito final.

P0092 não autoriza merge, push, instalação ou release. Sem integração prévia do P0091,
a execução deve parar antes de criar branch concorrente.
