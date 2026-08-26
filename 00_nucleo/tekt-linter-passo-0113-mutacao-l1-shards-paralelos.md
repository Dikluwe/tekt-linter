# Passo operacional 0113 — sanitização ampla de regras L1 em shards paralelos

> **Estado inicial:** ESCRITO — execução não iniciada
> **Branch acumulador:** `codex/p0112-mutation-l1-pure`
> **Estratégia de integração:** acumular P0112–P0113; sem merge ou reinstalação entre
> passos exclusivamente documentais
> **Assessment guarda-chuva:** 0041
> **Objetivo terminal:** executar e fechar 144 mutantes em 16 regras L1 puras, divididos
> em cinco shards causalmente independentes, com no máximo três execuções simultâneas e
> classificação/fechamento por shard antes da composição final.

## 1. Pergunta fechada

P0111 encontrou 57 lacunas acionáveis; P0112 matou todos os 22 mutantes compiláveis de um
lote simples. P0113 procura sinal nas regras L1 de risco médio, sem avançar ainda para
filesystem, subprocessos, parsers L3 ou wiring de execução.

O universo contém exatamente 144 mutantes. O passo não mede “validade global”, não define
score mínimo e não cria campanha infinita. Um eventual P0114 exige decisão posterior.

## 2. Shards congelados

| Shard | Tema | Arquivos | Mutantes | SHA-256 da lista |
|---|---|---|---:|---|
| S1 | linhagem e propriedade de prompt | `prompt_header`, `prompt_stale`, `multi_prompt_header` | 32 | `fc11fde2de1888dffbcc34aed0f0e5c77061184e21136bc4db331d422fb17d31` |
| S2 | projeções e relações semânticas | `dangling_contract`, `context_erasure`, `decision_ownership`, `infrastructure_error`, `semantic_field_loss` | 15 | `c2f26f19ec04cb601b7b767ca5c6420ece76fcc6376e62fee55b02233281c40d` |
| S3 | tipos externos em contratos | `external_type_in_contract` | 30 | `06bd4f529e22c14b22d8597395e36a4f2ffff2ed404a795435367f3b60a2672f` |
| S4 | padrões, limites e profundidade | `deep_pattern_nesting`, `compound_guard`, `range_pattern`, `or_pattern_alternatives` | 35 | `12683903693d10a66432545f60954cf0a04b3b0ab279404b0b63590074f68cf7` |
| S5 | fronteiras de pureza e wiring | `impure_core`, `pub_leak`, `wiring_logic_leak` | 32 | `7fdb07f0b0d23d5e67aef4add5a5222fc599fc9476d8a155f864eb30afabe3ce` |

Contagem total: **144**. As listas foram geradas em `aad21c3` por `cargo-mutants 27.1.0`.
Na execução, Assessment 0041 deve repetir contagem, ordem, hashes das listas, fontes,
prompts owner e `Cargo.lock`. Diferença é `GATE-DRIFT`, nunca atualização silenciosa.

### Hashes das fontes

