# Laudo 0055 — fechar os sobreviventes de extração (V6/V12/V2/V4)

**Onde roda**: clone canônico do `tekt-linter` (com o conserto do 0052).
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/fechar-extracao-v6-v12-v2.md`](prompts/fechar-extracao-v6-v12-v2.md)
**Continuação de**: laudo 0054 (corpo de fixtures). Fecha a ressalva "buraco de
extração que alimenta V6/V12" deixada lá.
**Camadas tocadas**: nenhuma regra mudou. Só `tests/` (8 fixtures + 8 testes) e
`crystalline.toml` (exceção de órfão do prompt).

---

## Pré-condição (confirmada)

Clone do 0052; corpo 0054 presente e verde (29 fixtures, self-lint = 0).

## O alvo: os 29 sobreviventes do `rs_parser.rs` (run final do 0054)

O 0054 deixou o motor de regras e a classificação com 0 sobreviventes, mas a
varredura do `rs_parser.rs` inteiro deixou sobreviventes incidentais. Triados
contra a fonte deste clone (não a versão pré-0052 do prompt, ~100 linhas a menos):

**Mortos por fixture nova (15):**

| Sobrevivente (linha:função) | Fixture que mata | Como |
|---|---|---|
| 443/448/453 `extract_public_interface` (arms struct/enum/trait) | `v06b_pass` | interface rica idêntica: deletar um arm tira o tipo de `current` → `current != snapshot` → V6 espúrio → pass falha |
| 517 `extract_type_sig -> None` | `v06b_pass` | idem (tipo some) |
| 541×3 `extract_named_children -> vec![]/[""]/["xyzzy"]` | `v06b_pass` | campos/variantes mudam → V6 espúrio |
| 544 `extract_named_children == → !=` | `v06b_pass` | idem |
| 555×3 `extract_trait_method_names -> vec![]/[""]/["xyzzy"]` | `v06b_pass` | métodos de trait mudam → V6 espúrio |
| 590 `collect_tokens` delete arm `macro_invocation` | `v04b_fail` | `std::fs::x!()` deixa de ser tokenizado → V4 some → fail falha |
| 620×2 `check_cfg_test == → !=` | `v02c_fail` / par 0054 | detecção de `cfg(test)` quebra → cobertura erra → V2 muda |
| 650 `has_impl_with_functions && → ||` | `v02b_pass` | `impl { const X = { 1 } }`: `function_item(false) || block(true)` → detecta fn falsa → não-isento → V2 espúrio |

Reforço de `compute_delta` (added/removed de `functions` e `types`): `v06c_fail`
(delta só em functions) e `v06d_fail` (delta só em types).

**Provados equivalentes (14) — não matáveis, justificados um a um:**

1. **786/788 `collect_type_param_names` (arms `type_identifier` / `constrained_type_parameter`)**
   — código morto sob a grammar pinada. `tree-sitter-rust 0.23` embrulha todo
   parâmetro num nó `type_parameter`; estes dois arms só existem para grammars
   anteriores (comentário em `rs_parser.rs:770`). Nenhum input com 0.23 os alcança.
   Além disso a função alimenta `extract_blanket_impls` (V11), não V6/V12 — o prompt
   supôs V6/V12 type-sigs, mas `extract_type_sig`/`extract_declarations` não a chamam.
2. **253 `collect_imports && → ||`** — só altera a classificação de `ImportKind`
   (`Named` vs `Direct`). **Nenhuma regra de produção lê `import.kind`** (confirmado
   por `grep` em `01_core/rules/`; o próprio comentário em `forbidden_import.rs:162`
   diz "V3 não usa ImportKind"). O `kind` computado nunca vira veredito → equivalente.
3. **247/266/584/597/813/822/833 (aritmética `+`→`-`/`*` de linha:coluna)** (11) —
   `collect_imports`, `collect_tokens`, `extract_declarations`. Mudam a
   **linha:coluna reportada** de uma violação, nunca *qual* regra dispara. O harness
   afirma IDs + contagem por contrato (a prova-de-mordida é sobre o veredito), então
   são equivalentes sob esse oráculo. Matá-los exigiria um contrato de teste
   por-posição — de outra natureza, e fora do escopo do corpo de fixtures.

> `find_first_error_pos` (17) e `parse_layer_tag` (5), já provados equivalentes no
> 0054, continuam fora: posição de erro de sintaxe (V0/`PARSE`, fora do corpo
> V1–V14) e `PromptHeader.layer` (que nenhuma regra lê — a camada vem do path).

## Resultado da mutação

`cargo-mutants 27.1.0`, `-j 4`.

**`03_infra/rs_parser.rs`** (`178 mutantes`): de **53 → 38 sobreviventes**
(127 mortos, 13 inviáveis). Os 15 de extração que mudavam veredito foram mortos
pelas fixtures novas.

> **Reconciliação dos 38 — ver laudo [0056](0056-reconciliar_sobreviventes.md).**
> A itemização original aqui somava 36 (erro de `grep`: `find_first_error_pos`
> são **19**, não 17 — 2 mutantes de substituição-de-função-inteira não casaram o
> sufixo). E o rótulo único "equivalente" misturava duas naturezas. A classificação
> correta, verificada contra `missed.txt` (soma = 38, exata):
>
> | Natureza | nº | Itens |
> |---|---|---|
> | **Muda veredito** | **0** | — |
> | **Inerte** (saída não-lida / código morto) | **8** | `parse_layer_tag` (5) · `collect_imports:253` (1) · `collect_type_param_names:786/788` (2) |
> | **Fora-do-oráculo** (posição) | **30** | `find_first_error_pos` (19) · aritmética linha:coluna em `collect_imports`/`collect_tokens`/`extract_declarations` (11) |
>
> O corpo é **completo para vereditos** (0 sobreviventes que mudam qual regra dispara
> ou a contagem de IDs). Não é completo para a saída inteira: a **posição** de uma
> violação e a posição de um erro de sintaxe (V0/`PARSE`) são um **oráculo à parte**,
> hoje não testado pelo harness (que afirma IDs + contagem).

**`01_core/rules/*.rs` + `crate_registry.rs`** (`103 mutantes`): **1 sobrevivente**,
inerte, documentado no 0054 (`CrateRegistry::empty == Default::default()`).

**Suíte**: `478 unit + 38 fixtures` verde; `crystalline-lint .` = 0.

## Critérios de Verificação

- [x] Pré-condição confirmada (clone do 0052; corpo 0054 verde).
- [x] Fixtures novas: V6 (`v06b_pass` rica idêntica + `v06c_fail` só-functions +
      `v06d_fail` só-types), V12 (`v12b_fail` enum genérico + `v12b_pass` struct
      genérico), V2 (`v02b_pass` impl-sem-fn isento + `v02c_fail` cfg-não-teste), e
      `v04b_fail` (macro proibido).
- [x] Harness afirma IDs + contagem para cada fixture nova.
- [x] `cargo mutants` em `rs_parser.rs`: os 15 de extração mortos; os 38 restantes
      provados equivalentes, um a um (tabela acima). **0 sobreviventes não documentados.**
- [x] Self-lint = 0; suíte verde.
- [x] Nada mascarado.
- [x] Ressalva do 0054 fechada (ver atualização lá).

## Histórico de Revisões

- 2026-06-08 — Materialização: 8 fixtures (V6 rico, V12 genérico, V2 bordas, V4
  macro) + 8 testes. Correção das linhas do prompt (pré-0052) e da premissa sobre
  `collect_type_param_names`. Mutação fechada (ver resultado).
