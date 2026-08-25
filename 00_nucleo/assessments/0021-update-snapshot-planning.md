# Assessment 0021 — planejamento e execução de update-snapshot

**Estado:** BLOCKED — RED de apresentação dry-run aguardando gate/correção
**Data:** 2026-08-24
**Passo:** P0092
**Baseline:** `aee1344`
**Commit do protocolo no branch:** `83ad45e`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| fluxo fix/update | `00_nucleo/prompts/fix-hashes.md` | `dd987d35beced5bd7fb6a0961f1e2cfa08d85d4c6f8a3702f797f7a6f32e8024` |
| contrato snapshot | `00_nucleo/prompts/contracts/prompt-snapshot-reader.md` | `80b6f7ab9fbb0f97fa085d7a34802792eb6fce4834ac204775b47749c77985be` |
| tipos V6/IR | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| apresentação/CLI | `00_nucleo/prompts/sarif-formatter.md` | `959d6e56785e6c32087fcae361300304d4a8197a2669f9df7f2b4809a4842605` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0092 | `00_nucleo/tekt-linter-passo-0092-auditoria-planejamento-update-snapshot.md` | `27780d89b05911458ce62b81957d95edebc2967c46cb34ecccddc2a8ebbd8415` |

## Alegações candidatas

1. `plan` aceita somente `rule_id == "V6"`, produz uma entrada por violação e preserva
   ordem e duplicatas.
2. A associação com `ParsedFile` usa igualdade integral do path, sem normalização.
3. Ausência de arquivo parseado e ausência de header produzem razões distintas e não
   chamam serialização.
4. Entrada acionável preserva source path/prompt path e serializa exatamente a interface
   do arquivo associado uma única vez.
5. `dry_run` não escreve e continua observável por entrada.
6. Execução real escreve cada entrada acionável exatamente uma vez, na ordem recebida.
7. `Ok`/`Err` são preservados por entrada e falha não interrompe itens posteriores.
8. Entrada não acionável permanece observável também após `execute`.
9. L2 não acessa filesystem, ambiente, relógio, rede ou processo.

## Evidência normativa presente

- `fix-hashes.md` atribui decisão a L2 e escrita a L3.
- A norma exige filtro exato V6, dry-run sem toque no disco e `plan()` sem descarte por
  `filter_map`.
- Falhas de leitura devem carregar `unreadable_reason` em vez de desaparecer.
- `sarif-formatter.md` exige delegação da serialização/escrita via port L2/L4.
- `prompt-snapshot-reader.md` fixa a serialização canônica, mas não o planejamento.

## SPEC-GAPs congelados

### G1 — API e nome do port divergentes

`fix-hashes.md` publica `SnapshotWriter` com `read_interface`, `serialize` e
`write_snapshot(prompt_path, interface)`. O restante do L0 alterna `SnapshotWriter` e o
passo candidato usa `SnapshotRewriter` com serialização separada. Não há uma API única
hash-pinned para gate black-box.

### G2 — tipos de plano/resultado ausentes

O L0 não publica `SnapshotEntry` ou `SnapshotResult`, seus campos, igualdade nem estados
válidos. B1/B2 não podem inventar representação de erro, dry-run ou sucesso.

### G3 — duplicatas e associação ambíguas

Não há precedência para múltiplos `ParsedFile`s com o mesmo path. Também não está dito se
cada V6 duplica uma entrada ou se prompts/paths devem ser deduplicados.

### G4 — entrada não acionável após planejamento

“Nunca descartar” é explícito para `plan`, mas o L0 não decide se `execute` retorna falha
para essa entrada, a omite deliberadamente ou delega sua observabilidade apenas ao
formatter do plano.

### G5 — semântica de dry-run e continuidade

O L0 proíbe escrita em dry-run, mas não define o `SnapshotResult` correspondente. Ordem
de chamadas, chamada única e continuação após `Err` também não estão publicadas.

## Decisão de preflight

Não construir gates nem ler `02_shell/update_snapshot.rs` até G1–G5 serem saneados e os
hashes resselados. A decisão recomendada, sujeita ao adversário A, é:

- port único `SnapshotRewriter` em L2, com serialização e escrita explicitamente
  separadas;
- `SnapshotEntry` e `SnapshotResult` públicos, comparáveis e com estado explícito;
- uma entrada por V6, ordem/duplicatas preservadas e primeiro `ParsedFile` de path exato;
- ausência de parsed/header como entrada não acionável que vira resultado de falha sem
  chamada ao port;
- dry-run como resultado explícito distinto de escrita realizada;
- execução estável, uma chamada por acionável e continuação após falha;
- zero I/O em L2; apenas o port pode produzir efeito externo.

## Parecer adversarial A e saneamento

O adversário A validou os oito hashes iniciais e concluiu `SPEC-GAP / BLOCKED`, com
G1–G5 confirmados. Qualquer gate anterior ao saneamento seria `GATE-DEFECT`.

O L0 foi saneado antes de ler produção:

- API única `SnapshotRewriter` com serialização e escrita separadas;
- enums públicos e comparáveis `SnapshotUnreadable`, `SnapshotEntry` e
  `SnapshotResult`, sem combinações inválidas de `Option`s;
