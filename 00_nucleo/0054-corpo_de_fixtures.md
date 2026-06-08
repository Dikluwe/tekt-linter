# Laudo 0054 — corpo de fixtures bite-proof + completude por mutação

**Onde roda**: no clone canônico do `tekt-linter` (o que tem o conserto do 0052).
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/corpo-de-fixtures.md`](prompts/corpo-de-fixtures.md)
**Camadas tocadas**: nenhuma regra L1–L4 mudou. O trabalho é todo em `tests/`
(dados de fixture + harness de integração) e em `crystalline.toml` (duas entradas
de exclusão/exceção). O motor de regras ficou intacto — só foi *exercitado*.

---

## Pré-condição (confirmada)

Este é o clone com o conserto do 0052:

- `03_infra/crate_registry.rs` existe e resolve membros de workspace
  (`from_root` → `[workspace].members` → `member_layer`/`owner_of`).
- `classify_import` (`03_infra/rs_parser.rs:337`) enxerga cross-crate: resolve
  membro first-party à sua camada (passo 4) e distingue dep externa declarada
  (passo 5) de item local (passo 7). Self-import nunca vira `Unknown` (resíduo
  0053).
- Suíte do 0052 verde: `478 unit + 28 fixtures = 0 falhas`.

**Não houve parada.** As três regras dependentes (V3/V9/V14) foram construídas
contra este clone, que as resolve corretamente.

## Divergência com o `master` público (merge pendente, à parte)

O `master` em `github.com/Dikluwe/tekt-linter` tem parsers de C/C++/Zig/Python e o
"Hash Locking", e **não** tem o conserto do 0052. Este clone tem o conserto e (no
estado de partida) o multi-linguagem parcial. As linhas divergiram; nenhuma é
superconjunto da outra. **Decisão de merge fica registrada como pendente e à
parte** — não bloqueou este trabalho: das 14 regras, 11 são idênticas nas duas
linhas; só V3/V9/V14 dependem do `classify_import`, que este clone tem certo.

---

## O que foi construído

### `tests/fixtures/vNN_{pass,fail}/` — 28 workspaces

Um par por regra V1–V14. Cada caso é o **menor** workspace/arquivo que exercita a
característica daquela regra, com o seu próprio `crystalline.toml` e `00_nucleo/`.
O linter roda como **caixa-preta** (binário, `current_dir` na fixture) — o harness
não liga contra a lib, replicando `crystalline-lint .` real.

| Regra | `pass` | `fail` (veredito fixado) |
|---|---|---|
| V1 prompt_header | header válido + prompt existe | sem header → `[V1]` |
| V2 test_file | L1 com `impl`-com-corpo + `#[cfg(test)]` | mesmo `impl` sem teste → `[V2]` |
| **V3 forbidden_import** | **multi-crate** L2→L1 (direção válida) | **multi-crate L2→L4** → `[V3]` |
| V4 impure_core | L1 puro | L1 com `std::fs::metadata(..)` → `[V4]` |
| V5 prompt_drift | sem hash declarado | `@prompt-hash` ≠ real → `[V5]` |
| V6 prompt_stale | snapshot == interface | snapshot ≠ interface → `[V6]` |
| V7 orphan_prompt | todos os prompts referenciados | prompt extra não-referenciado → `[V7]` |
| V8 alien_file | arquivo em camada mapeada | arquivo em dir não-mapeado → `[V8]` |
| **V9 pub_leak** | **multi-crate** import por porta (`entities`) | **multi-crate** subdir não-porta → `[V9]` |
| V10 quarantine_leak | L3 sem import de `lab` | L3 `use crate::lab::…` → `[V3, V10]` |
| V11 dangling_contract | trait com `impl` em L2 | trait sem `impl` → `[V11]` |
| V12 wiring_logic_leak | `struct` adaptador em L4 | `enum` em L4 → `[V12]` |
| V13 mutable_state_core | L1 sem estado mutável | L1 com `static mut` → `[V13]` |
| **V14 external_type_in_contract** | **multi-crate** import só first-party L1 | **multi-crate** `serde` (externo) + first-party no mesmo arquivo → `[V14]` |

### Decisões que o harness fixa (não "passou", mas IDs + contagem)

- **V10 co-ocorre com V3 por construção.** Qualquer import de produção→`lab` é
  proibido pela direção (V3) e pela quarentena (V10) ao mesmo tempo — não há
  camada-origem que dispare um sem o outro (V3 e V10 ambos pulam origem
  `Lab`/`L0`/`Unknown`). O veredito honesto é o par `[V3, V10]`, fixado como tal.
- **V14 prova a distinção do 0052 num só arquivo.** O `vNN_fail` importa
  `serde::Serialize` (externo real → `Unknown` → V14) **e** `corehelper::Helper`
  (membro first-party L1 → `Layer::L1` → **não** falha) no mesmo ficheiro. O
  veredito é exatamente **um** `V14`, não dois — é a prova-de-mordida da regra
  ciente de deps.
