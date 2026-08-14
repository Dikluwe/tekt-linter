# Laudo 0061 — excluir `#[cfg(test)]` da gravidade (V3/V9/V14), opção `check_test_imports`

**Onde roda**: cruza linter (clone do 0052) e lente (`tekt-cargo-dsm`). Continuação de 0058–0060.
**Criado em**: 2026-06-09
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/excluir-teste-gravidade.md`](prompts/excluir-teste-gravidade.md)
**Camadas tocadas**: L1 (`Import.is_test_origin`; guard em V3/V9/V14), L3
(`rs_parser` marcação de origem; `config.check_test_imports`), L4 (`main.rs`
threading), L2 (`cli` emit), e o oráculo (projeção simétrica).
**Primeira mudança de POLÍTICA de regra** da série — o que a regra *deve* afirmar,
não completude de resolvedor.

---

## A decisão

A gravidade (V3/V9/V14) é uma afirmação sobre o **grafo de produção**. Código
`#[cfg(test)]` é removido do build de release, então uma aresta que cruza camadas
**só em teste** não corrompe a gravidade entregue. A lente (grafo do compilador, sem
`--cfg test`) **exclui teste por construção**; o linter (textual) via tudo — o 0060
expôs isso como `lente_app→lente_infra` num `#[cfg(test)]` (só-linter 0→1).

**Decisão**: V3/V9/V14 **excluem `#[cfg(test)]` por padrão**; `check_test_imports`
(default **`false`**) reactiva a verificação em código de teste.

## O princípio do default (`false`)

Uma opção que **afrouxa** uma regra é segura; uma que a **aperta**, não.
`check_test_imports` aperta (ligada, o linter marca mais), então o **default é o
comportamento correcto** (excluir teste) e a opção é o extra — nunca o contrário.
Mesma regra de polegar do `allow_adapter_structs`. Isto torna explícito o grafo de
cada regra: V2 fala do grafo de teste (há cobertura?), V3/V9/V14 do de produção.

## A mudança

### A — marcar origem (`Import.is_test_origin`)
`collect_imports` (use/extern) **e** `collect_path_refs` (path-refs do 0060) marcam
cada import. **Descoberta de grammar**: o `#[cfg(test)]` é um `attribute_item`
**irmão** que decora o item seguinte — **não** filho do `mod_item` (verificado por
dump de AST). Logo `for_each_child_in_test_scope` propaga o escopo por **item
pendente** (o atributo marca o próximo não-atributo), não por bloco; o `#![cfg(test)]`
interno cai na mesma via (`is_cfg_test_attribute` casa `inner_attribute_item`, e
atributos internos vêm sempre no início). Reusa o critério de cfg(test) do
`check_cfg_test` (V2) — não reinventa.

> Esta foi a armadilha real: a 1ª versão marcava todos os irmãos do nível, o que pôs
> o `use serde` de **produção** do `v14_fail` como test-origin (V14 sumiu). O
> `v14_fail` mordeu o bug; o conserto (pendente-por-item) o fechou, e há regressão
> dedicada (`production_use_not_marked_when_cfg_test_sibling_mod_present`).

### B — config
`check_test_imports: Option<bool>` em `config.rs`, default `None` ⇒ false.

### C — guard nas regras
V3/V9/V14 ganham `.filter(|i| check_test_imports || !i.is_test_origin)` — um **guard**,
não lógica nova. Threading: `config` → `main.rs` (`unwrap_or(false)`) →
`run_pipeline` → `run_checks` → as três regras.

### D — oráculo (projeção simétrica)
`--emit-resolution` passa a emitir `is_test_origin` (emite tudo, marcado — segue
verdadeiro). O **harness do oráculo** remove as arestas test-origin **simetricamente**,
igual a lente as exclui por construção. No default, `só-linter` volta a **0**.

## Fixtures bite-proof (`tests/fixtures.rs`)

A MESMA aresta proibida L1→L3 (`corelib`→`infra`), dois modos:
- **`vtest_default_pass`** (`use infra::Thing` em `#[cfg(test)] mod`, sem opção) → `[]`.
- **`vtest_on_fail`** (mesma fonte + `check_test_imports = true`) → `[V3]`.