| Arquivo | SHA-256 |
|---|---|
| `prompt_header.rs` | `2f6bc4879206868ad12b156a3a8726d5203567843748a1f310858d17de14efba` |
| `prompt_stale.rs` | `050a4173ae5e0d309d7b4bb7763010678b03e88f542ada15adb12509a2b32091` |
| `multi_prompt_header.rs` | `c66b3c31cf2e7e8a83c1f0c0e065a13c0610f54a490b8fcaeceb0bd0b78b4514` |
| `dangling_contract.rs` | `b26f111661dc12ad3d42f08e8d673cc164d64c0b870e364c8be0e1e458f98351` |
| `context_erasure.rs` | `bd75c60264f083f50105ea7e6902302968b67fdfa305db31940300e5756b6f70` |
| `decision_ownership.rs` | `51dc9421df6cd1273be547f645e94fadd8824eb92fc29f864c75940df5c7bf6f` |
| `infrastructure_error.rs` | `03fdbacd6541a5ff737504a43f8b2b46dae126014cd2cadc8c51d704656b7298` |
| `semantic_field_loss.rs` | `0020d4f71607951d0a867e294838d94b86c8477960cbc44788b164847ab4b0c0` |
| `external_type_in_contract.rs` | `cef2e1e0873afa21d2c37ae8f23dbfa076086273dfb21ccbae5d5f79b95c0c43` |
| `deep_pattern_nesting.rs` | `9b32c9892f59ab319495868cd0968c63f90ab147d343853886827f127db923aa` |
| `compound_guard.rs` | `fdea0d4734a9bd707c97f6e99cfdfce57f3b6306802221453a1436c1335846b1` |
| `range_pattern.rs` | `750969e0b6c789eee94a9dd6120328baeb68923a3cbf71417103a4764d0f5f53` |
| `or_pattern_alternatives.rs` | `2579227c98b7b4274a08b65844a8f66f4166d650bb320f9972cf14929ccebb99` |
| `impure_core.rs` | `7b6569f99f908c21b139e381b332e2c99ebf9e9c463b8d1415ee4585e8f1bfb9` |
| `pub_leak.rs` | `608a0202a5800935e201f061612469c92101ae6aae1d5d75346eeafdfb69948d` |
| `wiring_logic_leak.rs` | `6a52b84dee54301fed14c0a1db66ae8b3784237f9106d9111a186645b397473f` |

## 3. Arquitetura Tekt e paralelismo seguro

- Todos os alvos são L1 e permanecem puros; L2/L3/L4 não podem receber política para
  facilitar teste.
- Cada shard roda em worktree Git próprio, criado do mesmo commit-base imutável.
- Cada worktree usa `CARGO_TARGET_DIR` e diretório `mutants.out` próprios. Compartilhar
  source tree, target ou saída entre processos é `GATE-DEFECT`.
- No máximo três shards simultâneos. S4, o maior e com mais aritmética de limites, não deve
  compartilhar slot com outra execução pesada se houver pressão de recursos.
- Shards não editam o branch acumulador. Produzem somente assessment, manifesto e commits
  próprios; a composição ocorre depois do fechamento individual.
- Um `RED` em um shard não contamina classificação dos demais, mas bloqueia a composição
  final até ser congelado e saneado.

O paralelismo reduz tempo de parede, não reduz exigência de reprodução serial dos
survivors nem autoriza classificação coletiva.

## 4. A — abertura e baseline comum

1. continuar no branch acumulador sem merge/reinstalação, pois P0112 não alterou binário
   ou testes;
2. criar Assessment 0041 antes de qualquer execução;
3. congelar commit-base, worktree limpo, ferramentas, `Cargo.lock`, binário instalado,
   auto-lint, ratchet, fmt, suíte e `fix-hashes --dry-run`;
4. preservar todas as campanhas `mutants.out*` por nome e manifesto SHA-256;
5. repetir as cinco listas e os 16 hashes de fonte;
6. criar worktrees `p0113-s1` a `p0113-s5` em branches temporários `codex/p0113-sN`.

Falha comum de baseline bloqueia todos os shards. Falha exclusiva de worktree bloqueia
somente o shard e é `GATE-DEFECT`.

## 5. B — autoridade por shard

Cada shard materializa `0041-sN-authority-map.tsv` antes da mutação:

```text
path | function | prompt_owner | observable_contract | existing_gate | blind_spot
```

Observações mínimas:

- S1: presença/ausência, propriedade biunívoca, delta como multiconjunto, ordenação e
  diagnóstico global;
- S2: projeção exata de erro, nível, cardinalidade e preservação de evidência;
- S3: normalização de imports, identidade de tipo, guards e índices de caminho;
- S4: máximos inclusivos, profundidade, exceções e precedência de contexto;
- S5: matrizes de camada, pureza, ports, adapter structs e multiplicidade de imports.

Prompt owner e ADR vigente têm precedência sobre teste histórico. Autoridade ausente é
`SPEC-GAP`; não se inventa comportamento para matar mutante.

## 6. C — campanhas cegas paralelas

