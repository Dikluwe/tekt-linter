# Passo operacional 0112 — mutação dirigida do núcleo L1 puro

> **Estado inicial:** ESCRITO — execução não iniciada  
> **Branch de execução:** `codex/p0112-mutation-l1-pure`  
> **Precondição:** P0111 integrado em `master` e linter do sistema atualizado  
> **Assessment:** 0040  
> **Objetivo terminal:** executar e fechar um universo fixo de 38 mutantes em seis módulos
> L1 puros, saneando todo `TEST-GAP` acionável sem transferir política, I/O ou coordenação
> para o núcleo Tekt.

## 1. Pergunta fechada

Depois do sinal positivo de P0111, verificar se mutação encontra novos buracos nos
classificadores e valores fechados de baixo risco que permanecem no núcleo puro:

1. permissões externas específicas por linguagem;
2. classificação de arquivos alienígenas e arquivos sem teste;
3. contenção de imports da quarentena;
4. diferença entre prompts referenciados e órfãos;
5. reconhecimento de estado mutável em L1.

O passo responde apenas por estes seis arquivos. Não autoriza campanha contínua, mutation
score global nem alegação de certificação.

## 2. Superfície L0 congelada

O comando de inventário com `cargo-mutants 27.1.0` produziu exatamente **38 mutantes**:

| Arquivo | SHA-256 pré-execução |
|---|---|
| `01_core/entities/l1_allowed_external.rs` | `dbbbb809e0c66f7fcbb7a349695037be381352a4d7ed743ce14ffcebd97a9bfd` |
| `01_core/rules/alien_file.rs` | `bd7b31b9c2547205087c7c170a53f8d86a97e465aaf93d1fe0ba3580f6212368` |
| `01_core/rules/test_file.rs` | `6072016519b73d6df8831dc4cfe7cc8f97ee94fd3116e8eda963e338fca86d1f` |
| `01_core/rules/quarantine_leak.rs` | `b50466a01b76c9c4eebdf92fa05685d6022843266b867cc11ac736c933a8ba97` |
| `01_core/rules/orphan_prompt.rs` | `ecba43f507a415762d7a25e8b0bca0aff23d52d632c261bf4f6ad928e52989e4` |
| `01_core/rules/mutable_state_core.rs` | `7ec860d6197e44a0f3dd1bd8f96d97a27f831fa3888035219ae43ff17d6a2382` |
| lista ordenada dos 38 mutantes | `16d066ef9a3cda926b2db4ec04739f1115ffd8800308bc2043b2491648213f35` |

Na abertura da execução, Assessment 0040 deve repetir os hashes e a lista a partir do
`master` pós-P0111. Qualquer diferença é `GATE-DRIFT`: parar, explicar e reescrever o pin;
não executar uma superfície silenciosamente diferente.

Arquivos `layer.rs`, `violation.rs`, `hash_pair.rs` e `parse_error.rs` produziram zero
mutantes no inventário atual. Eles são controle negativo de geração, não módulos
“aprovados” nem evidência de cobertura.

## 3. Fronteira arquitetural Tekt

- L1 continua puro: sem `std::fs`, ambiente, relógio, processo, rede ou escrita.
- Testes podem construir entidades e chamar políticas públicas; não podem adicionar seam
  de infraestrutura ao núcleo apenas para satisfazer a ferramenta.
- L2/L3/L4 não entram no universo e não devem ser modificados.
- Mutante que sugira mover decisão para wiring é `ARCH-RED`, não convite a fazê-lo.
- A autoridade é o prompt owner de cada arquivo, seguida por ADRs vigentes e assessments
  fechados. Teste histórico isolado não supera essas autoridades.

## 4. A — abertura segregada e baseline

1. integrar P0111 em `master` antes de criar o branch;
2. atualizar o binário instalado e provar que corresponde ao `master`;
3. criar `codex/p0112-mutation-l1-pure` sem carregar worktree sujo;
4. criar Assessment 0040 antes de editar produção ou testes;
5. congelar commit-base, versões de Rust/cargo-mutants, hashes dos seis arquivos, lista dos
   38 mutantes, `Cargo.lock`, auto-lint e `fix-hashes --dry-run`;
6. preservar campanhas `mutants.out*` anteriores por nome e SHA-256 antes da rodada.