O par é o **bite-proof dos dois lados**: só o flag difere. Cobertas as **duas vias**:
- **`vtest_pathref_default_pass`/`vtest_pathref_on_fail`** — path-ref (0060) dentro de
  `#[cfg(test)] fn`, sem `use` → `[]` / `[V3]`.
- **Não-regressão de produção**: **`vtest_prod_fail`** — a MESMA aresta L1→L3 **fora**
  de teste segue `[V3]` (produção nunca é test-origin; a mudança não a toca).

Mais 6 testes unitários em `rs_parser.rs` (marcação use/path-ref/inner-attr; produção
não marcada com mod cfg(test) irmão presente) e o guard dos dois modos em
`forbidden_import.rs`. Suíte: **492 unit + 58 fixtures verde**.

## Mutação (escopo alterado)

`cargo mutants` sobre `rs_parser.rs` (marcação) + as três regras (guard), filtrado às
funções tocadas: **18 mutantes, 18 mortos, 0 sobreviventes**. A marcação
(`for_each_child_in_test_scope`, `is_cfg_test_attribute`) e o guard (`||`, `!`,
comparações) são mordidos pelo par default-off/on. Um sobrevivente inicial
(`has_inner_cfg_test → false`) revelou **código redundante** — o `#![cfg(test)]` já
era tratado pela via pendente; a função foi **removida** (não testada à força com um
caso artificial), eliminando o mutante e simplificando.

## Validação pelo oráculo (re-run)

- **Lente, default**: `só-linter` **1 → 0** — a aresta `lente_app→lente_infra` de
  `04_wiring/app/src/erro.rs:87` (`#[cfg(test)]`) é projectada fora, simétrica à lente.
  `linter=21, lente=21, acordo=21, cego-linter=0`.
- **Self-lint do linter = 0**; nenhuma violação de **produção** perdida (a exclusão só
  tira arestas test-origin).
- **Reprodutores do 0060** (`biteproof_pathref`: `b::Thing` é **produção**) seguem
  fechados — `is_test_origin: false`, contam normalmente.

## Assimetria registrada (o modo ligado não tem oráculo)

Com `check_test_imports = true`, o linter fica **sem oráculo**: a lente exclui teste
por construção, não há segunda computação independente do grafo de **teste**. Então o
modo ligado é verificado **só por fixture** (`vtest_*_on_fail`), não pelo diferencial.
Isto é argumento extra para o default ser o modo que o oráculo cobre — registrado aqui,
não mascarado.

## Critérios de Verificação

- [x] Pré-condição (0060 presente; `check_cfg_test` reusado via `is_cfg_test_attribute`,
      não reinventado).
- [x] `is_test_origin` marcado em `collect_imports` **e** `collect_path_refs` por
      ancestral/irmão `#[cfg(test)]` (grammar: atributo é irmão que decora o seguinte).
- [x] `check_test_imports` em `config.rs`, **default `false`** (`None`).
- [x] Guard em V3/V9/V14: pula test-origin se `!check_test_imports`; ligada, volta.
- [x] `--emit-resolution` emite `is_test_origin`; o oráculo projecta test-origin fora
      **simetricamente** (não escondido no emit).
- [x] Fixtures: default-pass + on-fail (bite-proof dos dois lados); via `use` e via
      path-ref; produção não-regride.
- [x] Mutação re-rodada: **0 sobreviventes que mudam veredito** (1 redundante removido).
- [x] Oráculo default: lente `só-linter` **volta a 0**; self-lint 0; nenhuma violação
      de produção perdida.
- [x] Laudo: registra que o modo **ligado não tem oráculo** (verificado por fixture);
      o default-seguro justificado; nada mascarado.

## Histórico de Revisões

- 2026-06-09 — Política de regra: V3/V9/V14 excluem `#[cfg(test)]` por padrão,
  `check_test_imports` (default false) reabre. Campo `Import.is_test_origin` marcado
  nas duas vias de coleta (atributo é irmão na grammar — marca por item pendente);
  guard nas três regras; emit + projeção simétrica no oráculo. 5 fixtures (par
  bite-proof × via use/path-ref + produção). Mutação 0 veredito-mudante. Oráculo:
  só-linter 1→0. Modo ligado sem oráculo, nomeado.