Cada shard executa somente seus `--file`, com `--no-shuffle`, no máximo dois jobs internos
quando houver três shards simultâneos, e timeout individual de 30 minutos. Preservar:

- comando, versão, commit-base e worktree;
- `mutants.json`, `outcomes.json`, `caught.txt`, `missed.txt`, `timeout.txt`,
  `unviable.txt` e logs;
- SHA-256 de todos os artefatos e exit status.

Nenhuma edição durante C. O número de mutantes deve ser 32/15/30/35/32 exatamente.

## 7. D — reprodução e classificação segregadas

Cada não morto ganha uma linha em `0041-sN-mutation-verdicts.tsv`:

```text
mutant_id | path | function | mutation | outcome | serial_reproduction |
semantic_effect | verdict | authority | action | final_outcome
```

Classes permitidas:

| Classe | Tratamento |
|---|---|
| `TEST-GAP` | congelar RED, escrever gate público e repetir o cluster |
| `PRODUCTION-RED` | congelar divergência da autoridade antes de corrigir |
| `EQUIVALENT` | provar equivalência no domínio fechado; não pinçar implementação |
| `DEAD-REDUNDANT` | registrar redundância; remoção exige passo separado |
| `SPEC-GAP` | bloquear saneamento por conveniência |
| `TOOL-LIMIT` | reproduzir inviável, timeout ou limitação do executor |
| `ARCH-RED` | rejeitar solução que desloque política para camada errada |

Todo `MISSED` é repetido com `-j1` no worktree do shard. Divergência é `FLAKY-GATE`.

## 8. E — saneamento e commits por módulo

Somente depois de D:

1. criar teste RED contra contrato público;
2. preferir matriz/propriedade que mate um cluster causal, sem casar linha interna;
3. preservar path, line, level, evidence, ordem e multiplicidade integralmente;
4. não editar produção correta ou prompt para elevar score;
5. repetir serialmente o cluster e atualizar o manifesto;
6. fechar cada shard em commit independente.

Se dois shards precisarem editar o mesmo teste compartilhado, eles deixam de ser
independentes: serializar a composição e repetir ambos antes do fechamento.

## 9. F — composição no branch acumulador

Depois de todos os shards fechados:

1. confrontar assessments e hashes antes de trazer commits;
2. compor na ordem S1→S5, sem merge octopus;
3. após cada composição, executar testes dirigidos dos módulos afetados;
4. ao final, executar uma única suíte completa, fmt, ratchet, auto-lint, dry-run de hashes
   e diff-check;
5. produzir resumo 0041 com custo por shard, `CAUGHT/MISSED/UNVIABLE/TIMEOUT`, gaps reais,
   equivalentes e qualquer mudança funcional;
6. manter o branch sem merge se só houver evidência documental; integrar quando houver
   volume acumulado relevante ou mudança de produção/testes.

## 10. Estados terminais

Cada shard termina em:

- `SHARD FOUND TEST GAPS — CLOSED`;
- `SHARD FOUND NO ACTIONABLE SURVIVORS`;
- `SHARD FOUND PRODUCTION RED`;
- `BLOCKED`.

P0113 termina somente quando os cinco shards estiverem terminais e compostos:

- `CAMPAIGN CLOSED — READY FOR ACCUMULATED INTEGRATION`;
- `CAMPAIGN CLOSED — KEEP ACCUMULATING` quando não houver mudança funcional/testes;
- `BLOCKED` se qualquer `SPEC-GAP`, `ARCH-RED` ou RED não saneado permanecer.

## 11. Commits previstos

1. `audit(p0113): freeze parallel L1 mutation campaign`
2. `audit(p0113-sN): classify <tema> mutation shard` — cinco commits
3. `test(p0113-sN): kill <cluster> mutation gaps` — zero ou mais
4. `docs(p0113-sN): close <tema> mutation shard` — cinco commits
5. `docs(p0113): close parallel L1 mutation campaign`

O passo não promove mutação a CI e não escreve P0114 automaticamente.