- uma entrada por ocorrência V6, ordem e duplicatas preservadas;
- primeiro `ParsedFile` de path integralmente igual, sem normalização;
- estados distintos para missing parsed/header, sem serialização;
- um resultado por entrada; dry-run distinto, escrita única, erro exato e continuação;
- ownership L2/L3/L4 e ausência de I/O em L2 explícitos.

Os hashes acima resselam o contrato. B1/B2 podem ser materializados cegamente; produção
continua proibida até ambos serem congelados.

## Gates B1/B2 congelados — RED

Dois verificadores independentes validaram os oito hashes atuais e criaram arquivos
separados sem ler produção, L3 ou testes existentes.

- B1 `tests/update_snapshot_planning_assessment.rs`, SHA-256
  `09b81471d75656164df6f3332ec38c8a624c50ae961619ae61f6336a6a1a91aa`:
  filtro/cardinalidade/ordem, paths hostis, primeiro duplicado, ausências e spy de
  serialização;
- B2 `tests/update_snapshot_execution_assessment.rs`, SHA-256
  `a0c5ba16d856f730b757d24131aed25b2c9548099fbbcf3de8d32e11c8196ef7`:
  dry-run, unreadable, ordem, duplicatas, escrita única, erro exato e continuação;
- `rustfmt --check` dirigido: PASS nos dois;
- compilação: RED pela ausência de `SnapshotUnreadable` e das variantes normativas de
  `SnapshotEntry`/`SnapshotResult`.

O RED está congelado. O confronto C da produção está autorizado a partir deste ponto.

## Confronto C e correção

O confronto encontrou `SnapshotEntry` e `SnapshotResult` como structs com campos
paralelos/`Option`, sem os estados normativos. `execute` usava `filter_map` e removia toda
entrada com `unreadable_reason`, contrariando cardinalidade e observabilidade congeladas.

O commit `e106a38` materializou `SnapshotUnreadable`, `SnapshotEntry` e `SnapshotResult`
como enums públicos comparáveis. `plan` preserva uma ocorrência por V6, usa o primeiro
path integralmente igual e serializa somente `Ready`. `execute` usa `map`, produz um
resultado por entrada, distingue dry-run/escrita/falha/unreadable e continua após `Err`.
Os formatters foram adaptados aos estados tipados. Sete arquivos adicionais mudaram
somente na metadata `@prompt-hash` pelo reparador oficial.

## Primeiro confronto D — bloqueio adicional

- G1–G5: fechados normativamente antes dos gates;
- B1 SHA-256
  `09b81471d75656164df6f3332ec38c8a624c50ae961619ae61f6336a6a1a91aa`:
  3/3 PASS;
- B2 SHA-256
  `a0c5ba16d856f730b757d24131aed25b2c9548099fbbcf3de8d32e11c8196ef7`:
  2/2 PASS;
- RED causal: estados normativos ausentes e unreadable descartado em execução;
- correção: `e106a38`;
- suíte: 628 unitários, 83 fixtures e todos os gates de integração PASS;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- hashes: `Nothing to fix`; `rustfmt` dirigido e `git diff --check`: PASS;
- adversário D: planejamento/execução e arquitetura PASS, mas fechamento `BLOCKED`.

O fluxo real `--update-snapshot --dry-run` usa `format_plan`, que não torna observável o
campo `snapshot`, embora o L0 exija reportar a interface que seria escrita.
`format_results` também classificaria `DryRun` sob “Updated”. Nenhum gate congelado cobre
essa apresentação. Um verificador novo deve criar gate black-box dirigido aos formatters
antes da correção.

## Gate corretivo de apresentação congelado

Um terceiro verificador, sem ler produção ou gates anteriores, validou sete L0. O hash do
passo havia mudado durante o registro provisório do bloqueio; ele recusou corretamente
essa leitura e derivou o oráculo somente dos demais L0 válidos. O passo foi então
resselado na tabela acima sem alterar a expectativa do gate.

`tests/update_snapshot_dry_run_presentation_assessment.rs`, SHA-256
`77c3003ced9d8386fdd66edf029aac97b53fe77e37ce7236614214bf51f0a6ed`:

- motivo distinto de unreadable: 1 PASS;
- `format_plan` torna snapshot hostil observável: RED;
- `format_results(DryRun)` evita “Updated” e mostra snapshot: RED.

O RED de apresentação está congelado; a correção pode começar.

Residual futuro: Git prova arquivos e escopos separados, mas não prova sozinho independência
cognitiva dos verificadores porque ambos os gates foram congelados no mesmo commit com
autor genérico. O registro segregado e o conteúdo não mostram oráculo compartilhado.

## Papéis

- A: adversário somente Assessment/L0 hash-pinned;
- B1: gate de planejamento após saneamento, sem produção;
- B2: identidade e arquivo separados para execução, sem B1/produção/L3;
- C: confronto somente após B1/B2 congelados;
- D: fechamento adversarial de causalidade, arquitetura, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge ou push.