- **V3/V9/V14 são multi-crate de verdade.** Workspaces com `[workspace].members`,
  `[dependencies]` reais (path/externa), camadas dos membros via `[layers]`. É a
  forma que o self-lint (crate único) nunca exercita — onde o 0052 regrediu.

### `tests/fixtures.rs` — harness

28 `#[test]`, um por fixture. Cada um roda o binário (`--format sarif .`), extrai
os `ruleId` do SARIF e afirma o **multiset exato** de IDs. Específicos o bastante
para servir de oráculo à mutação.

### `crystalline.toml` (repo) — duas entradas

- `[excluded] fixtures = "tests"`: tira `tests/` do self-lint (file walker e
  prompt walker). As fixtures são dados deliberadamente quebrados — fora da
  topologia de camadas.
- `[orphan_exceptions] "00_nucleo/prompts/corpo-de-fixtures.md"`: o prompt
  materializa fixtures+harness em `tests/` (excluído), não arquivo Rust — órfão
  legítimo, igual a `cargo.md`/`readme_prompt.md` (padrão ADR-0006).

---

## Armadilhas encontradas (e o que ensinam sobre o motor)

Construir as fixtures expôs, por tentativa-contra-o-linter, regras finas do motor
que a documentação não destacava — cada uma virou uma decisão de fixture:

1. **V4 lê tokens de *call/macro*, não o `use`.** `use std::fs;` sozinho não morde;
   é preciso uma chamada qualificada `std::fs::metadata(..)`. Encadear
   (`.read_to_string(..).unwrap()`) conta **duas** vezes (o texto do nó-função
   externo também começa com `std::fs`). Fixture usa chamada única → exatamente
   um `V4`.
2. **`is_declaration_only` isenta de V2.** Um ficheiro sem `impl { fn() {…} }` é
   tratado como "só declaração" e dispensa teste. Free functions não bastam para
   provocar V2 — a fixture precisa de um `impl`-com-corpo.
3. **`crate::lab` só resolve a `Lab` com `[module_layers] lab = "lab"`.** Sem o
   mapeamento, vira `Unknown` e nem V3 nem V10 disparam. A fixture V10 declara o
   mapeamento.
4. **`resolve_file_layer` casa o *primeiro componente* do path com `[layers]`**, e
   `[layers]` é um dir por camada. Para o V14 (dois crates L1), os membros foram
   aninhados sob um único `crates/` mapeado a L1 (`members = ["crates/corelib",
   "crates/corehelper"]`).
5. **Órfãos em cadeia.** Todo prompt em `00_nucleo/prompts/` da fixture tem de ser
   referenciado por algum header, senão vira V7 e polui o veredito. O `vNN_fail`
   de V1 (sem header) teve o `core.md` removido para não orfanar.

---

## Mutação — prova de completude contra o motor de regras

Ferramenta: `cargo-mutants 27.1.0`. Escopo (conforme o prompt):
`--file 01_core/rules/*.rs --file 03_infra/crate_registry.rs`
(motor de regras + a classificação ciente de deps do 0052), e numa segunda fase
`--file 03_infra/rs_parser.rs` (onde vivem `classify_import`/`resolve_subdir`).

> _Resultados preenchidos abaixo após a execução._

### Fase 1 — motor de regras + `crate_registry.rs` (escopo nomeado pelo prompt)

`cargo mutants -j 4 --file '01_core/rules/*.rs' --file '03_infra/crate_registry.rs'`

**103 mutantes → 80 mortos, 22 inviáveis, 1 sobrevivente.** O único sobrevivente é
**equivalente** (ver abaixo). O motor de regras (V1–V14) está mutation-complete: cada
ramo de decisão de regra é mordido por ao menos uma fixture.

> Duas iterações: a 1ª deixou 5 sobreviventes em `prompt_stale::added_strs` (4) e
> `crate_registry::empty` (1). Os 4 do `added_strs` operam só sobre **reexports**, que
> nenhuma fixture V6 exercitava. Foram mortos redesenhando o par V6: o `fail` tem como
> **único** delta um `pub use` ausente no snapshot; o `pass` tem reexports em **ordem
> invertida** (`current != snapshot`, mas delta vazio por diferença-de-conjunto) — o que
> força `added_strs` a ser chamado com conteúdo igual, matando os mutantes-fantasma
> (`vec!["xyzzy"]`/`vec![""]`) que só morrem quando o ramo `delta.is_empty()` é exercitado.

### Fase 2 — `rs_parser.rs` (inclui `classify_import`/`resolve_subdir`)

`cargo mutants -j 4 --file '03_infra/rs_parser.rs'`

