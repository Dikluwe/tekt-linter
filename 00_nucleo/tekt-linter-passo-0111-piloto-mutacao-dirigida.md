# Passo operacional 0111 — piloto dirigido de testes de mutação

> **Estado:** ESCRITO — não executado
> **Data:** 2026-08-25
> **Branch:** `codex/p0111-mutation-pilot`
> **Baseline:** `b2a2826e540a556081476918f98cb85c5dfe21be`
> **Objetivo terminal:** determinar, num universo de 192 mutantes, se os testes atuais
> deixam falhas semanticamente relevantes sobreviverem

## 1. Pergunta e horizonte finito

Responder experimentalmente:

> Testes de mutação dirigidos ainda encontram falhas reais na capacidade dos testes do
> linter de perceber implementações incorretas?

O universo inicial contém exatamente 192 mutantes gerados por `cargo-mutants 27.1.0` em
cinco arquivos:

```text
01_core/entities/refinement_seal.rs
01_core/rules/forbidden_import.rs
01_core/rules/prompt_drift.rs
02_shell/fix_hashes.rs
03_infra/refinement_snapshot.rs
```

O passo termina depois de uma execução baseline, uma rodada integral, classificação de
100% dos sobreviventes/timeouts/inviáveis e, somente para `TEST-GAP` inequívoco, uma rodada
de saneamento e reteste dirigido. Não expande para o restante do repositório.

## 2. Por que estes cinco componentes

- `refinement_seal`: matriz pequena de aceitação positiva/negativa/unknown, crítica para
  não certificar resultado errado;
- `forbidden_import`: direção de gravidade Tekt e evidência de inversão;
- `prompt_drift`: bijeção e comparação de hash;
- `fix_hashes`: planejamento, ordem, colisão, rollback e resultado transacional;
- `refinement_snapshot`: loader fechado, duplicatas, budgets e classes de erro.

Regras e parser Rust já receberam campanhas históricas extensas P0054–P0062. Repeti-las
não pertence ao piloto. Os componentes escolhidos concentram contratos adicionados ou
fortalecidos depois daquele selo histórico.

## 3. O que o piloto não afirma

- mutation score não é prova formal nem certificação;
- cobertura de linha não é suficiência semântica;
- mutante sobrevivente não é automaticamente bug;
- mutante morto não prova que o teste correto o matou;
- zero sobreviventes não autoriza claim universal sobre o linter;
- CRAP não será gate neste passo;
- não alterar CI global, release, versão ou instalação do sistema;
- não fazer merge, push ou tag.

## 4. A — congelamento L0 hash-pinned

Criar Assessment 0039 antes de qualquer teste novo ou mudança funcional e fixar:

| Insumo | SHA-256/OID |
|---|---|
| baseline Git | `b2a2826e540a556081476918f98cb85c5dfe21be` |
| lista dos 192 mutantes | `f7cb4b6708c576eb40aa513064bc2ec02a8d916382ed4a43acad77db1fb987d8` |
| `refinement_seal.rs` | `01bce52ec024c3ae52473b2e3339bd4f99d93beafc040d602d8e6bdd59a7e015` |
| `forbidden_import.rs` | `a6f19cd55547de8bfb3961c6741b25a1cbdb3d8d13f7d7904e9ce0574d30eff3` |
| `prompt_drift.rs` | `31fe43b1c7dafc877bb7016b3b3711c3a37dd92ff8dd8044009640cf0a93c7a6` |
| `fix_hashes.rs` | `d83fe7cf0a4f0d93dd080b6e39130791aea35743fd71f04a0baffdcfb320b6d4` |
| `refinement_snapshot.rs` | `e482661b43557c3ca979e5170e151b2cf218013eb7b1606a1a89e06d3e14d02e` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

Congelar ainda:

- `cargo-mutants 27.1.0` e `cargo-llvm-cov 0.8.7`;
- worktree limpo, `cargo fmt --check`, suíte completa, auto-lint e hashes;
- hashes e conteúdo de `mutants.out` e `mutants.out.old` preexistentes.

`cargo-mutants` rotaciona esses diretórios. Preservar ambos antes da rodada em arquivo
hash-pinned fora de `mutants.out*`; não apagar nem sobrescrever evidência histórica.

Gate A: a repetição de `cargo mutants --list` produz as mesmas 192 linhas. Mudança da lista
reabre A.

## 5. B — cobertura como mapa, não meta

Executar `cargo llvm-cov` sobre a suíte integral e extrair somente os cinco arquivos. Para
cada função, registrar regiões/branches não exercitados e o SHA-256 do JSON bruto.

Não adicionar teste ainda. Cobertura serve para formular hipóteses, sem meta percentual e
sem declarar código coberto como correto. CRAP pode ser calculado apenas como ranking
exploratório se a ferramenta fornecer complexidade de forma reproduzível; ausência dessa
medida não bloqueia o piloto.

Saída B: `0039-coverage-map.tsv` com:

```text
path | function | regions_total | regions_covered | branches_total | branches_covered |
risk_note
```

## 6. C — rodada de mutação cega

Executar exatamente:

