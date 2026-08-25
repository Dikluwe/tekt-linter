# Passo operacional 0108 — fechamento documental, semântico, fmt e merge

> **Estado:** ESCRITO — não executado  
> **Data:** 2026-08-25  
> **Branch:** `codex/tekt-nucleus-artifact`  
> **Baseline:** `adc7c6de13d97abaa82f65ba066f71c534deb6fd`  
> **Objetivo terminal:** integrar em `master` e encerrar P0104–P0108

## 1. Horizonte finito

Fechar exatamente quatro dívidas:

1. reconciliar documentos que ainda anunciam bloqueios já resolvidos;
2. auditar a suficiência semântica dos 44 prompts individualizados por P0106;
3. quitar o delta global conhecido de `cargo fmt --check` em 26 arquivos;
4. integrar o branch em `master` após todos os gates e resselo.

O passo termina no merge verificado. Não abre auditoria de V16–V25, certificação selada,
novos Núcleos Tekt, redesign funcional ou alteração da arquitetura.

## 2. Fora de escopo

- corrigir achados históricos de complexidade do auto-lint;
- alterar V0–V26, parsers, IR, CLI, camadas ou schema TOML;
- reunificar prompts ou mudar a bijeção prompt⇄código;
- exigir verificador cognitivamente independente retroativamente;
- modificar Bateia, Tekt ou tekt-cargo-dsm;
- atualizar instalação global, fazer push, tag ou release;
- usar formatação para esconder alteração funcional.

## 3. A — baseline e L0 hash-pinned

Criar Assessment 0036 antes de writes e fixar:

| Insumo | SHA-256 |
|---|---|
| manifesto dos 44 owners P0106 | `38b4d76c7749ab4d18b8d30d848ff07c250dcc1f074d5d72d1be54fbd369555f` |
| fechamento P0106 | `2d528395e0822c8409cc673c0bc68d4aefe9e8e416a073de6dc5d82ba50e1817` |
| fechamento P0107 | `5f7e671a5b44acec849af9b5ba1e3a92d3c3e80a06c926609d9ebca7e0389b87` |
| baseline Git | `adc7c6de13d97abaa82f65ba066f71c534deb6fd` |

Congelar também o OID local de `master`, worktree limpo, commits `master..HEAD`, saída
integral de `cargo fmt --check`, documentos com estado obsoleto e SHA-256 dos 44 prompts e
44 códigos do manifesto 0034. Se `master` mudar depois de A, refazer o gate de integração
contra o novo OID; não fazer merge sobre baseline móvel.

## 4. B — reconciliação documental

Revisar sem apagar cronologia:

- `00_nucleo/assessments/0033-nucleus-artifact.md`;
- `00_nucleo/relatorio-p0105-artefato-nucleo-tekt.md`;
- qualquer documento normativo que afirme que P0104/P0106 ainda bloqueia merge.

Manter o diagnóstico histórico em seção datada e adicionar estado atual explícito:
P0104/P0106 foram fechados pelo Assessment 0034/commit `b458714`; a representação TOML e
o parecer pré-merge foram fechados pelo Assessment 0035/commit `adc7c6d`. Não reescrever o
passado como se esses resultados fossem conhecidos em P0105 e não promover
`READY WITH RESIDUAL AUDIT` a garantia formal.

Gate B: busca textual não encontra afirmação ativa e não anotada de que os 13 V15 ou
P0104/P0106 ainda impedem o merge.

## 5. C — auditoria semântica fechada dos 44 prompts

As 44 linhas de `0034-manifest-individualizacao.tsv` são o universo completo. Criar
manifesto 0036-C com uma linha por par:

```text
consumer | owner_prompt | prompt_sha256 | code_sha256 |
responsibility | constraints | observable_criteria | verdict | note
```

### 5.1 Critérios mínimos

Cada prompt deve:

1. nomear um único owner correspondente ao consumer;
2. distinguir sua responsabilidade dos outros 43 owners;
3. preservar camada e direção de gravidade Tekt;
4. declarar restrição que impeça expansão para responsabilidade vizinha;
5. declarar critério observável ou apontar gate/ADR vigente;
6. não contradizer código público, ADR ou classificador 0034;
7. não depender de texto removido sem autoridade substituta;
8. não criar claim compartilhada artificial onde o classificador decidiu zero núcleos.

Não usar comprimento como proxy de qualidade.

### 5.2 Vereditos

| Veredito | Significado | Ação |
|---|---|---|
| `SUFFICIENT` | contrato específico e suficiente | nenhuma edição |
| `ENRICH` | owner correto, falta restrição/critério | ampliar somente o prompt |
| `CONTRADICTION` | diverge de código/ADR/classificador | RED; decidir antes de editar |
| `SPEC-GAP` | falta autoridade para decidir | decisão antes de seguir |

