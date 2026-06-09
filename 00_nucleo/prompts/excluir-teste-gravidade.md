# Prompt: excluir `#[cfg(test)]` da gravidade (V3/V9/V14), opção `check_test_imports`

> Numere em sequência (provável 0061) e salve em `00_nucleo/prompts/` do **linter**.
> Cruza linter (clone do 0052) e lente. Continuação de 0058–0060.
> Primeira mudança de **política de regra** da série (o que a regra *deve* fazer),
> não de resolvedor. Vem de decisão do autor do Tekt, registrada nesta sequência.

## Contexto e a decisão

A gravidade (V3/V9/V14) é uma afirmação sobre o **grafo de produção** — o que o
artefato entrega. Código `#[cfg(test)]` é removido do build de release, então uma
aresta que cruza camadas **só em teste** não corrompe a gravidade entregue. A lente
(grafo do compilador, sem `--cfg test`) **exclui teste por construção**; o linter
(textual) via tudo, e por isso V3/V9/V14 estavam, sem dizer, aplicando gravidade de
produção a código de teste. O 0060 expôs isso como `lente_app→lente_infra` num
`#[cfg(test)]` (`só-linter` 0→1).

**Decisão**: V3/V9/V14 **excluem `#[cfg(test)]` por padrão**; uma opção
`check_test_imports` (default **`false`**) reativa a verificação em código de teste,
para quem quer o teste-como-canário.

## O princípio do default (por que `false`)

Uma opção desabilitada que **afrouxa** uma regra é segura; uma que a **aperta**, não.
`check_test_imports` aperta (ligada, o linter marca mais), então o **default tem de
ser o comportamento correto** (excluir teste) e a opção é o **extra**. O contrário —
verificar teste por padrão, com opção de desligar — traria de fábrica o modo mais
propenso a falso-positivo, repetindo o modo de falha da anamnese em forma de config.
Mesma regra de polegar do `allow_adapter_structs` (opção que afrouxa o V12; default
estrito). **Default = seguro; a opção move para o menos seguro, nunca o contrário.**

Isto também desfaz a falsa inconsistência V2-vs-V3: cada regra fala de **um** grafo —
o V2 do grafo de teste (há cobertura?), o V3/V9/V14 do de produção. Eram só implícitas
sobre qual grafo; aqui ficam explícitas.

## Pré-condição

Clone do 0052; 0060 presente (`collect_path_refs`); corpus verde; **`check_cfg_test`
já existe** (detecção de `#[cfg(test)]` usada pelo V2 — **reusar, não reinventar**).

## A mudança

### A. Marcar origem do import (test vs produção)
`collect_imports` (use/extern) **e** `collect_path_refs` (refs por caminho, do 0060)
marcam cada import com `is_test_origin`, true se algum ancestral no AST é item
`#[cfg(test)]` (módulo/fn/bloco). Reusar a detecção do `check_cfg_test`. Definição
alinhada ao compilador: test-origin ≙ o que o build de produção remove (`#[cfg(test)]`).

### B. Config
`check_test_imports: bool` em `config.rs`, **default `false`**. (Está no caminho do
veredito — já sob mutação desde o 0057.)

### C. Gate nas regras
V3/V9/V14 **pulam** imports com `is_test_origin` quando `!check_test_imports`. É um
**guard**, não lógica nova. Ligada a opção, o comportamento antigo (verifica teste)
volta.

### D. Oráculo — projeção simétrica (não esconder no emit)
`--emit-resolution` passa a emitir o campo **`is_test_origin`** (emite tudo, marcado —
segue verdadeiro). O **harness do oráculo** remove as arestas test-origin
**simetricamente**, igual a lente as exclui por construção (mesma disciplina da
projeção bin→pacote do 0058). Resultado: no modo default, `só-linter` volta a **0**.

## Oráculo: assimetria a registrar

- **Default (exclui teste)**: o diferencial contra a lente **vale** — os dois veem só
  produção; `só-linter` volta a 0.