**178 mutantes → 112 mortos, 13 inviáveis, 53 sobreviventes.**

A **classificação ciente de deps** — `classify_import`, `resolve_subdir`,
`module_layer`, `first_segment` (a superfície do 0052) — tem **0 sobreviventes**. O
único sobrevivente que era de `resolve_subdir` (linha 406, `||`→`&&` no ramo
`crate::`/`super::`) foi morto com a fixture `v09b_fail_intra`: `use crate::internal;`
(2 segmentos, módulo L1 não-porta), o caso em que o ramo `crate::` e o ramo cross-crate
de `resolve_subdir` **divergem** — a fixture V9 cross-crate não o mordia.

Os 53 sobreviventes restantes estão **fora do escopo nomeado pelo prompt** (são
maquinaria incidental do parser, não a decisão de regra nem a classificação) e
caem em três categorias:

1. **Posição/metadado — equivalentes sob o oráculo de IDs (≈30).**
   `find_first_error_pos` (17) e a aritmética `+1` de linha/coluna em `collect_tokens`,
   `collect_imports` e `extract_declarations`. Mudam a **linha:coluna reportada**, nunca
   *qual* regra dispara. O harness afirma **IDs + contagem** por contrato (a prova-de-
   mordida é sobre o veredito), então estes são equivalentes para esse oráculo. Matá-los
   exigiria um contrato de teste por-posição — mais frágil e de outra natureza.
   (`find_first_error_pos` serve a V0/`PARSE`, que está fora do corpo V1–V14 deste prompt.)
2. **`parse_layer_tag` (5) — equivalentes.** Produzem `PromptHeader.layer`, que **nenhuma
   regra lê**: a camada efetiva do ficheiro vem de `resolve_file_layer` (path), não da tag
   `@layer`. Mutar a tag não muda veredito algum (confirmado por `grep` nas regras).
3. **Extração de interface/declaração/cobertura — lacuna conhecida, adiada (≈18).**
   `extract_public_interface`/`extract_type_sig`/`extract_named_children`/
   `extract_trait_method_names` (V6 com tipos), `collect_type_param_names` (V12 com
   genéricos), `check_cfg_test`/`has_impl_with_functions` (V2 em bordas). **Não são
   equivalentes** — seriam mortos com fixtures V6/V12 que carreguem tipos e membros no
   snapshot. Ficam **explicitamente adiados** para um prompt seguinte de enriquecimento do
   corpo (tipos/genéricos na interface), por exercitarem detalhe de *extração* do parser,
   não a lógica de decisão/classificação que este prompt centra.

### Mutante sobrevivente documentado como equivalente

- `03_infra/crate_registry.rs:43` — `replace CrateRegistry::empty -> Self with
  Default::default()`. `CrateRegistry` deriva `Default`, que produz `members:
  Vec::new()` — **exatamente** o que `empty()` retorna. Os dois construtores são
  comportamentalmente idênticos; nenhum teste pode distingui-los. Equivalente genuíno.

---

## Critérios de Verificação

- [x] Pré-condição confirmada (este é o clone com o 0052).
- [x] Divergência com o `master` público registrada (merge pendente, à parte).
- [x] Uma fixture-passa e uma fixture-falha por regra V1–V14, com veredito fixado.
- [x] V3/V9/V14 multi-crate; V3 inclui a inversão L2→L4; V14 distingue externo
      real de first-party L1.
- [x] Harness afirma IDs + contagem (multiset SARIF), não só sucesso/fracasso.
- [x] `cargo-mutants` sobre o **motor de regras + classificação** (escopo nomeado):
      0 sobreviventes não documentados (rules `01_core/rules/*.rs` = 0;
      `classify_import`/`resolve_subdir` = 0; `crate_registry::empty` = 1 equivalente
      documentado). A varredura do `rs_parser` inteiro (além do escopo) deixa 53
      sobreviventes incidentais, todos categorizados: ~30 posição/metadado-equivalentes,
      5 `parse_layer_tag`-equivalentes, ~18 de extração de interface **adiados** com
      justificativa.
- [x] Cobertura de ramos no código de **regra/classificação** sem ramo morto — provada
      pela mutação (que subsume cobertura de ramos): 0 sobreviventes ⇒ todo ramo de
      decisão é executado e *observado* por uma fixture.
- [x] O linter continua passando em si mesmo (`crystalline-lint .` = 0).
- [x] Nada mascarado (a exclusão de `tests/` é dado de fixture, não whitelist de
      caso; documentada).
- [x] Laudo escrito em `00_nucleo/` no formato dos laudos anteriores.

---

## Histórico de Revisões

- 2026-06-08 — Materialização inicial: 28 fixtures + harness, exclusão de
  `tests/`, exceção de órfão do prompt. Suíte 478+28 verde, self-lint = 0.
