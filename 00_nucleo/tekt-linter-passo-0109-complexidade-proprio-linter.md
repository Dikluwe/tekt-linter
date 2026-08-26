# Passo operacional 0109 — resolver a complexidade histórica do próprio linter

> **Estado:** EXECUTADO — fechado pelo Assessment 0037
> **Data:** 2026-08-25  
> **Branch:** `codex/p0109-self-complexity`  
> **Baseline:** `507fc519236363b12c0379250fddb4ebda18b50e`  
> **Objetivo terminal:** fechar a dívida V16–V25 do auto-lint com saneamento semântico,
> classificação integral e ratchet verificável

## 1. Horizonte finito

Resolver exatamente os achados históricos de complexidade emitidos pelo próprio linter no
baseline:

| Regra | Quantidade | Natureza inicial |
|---|---:|---|
| V16 | 3 | wildcard: 1 warning silencioso e 2 delegações informativas |
| V17 | 2 | guards compostos em `git_refinement` |
| V18 | 0 | fechado por ausência |
| V19 | 68 | métrica informativa de alternativas condensadas |
| V20 | 17 | métrica informativa de profundidade de padrão |
| V21–V25 | 0 | fechadas por ausência |

Universo: 90 diagnósticos em 23 arquivos. O passo termina com cada ocorrência original
classificada, todo RED saneado e um ratchet que proíba regressão. Não exige zerar métricas
V19/V20 quando a forma condensada for a representação semântica mais clara.

Os achados V2 e V3 presentes na execução completa não são complexidade V16–V25 e ficam
fora deste passo. Devem permanecer visíveis no relatório final, sem serem reclassificados
ou ocultados.

## 2. Princípios arquiteturais

1. Respeitar as camadas Tekt e a direção de gravidade; nenhuma extração pode criar import
   ascendente ou deslocar decisão de L1 para adapters.
2. V16/V17 são sinais acionáveis. V19/V20 são instrumentos de observação, não ordens para
   duplicar braços ou introduzir helpers artificiais.
3. Refatoração deve preservar domínio, ordem, multiplicidade, evidência, severidade,
   paths, mensagens e comportamento fail-closed.
4. Uma aceitação residual precisa explicar por que a condensação representa equivalência
   real; “muito trabalhoso” não é justificativa.
5. Não alterar as próprias regras V16–V25 para fazer o auto-lint ficar verde. Qualquer
   falso positivo alegado é `SPEC-GAP` e exige passo próprio de calibração.
6. Não usar `allow`, exclusão de arquivo, redução de severidade ou alteração de config
   como substituto de saneamento.

## 3. A — congelamento L0 hash-pinned

Antes de qualquer write funcional, criar Assessment 0037 e congelar:

| Insumo | SHA-256/OID |
|---|---|
| baseline Git | `507fc519236363b12c0379250fddb4ebda18b50e` |
| saída integral do auto-lint | `0cae776bab3c3ec11a13d37fc59a7232a8fed1c9dd623b5c95e5ae38084d5cd0` |
| ADR-0016 | `abdf38e2c75b7f3a113a50db61913320d08f510e5c7f377efde020af1c19ffd2` |
| prompt V16 | `2414fcef861fc426d0fc25555eb00369b5be670ddab911b49723acaae5b450de` |
| prompt V17 | `d5ef806723eea38137c8c71ace80057cd7c8e79aa7d4ef7696fa3b72b9ea1a98` |
| prompt V19 | `91c409539ea603c2e4ae1aa4932e6bedddb209991652b4a345dbbe7e3b159620` |
| prompt V20 | `b4a4acabc362920561bec12f95ddcb99a0fcf68c5d53759b9ff3e6ab1d5060e5` |

Materializar `0037-self-complexity.tsv`, uma linha por diagnóstico, com:

```text
id | rule | level | path | line | evidence_sha256 | semantic_cluster |
initial_verdict | final_verdict | rationale | test_or_oracle | commit
```

Gates A: exatamente 90 linhas; contagens 3/2/68/17; exatamente 23 paths; worktree limpo;
`cargo fmt --check`, `cargo test` e `fix-hashes --dry-run` verdes. Mudança do baseline ou
das contagens antes do primeiro commit reabre A.

## 4. B — protocolo segregado de classificação

Classificar sem editar produção. A classificação não pode depender somente da mensagem do
linter: deve ler o contexto local, o prompt owner, os testes vigentes e o contrato público
afetado.

| Veredito | Significado | Ação |
|---|---|---|
| `REFACTOR` | forma amplia risco ou esconde decisão | criar gate RED e sanear |
| `ACCEPT-EQUIVALENT` | alternativas são semanticamente idênticas | manter e justificar |
| `ACCEPT-BOUNDARY` | padrão profundo modela explicitamente uma fronteira fechada | manter e justificar |
| `RULE-RED` | regra diagnosticou fora do contrato normativo | congelar; não mudar produção |
| `SPEC-GAP` | L0 não decide se a forma é aceitável | bloquear cluster até decisão |

Um avaliador B produz a classificação e um confronto C tenta refutá-la antes de qualquer
saneamento. C deve procurar perda de variante, alteração de short-circuit, mudança de
ownership, conversão de erro em ausência, duplicação divergente e helpers que atravessem
camadas.

Gates B/C: 90/90 com veredito e justificativa; zero `SPEC-GAP` aberto; todo
`ACCEPT-*` possui razão positiva e todo `REFACTOR` possui teste/oráculo observável.