Baseline deve passar `cargo fmt --check`, `cargo test --all-targets`, ratchet P0109,
auto-lint e `git diff --check`. Falha do baseline é `GATE-DEFECT` e bloqueia mutação.

## 5. B — mapa de autoridade e observações

Antes de executar, materializar `0040-authority-map.tsv` com uma linha por função mutável:

```text
path | function | prompt_owner | observable_contract | existing_gate | blind_spot
```

O mapa deve distinguir:

- retorno booleano fechado;
- cardinalidade, ordem, nível e evidência de `Violation`;
- matriz completa de linguagens/camadas;
- defaults deliberados versus ausência de autoridade.

Cobertura pode orientar a triagem, mas percentual não é gate e zero branches reportado
pela ferramenta é `TOOL-LIMIT`, como em P0111.

## 6. C — rodada cega dos 38 mutantes

Executar uma única rodada com os seis `--file`, `--no-shuffle`, no máximo quatro jobs e
timeout total operacional de 30 minutos. Preservar por SHA-256:

- `mutants.json` e `outcomes.json`;
- `caught.txt`, `missed.txt`, `timeout.txt` e `unviable.txt`;
- comando, versão e exit status.

Nenhuma edição durante C. O universo deve permanecer exatamente 38; acréscimo, perda ou
reordenação inexplicada é `GATE-DRIFT`.

## 7. D — classificação integral

Materializar `0040-mutation-verdicts.tsv` para todo não morto:

```text
mutant_id | path | function | mutation | outcome | reproduction |
semantic_effect | verdict | authority | action | final_outcome
```

Vereditos permitidos:

| Classe | Decisão |
|---|---|
| `TEST-GAP` | comportamento público muda; congelar RED e escrever gate público |
| `PRODUCTION-RED` | baseline contradiz autoridade; congelar antes de corrigir |
| `EQUIVALENT` | provar equivalência no domínio fechado; não pinçar detalhe interno |
| `DEAD-REDUNDANT` | diferença não alcança consumidor; remoção fica em passo próprio |
| `SPEC-GAP` | autoridades não decidem; bloquear correção conveniente |
| `TOOL-LIMIT` | inviável, timeout ou limitação reproduzida do executor |

Cada `MISSED` deve ser repetido serialmente. Divergência paralela/serial é `FLAKY-GATE` e
deve ser reproduzida antes de qualquer classificação semântica.

## 8. E — saneamento dirigido

Para cada `TEST-GAP`:

1. escrever teste que falhe sob o mutante e passe no baseline;
2. observar retorno, diagnóstico ou matriz pública, nunca a linha mutada;
3. cobrir positivo, negativo e fronteira quando a função for booleana;
4. preservar ordem, multiplicidade, path, line, level e evidence quando retornar violações;
5. não modificar produção correta nem prompt para matar mutante;
6. commitar por cluster e repetir serialmente apenas o cluster afetado.

`PRODUCTION-RED`, `SPEC-GAP` e `ARCH-RED` impedem fechamento até decisão explícita.

## 9. F — fechamento e horizonte finito

Após o saneamento:

- repetir todos os `TEST-GAP` originais e exigir `CAUGHT`;
- rodar a lista integral de 38 novamente somente se testes compartilhados puderem alterar
  mais de um cluster;
- executar fmt, suíte completa, ratchet, auto-lint, dry-run de hashes e diff-check;
- registrar custo, mutantes por classe e gaps reais encontrados;
- decidir separadamente se existe sinal suficiente para um P0113. Não criar P0113 por
  inércia.

Estados terminais:

- `BATCH FOUND TEST GAPS — CLOSED`;
- `BATCH FOUND NO ACTIONABLE SURVIVORS`;
- `BATCH FOUND PRODUCTION RED`;
- `BLOCKED` por baseline, arquitetura ou `SPEC-GAP`.

## 10. Commits previstos

1. `audit(p0112): freeze pure L1 mutation batch`
2. `audit(p0112): map authorities and classify survivors`
3. `test(p0112): kill <cluster> mutation gaps` — zero ou mais
4. `docs(p0112): close pure L1 mutation batch`

O passo termina no branch limpo e com Assessment 0040 fechado. Merge e atualização do
linter instalado exigem os gates finais; promoção de mutação para CI periódico permanece
decisão futura baseada em custo e sinal.