- **Ligada (`check_test_imports = true`)**: o linter fica **sem oráculo** — a lente
  exclui teste por construção, não há segunda computação do grafo de teste. Então o
  modo ligado é verificado **só por fixture**, não pelo diferencial. Registrar isso no
  laudo (e é argumento extra para o default ser o modo que o oráculo cobre).

## Fixtures bite-proof (os dois modos)

Par central — **mesma** aresta L1→L3 dentro de `#[cfg(test)]`, dois `crystalline.toml`:
- **`vtest_default_pass`** (`check_test_imports` ausente/false): `[]` — o guard exclui.
  **Bite-proof**: ligar a opção nesta fixture faz virar `["V3"]` (provar os dois lados).
- **`vtest_on_fail`** (`check_test_imports = true`): `[V3]` — o gate abre.

Cobrir as **duas** vias de coleta: uma via `use` em `#[cfg(test)] mod`, outra via
**path-ref** (do 0060) dentro de `#[cfg(test)] fn` — as duas têm de marcar origem.

Não-regressão de produção: uma aresta L1→L3 **fora** de teste segue `[V3]` nos dois
modos (a mudança não toca produção).

Harness afirma o multiset de IDs. Re-rodar a **mutação** no escopo alterado (marcação
de origem em `rs_parser`, o guard nas regras, a leitura da config): **0 sobreviventes
que mudam veredito** — a marcação e o guard mordidos pelo par (default-off vs on).

## Validação pelo oráculo (re-run)

- **Lente, default**: `só-linter` volta a **0** (a aresta `lente_app→lente_infra` de
  `#[cfg(test)]` é projetada fora, simétrica à lente). `cego-linter` segue 0.
- **Self-lint do linter = 0** (e confirmar que nenhuma violação de produção sumiu — a
  exclusão só tira arestas test-origin, que não são gravidade de produção).
- **Reprodutores do 0060** seguem fechados (não eram test-origin).

## Critérios de Verificação

- [ ] Pré-condição (0060 presente; `check_cfg_test` reusado, não reinventado).
- [ ] `is_test_origin` marcado em `collect_imports` **e** `collect_path_refs` por
      ancestral `#[cfg(test)]`.
- [ ] `check_test_imports` em `config.rs`, **default `false`**.
- [ ] Guard em V3/V9/V14: pula test-origin se `!check_test_imports`; ligada, volta.
- [ ] `--emit-resolution` emite `is_test_origin`; o **oráculo** projeta test-origin
      fora **simetricamente** (não escondido no emit).
- [ ] Fixtures: default-pass (com bite-proof ligando a opção → V3) + on-fail; via `use`
      e via path-ref; produção não-regride.
- [ ] Mutação re-rodada: 0 sobreviventes que mudam veredito no escopo alterado.
- [ ] Oráculo default: lente `só-linter` volta a **0**; self-lint 0; nenhuma violação
      de produção perdida.
- [ ] Laudo: registra que o modo **ligado não tem oráculo** (verificado por fixture);
      o default-seguro justificado; nada mascarado.

## Fora de escopo (trilhas seguintes)

- **Isolamento de teste** (testes do L1 exigirem compilar o L3) — invariante
  **diferente**, sobre o grafo de **dev-deps**, não a gravidade de produção. Regra
  própria futura, não esta reaproveitada.
- **Corpus de projetos reais variados** — descoberta dos cegos ainda desconhecidos.
- **Contador de `Layer::Unknown`**; **oráculo de posição/severidade**; **merge com o
  `master` público**.

## Disciplina

Reusar `check_cfg_test` (não reinventar detecção de teste); default seguro (a opção só
aperta); emit verdadeiro + projeção simétrica no oráculo; fixtures provando os dois
modos (o default bite-proof pelos dois lados); mutação re-rodada; laudo nomeando a
ausência de oráculo no modo ligado; nada mascarado.