## 5. D — lotes de saneamento

Executar em commits separados e repetir os testes direcionados após cada lote.

### D1 — V16 e V17 acionáveis

- eliminar o wildcard silencioso de `01_core/rules/unsourced_constant.rs` com enumeração
  total ou decisão explícita;
- avaliar as duas delegações V16: manter somente se C provar que são fronteiras totais e
  ruidosas; caso contrário, explicitar variantes;
- decompor os dois guards compostos de `03_infra/git_refinement.rs` sem alterar ordem de
  I/O, short-circuit, classificação de erro ou política fail-closed.

Critério: zero warning V16/V17. V16 informativo só pode permanecer como
`ACCEPT-EQUIVALENT`/`ACCEPT-BOUNDARY` documentado.

### D2 — V19 em núcleo e shell

Tratar primeiro `01_core/**` e `02_shell/**`. Separar braços somente quando cada
alternativa tiver decisão, evidência ou evolução independente. Manter or-pattern quando
as alternativas compartilham integralmente o mesmo contrato.

Critério: nenhum `REFACTOR` V19 pendente; ausência de duplicação textual divergente; testes
de permutação, ordem e multiplicidade preservados.

### D3 — V19/V20 nas fronteiras de infraestrutura

Agrupar por cluster, não por arquivo: parsers, Git, config, walker, prompt I/O e snapshots.
Para padrões profundos, preferir tipos/intermediários nomeados somente quando eles reduzem
ambiguidade real. Não esconder nesting por helper de uma linha.

Critério: nenhum `REFACTOR` V19/V20 pendente; classes de erro, bytes de path, limites de
budget e determinismo preservados.

### D4 — Lab

Classificar `lab/refinement_probe.rs` separadamente. Lab não pode contaminar contratos de
produção nem ser usado para justificar exceção em L1–L4.

## 6. E — ratchet sem transformar métrica em proibição

Adicionar um gate de regressão versionado que compare diagnósticos canônicos por
`rule + path + evidência estrutural`, não por número de linha. O gate deve:

1. falhar para qualquer novo V16/V17 acionável;
2. falhar se reaparecer ocorrência removida;
3. falhar se V19/V20 crescerem sem entrada classificada no manifesto;
4. aceitar deslocamento de linha e formatação sem atualizar snapshot;
5. exigir justificativa para adicionar `ACCEPT-*`;
6. não interferir na análise de repositórios consumidores.

Se o linter não oferece identidade estrutural estável suficiente, isso é `SPEC-GAP`; não
criar snapshot frágil de texto integral ou linhas absolutas.

## 7. F — resselo e fechamento

Depois dos lotes:

1. executar `fix-hashes --dry-run` e congelar os pares planejados;
2. resselo real somente após confirmar que os pares correspondem aos owners editados;
3. repetir dry-run e exigir `Nothing to fix`;
4. executar `cargo fmt --check`, `cargo test`, gates dirigidos V16–V25,
   `git diff --check` e status;
5. executar o auto-lint completo e registrar separadamente V2/V3 fora de escopo;
6. confrontar o manifesto: 90 ocorrências com destino final, zero `REFACTOR`,
   `RULE-RED` ou `SPEC-GAP` aberto;
7. executar adversário final contra diff, testes, resselo e ratchet.

Estado terminal: `READY WITH EXPLICIT METRIC BASELINE` se restarem somente
`ACCEPT-EQUIVALENT`/`ACCEPT-BOUNDARY`; `READY` se V16–V20 chegarem legitimamente a zero.
Ambos fecham a dívida. “Exit 0” obtido por supressão não fecha o passo.

## 8. RED, gate e SPEC-GAP

| Classe | Exemplos |
|---|---|
| RED de produção | wildcard silencioso; guard muda semântica; match perde variante; erro vira ausência |
| RED do gate | identidade baseada em linha; contagem inclui fixture/exclusão errada; snapshot aceita diagnóstico novo |
| gate | hash de prompt/código desatualizado após saneamento autorizado |
| SPEC-GAP | equivalência não decidida; regra diverge do ADR; identidade estrutural instável |

Todo RED é congelado antes da correção e repete os gates do cluster. Todo SPEC-GAP exige
decisão explícita antes de continuar aquele cluster.

## 9. Fora de escopo

- corrigir V2/V3 ou qualquer regra estrutural fora de V16–V25;
- recalibrar detectores V16–V25 para reduzir a própria contagem;
- modificar Bateia, Tekt, Typst ou tekt-cargo-dsm;
- criar modo de certificação/selagem;
- alterar arquitetura pública, schema TOML ou bijeção prompt⇄código;
- push, tag, release, instalação global ou merge em `master`.

## 10. Commits previstos

1. `audit(p0109): freeze self-complexity baseline`
2. `audit(p0109): classify historical complexity findings`
3. `refactor(p0109): close actionable wildcard and guard findings`
4. `refactor(p0109): reduce core and shell decision complexity`
5. `refactor(p0109): reduce infrastructure decision complexity`
6. `test(p0109): ratchet accepted complexity baseline`
7. `chore(p0109): reseal prompt graph`
8. `docs(p0109): close historical self-complexity audit`

O merge requer autorização em passo posterior. P0109 termina no branch limpo, com o
Assessment 0037 fechado e a superfície residual explicitamente finita.