```text
cargo mutants -j 4 --no-shuffle --no-times \
  --file 01_core/entities/refinement_seal.rs \
  --file 01_core/rules/forbidden_import.rs \
  --file 01_core/rules/prompt_drift.rs \
  --file 02_shell/fix_hashes.rs \
  --file 03_infra/refinement_snapshot.rs
```

Se `--no-times` não existir na versão instalada, registrar `GATE-DEFECT`, remover somente
essa opção e manter todos os demais argumentos. Não trocar arquivos, regex ou lista após
ver resultados.

Limites:

- no máximo 4 jobs;
- no máximo 192 mutantes nesta rodada;
- timeout total operacional de 60 minutos;
- nenhuma edição durante a execução;
- saída integral, `mutants.json`, `outcomes.json`, `caught.txt`, `missed.txt`,
  `timeout.txt` e `unviable.txt` preservados por hash.

Falha do baseline não é mutante sobrevivente: é `GATE-DEFECT` e bloqueia C.

## 7. D — classificação integral antes de corrigir

Materializar `0039-mutation-verdicts.tsv`, uma linha para cada mutante não morto:

```text
mutant_id | path | function | mutation | outcome | reproduction |
semantic_effect | verdict | authority | action | final_outcome
```

Vereditos permitidos:

| Veredito | Significado | Ação |
|---|---|---|
| `TEST-GAP` | mutação muda comportamento contratual e testes não percebem | congelar RED e criar gate |
| `EQUIVALENT` | programa observável é equivalente neste domínio fechado | justificar com prova local |
| `DEAD-REDUNDANT` | código não afeta comportamento ou revela redundância | avaliar remoção separadamente |
| `SPEC-GAP` | autoridades não decidem se a diferença é correta | bloquear correção |
| `TOOL-LIMIT` | mutante é inviável/timeout/artefato da ferramenta | reproduzir e documentar |

Cada `MISSED` deve ser reproduzido individualmente por regex exata em execução limpa. Uma
segunda falha idêntica confirma o sobrevivente; resultado divergente é `FLAKY-GATE`, não
`TEST-GAP`. Timeouts e inviáveis são inventariados, não contam como mortos.

Classificação deve confrontar prompt owner, ADR, teste existente e efeito observável. Não
usar apenas intuição sobre o diff mutado.

## 8. E — saneamento somente dos TEST-GAPs

Para cada `TEST-GAP` confirmado:

1. escrever teste RED que falha sob o mutante e passa no baseline;
2. preferir fixture/propriedade que observe contrato público, não linha interna;
3. não modificar produção se a produção baseline já está correta;
4. se a mutação revelar bug real da produção, congelar separadamente como
   `PRODUCTION-RED` antes de corrigir;
5. manter um commit por cluster semântico;
6. repetir somente os mutantes do cluster após cada commit.

Não escrever testes para matar `EQUIVALENT`, `DEAD-REDUNDANT` ou `TOOL-LIMIT`. Não ajustar
prompts retroativamente para justificar produção.

## 9. F — repetição e critérios de resposta

Depois de E:

- repetir todos os `TEST-GAP` originais e exigir `CAUGHT`;
- executar uma segunda rodada integral somente se E mudou teste compartilhado capaz de
  afetar outros mutantes; caso contrário, não gastar novamente os 192 casos;
- rodar fmt, suíte completa, auto-lint, ratchet P0109, dry-run de hashes e diff-check;
- confrontar cobertura B antes/depois apenas como explicação, sem gate percentual;
- executar adversário final sobre sobreviventes classificados e alegações.

O piloto responde “sim, encontrou falhas” se houver ao menos um `TEST-GAP` ou
`PRODUCTION-RED` confirmado. Responde “não neste universo” se todos os não mortos forem
equivalentes, redundantes ou limites reproduzidos da ferramenta. Não generalizar além dos
cinco arquivos.

## 10. RED, gate e SPEC-GAP

| Classe | Exemplos |
|---|---|
| RED de produção | mutante expõe baseline que já viola prompt/ADR |
| RED de teste | mutante muda veredito, erro, rollback ou bytes e sobrevive |
| RED do gate | baseline falha; diretório histórico sobrescrito; lista muda durante C |
| gate | hash/fixture desatualizado por teste autorizado |
| SPEC-GAP | mudança observável sem autoridade para decidir o resultado correto |

Todo RED é congelado antes da correção. SPEC-GAP não pode ser transformado em teste por
conveniência.

## 11. Estado terminal e commits previstos

Estados terminais:

- `PILOT FOUND TEST GAPS — CLOSED`: gaps encontrados, saneados e retestados;
- `PILOT FOUND PRODUCTION RED`: bug real exige saneamento e fechamento antes de concluir;
- `PILOT FOUND NO ACTIONABLE SURVIVORS`: nenhum gap acionável neste universo;
- `BLOCKED`: baseline, ferramenta ou SPEC-GAP impede classificação completa.

Commits previstos:

1. `audit(p0111): freeze mutation pilot surface`
2. `audit(p0111): map directed coverage`
3. `audit(p0111): classify mutation survivors`
4. `test(p0111): kill <cluster> mutation gaps` — zero ou mais
5. `docs(p0111): close directed mutation pilot`

O passo termina no branch limpo com Assessment 0039 e todos os não mortos classificados.
Merge e promoção a gate periódico exigem decisão posterior baseada no custo e no sinal do
piloto.