Gates: exatamente 44 linhas/44 prompts; hashes conferidos; V15 zero; cada `ENRICH`
referencia sua linha; zero `CONTRADICTION`/`SPEC-GAP` aberto. Enriquecimentos, se houver,
ficam em um único commit L0. Não alterar código para fazê-lo concordar com prompt compacto;
divergência funcional abre passo futuro fora de P0108.

## 6. D — quitação global de rustfmt

O baseline contém exatamente 26 arquivos divergentes:

```text
01_core/contracts/parse_error.rs
01_core/contracts/prompt_reader.rs
01_core/entities/l1_allowed_external.rs
01_core/entities/violation.rs
01_core/rules/compound_guard.rs
01_core/rules/forbidden_import.rs
01_core/rules/impure_core.rs
01_core/rules/mod.rs
01_core/rules/mutable_state_core.rs
01_core/rules/or_pattern_alternatives.rs
01_core/rules/prompt_drift.rs
01_core/rules/quarantine_leak.rs
01_core/rules/test_file.rs
01_core/rules/wiring_logic_leak.rs
02_shell/mod.rs
03_infra/c_parser.rs
03_infra/cpp_parser.rs
03_infra/elixir_parser.rs
03_infra/go_parser.rs
03_infra/java_parser.rs
03_infra/py_parser.rs
03_infra/ts_parser.rs
03_infra/zig_parser.rs
tests/fixtures/ghost_variant.rs
tests/hardcoded_contextual_value_v21_assessment.rs
tests/prompt_ownership_wiring_assessment.rs
```

Protocolo:

1. congelar o diff pré-fmt em 0036-D;
2. executar `cargo fmt` uma única vez;
3. exigir que somente os 26 paths listados mudem;
4. comparar tokens/AST antes/depois com método congelado antes do write; qualquer delta
   não cosmético é RED;
5. exigir `cargo fmt --check` exit 0;
6. executar suíte completa antes do resselo;
7. manter formatação em commit isolado.

Se rustfmt tocar outro arquivo, classificar e atualizar o inventário antes do commit.

## 7. E — resselo e regressão pré-merge

Depois de B–D commitados:

1. executar `cargo run -- . --fix-hashes --dry-run`;
2. congelar a lista exata de pares, limitada a prompts enriquecidos e códigos formatados;
3. exigir V15/V26 íntegros antes do write;
4. executar resselo real uma vez;
5. repetir dry-run e exigir `Nothing to fix`;
6. auditar que o resselo tocou somente `@prompt-hash`, `Hash do Código` e pins transitivos;
7. executar `cargo fmt --check`, `cargo test`, auto-lint, `git diff --check` e status.

Critérios: fmt exit 0; suíte verde; V1/V5/V7/V15/V26 zero; worktree limpo; Assessment
0036 `READY TO MERGE`, sem RED/SPEC-GAP. Achados históricos V16–V25 são registrados sem
expandir o escopo.

## 8. F — merge controlado em master

O merge é autorizado somente se E estiver integralmente verde:

1. registrar tip final do branch e OID congelado de `master`;
2. trocar para `master` com worktree limpo;
3. integrar `codex/tekt-nucleus-artifact` preservando commits, sem squash;
4. conflito bloqueia merge e reabre A; não resolver automaticamente;
5. após merge, repetir fmt, testes, dry-run, `git diff --check` e status;
6. exigir `Nothing to fix`, testes verdes e worktree limpo;
7. registrar commit de merge e provar ancestralidade com
   `git merge-base --is-ancestor <branch-tip> master`.

Não fazer push, tag, release ou instalação global sem autorização separada.

## 9. RED, gate e SPEC-GAP

| Classe | Exemplos |
|---|---|
| RED | bloqueio documental falso; prompt contraditório; fmt não cosmético; resselo inesperado; teste falha; conflito |
| gate | hash, fixture ou contagem desatualizada com autoridade inequívoca |
| SPEC-GAP | responsabilidade sem autoridade; conflito documental real; baseline de merge móvel |

Todo RED exige correção e repetição. Todo SPEC-GAP exige decisão antes do merge.

## 10. Commits previstos

1. `audit(p0108): freeze final integration surface`
2. `docs(p0108): reconcile resolved historical blockers`
3. `audit(p0108): classify 44 prompt contracts`
4. `docs(p0108): enrich insufficient prompt contracts` — somente se necessário
5. `style(p0108): close repository rustfmt debt`
6. `chore(p0108): reseal final prompt graph`
7. `docs(p0108): declare branch ready to merge`
8. merge commit em `master`

O passo termina quando `master` contém o tip auditado, os gates pós-merge passam e o
worktree está limpo.
