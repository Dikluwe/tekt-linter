# Prompt: fechar os ~18 sobreviventes de extração (V6/V12/V2) — completude do parser

> Numere em sequência ao laudo do corpo de fixtures (provável 0055) e salve em
> `00_nucleo/prompts/`. Alvo: o repo do **linter** (o clone com o conserto do 0052).
> Continuação direta do laudo 0054 (corpo de fixtures + mutação).

## Contexto

O laudo 0054 deixou o **motor de decisão** (regras V1–V14) e a **classificação de
import** (`classify_import`/`resolve_subdir`, a superfície do 0052) com **0
mutantes sobreviventes**. Mas a varredura do `rs_parser.rs` inteiro deixou 53
sobreviventes, e o laudo os separou em três grupos:

- ~30 de **posição/metadado** (`find_first_error_pos`, aritmética `+1` de
  linha:coluna) — equivalentes sob o oráculo do harness, que afirma IDs+contagem,
  não posição.
- 5 de **`parse_layer_tag`** — equivalentes: produzem `PromptHeader.layer`, que
  nenhuma regra lê (a camada efetiva vem de `resolve_file_layer` pelo path).
- **~18 de extração de interface/declaração** — o próprio laudo registrou que
  **NÃO são equivalentes**: seriam mortos por fixtures que carreguem tipos,
  membros e genéricos. Foram **adiados**.

Este prompt fecha esses ~18. Não é "mais um enriquecimento" solto: cada
sobrevivente abaixo tem um gatilho preciso, lido da fonte. Ao fim, ou o mutante
morre, ou é **reprovado como adiado** e vira equivalente provado — a ressalva do
0054 ("buraco conhecido na extração que alimenta V6/V12") sai do laudo.

> **Nota de nucleação (0055)**: linhas do `rs_parser.rs` no prompt original eram
> de uma versão pré-0052 (~100 a menos); corrigidas acima para este clone.
> E `collect_type_param_names` (774) alimenta **`extract_blanket_impls` (V11)**,
> não `extract_type_sig`/`extract_declarations` — seus arms `type_identifier`/
> `constrained_type_parameter` são compat de grammars antigas (tree-sitter-rust
> 0.23 só emite `type_parameter`), logo são **equivalentes sob a grammar pinada**.

## Pré-condição

Clone canônico (com o conserto do 0052) e o corpo do 0054 presente e verde: 28
fixtures + harness `tests/fixtures.rs`, self-lint = 0. Confirmar antes de mexer.

## Os ~18, por gatilho (lido de `01_core/rules/` + `03_infra/rs_parser.rs`)

### Bloco V6 — `prompt_stale` (a maioria dos ~18)

Mecânica real: o V6 retorna vazio se `current == snapshot` (`prompt_stale.rs:34`);
senão computa `compute_delta`, cujos campos são **added/removed** de
`functions`, `types` e `reexports`. A fixture V6 do 0054 só variava **reexports**
— então as dimensões `functions` e `types` do snapshot nunca foram exercitadas, e
os extratores que as constroem sobrevivem à mutação.

Extratores a cobrir (assinaturas em `rs_parser.rs`): `extract_public_interface`
(427), `extract_type_sig` (512), `extract_named_children` (540, campos de
struct/variantes de enum), `extract_trait_method_names` (554),
`collect_type_param_names` (774 — ver nota abaixo).

Fixtures (enriquecer o par V6 e/ou somar irmãos `v06c_…`):

1. **`pass` com interface rica e idêntica ao snapshot.** O ficheiro tem ≥1 função
   com tipos em parâmetro e retorno, ≥1 `struct` com ≥1 campo nomeado, ≥1 `enum`
   com ≥1 variante nomeada, ≥1 `trait` com ≥1 método, e ≥1 tipo genérico
   `Foo<T>`. O snapshot do prompt codifica **exatamente** essa interface →
   `current == snapshot` → **sem V6**. Isto mata os mutantes "extrator perde/
   embaralha X": qualquer alteração na extração quebra a igualdade, dispara um V6
   espúrio, e o `pass` (que afirma 0 violações) falha sob o mutante.
2. **`fail` com delta só em `functions`.** Snapshot omite uma função que o código
   tem → exercita `added_functions`/`added_fns` e a extração de
   `FunctionSignature`/`extract_type_sig`. Veredito fixado: um `[V6]`.
3. **`fail` com delta só em `types`.** Snapshot tem uma type sig com um campo a
   menos (ou um método de trait renomeado, ou um param genérico a menos) → exercita
   `added_types`, `extract_named_children`, `extract_trait_method_names`,
   `collect_type_param_names`. Veredito fixado: um `[V6]`.

### Bloco V12 — `wiring_logic_leak` (genéricos)

Mecânica real: a regra dispara por `DeclarationKind` (`wiring_logic_leak.rs:51`:
`Enum => true`; `Struct => !allow_adapter_structs`). `collect_type_param_names`
alimenta a declaração/type-sig com os genéricos; sem fixture genérica no L4 ele
sobrevive.

Fixture (enriquecer o par V12):
4. **`fail` com `enum` genérico no L4:** `enum E<T> { … }` → deve disparar **um**
   `[V12]`. **`pass` com `struct` adaptador genérico no L4:** `struct S<T> { … }` →
   **não** dispara (`allow_adapter_structs`). Isto torna a extração de genéricos
   carga-útil para a detecção de kind, matando os mutantes de
   `collect_type_param_names` que a fixture V6 não alcançar.

### Bloco V2 — `test_file` (bordas, em L3)

Mecânica real: a regra L1 confia em `has_test_coverage`/`is_declaration_only`
(`test_file.rs`: dispara se L1 e `!has_test_coverage`; isenta declaração-só). Quem
computa isso é o L3: `check_cfg_test`/`has_test_attribute` (615) e `has_impl_with_functions` (642). O
par V2 do 0054 (impl-com-corpo com/sem teste) não exercita as bordas.

Fixtures (somar irmãos `v02b_…`/`v02c_…`):
5. **`impl` sem função** em L1 sem teste: `impl Foo { const X: u8 = 1; }` (ou impl
   vazio) → `has_impl_with_functions == false` → tratado como declaração-só →
   **isento, sem V2**. Mata o mutante que inverte `has_impl_with_functions`.
6. **`cfg` que não é teste** em L1 sem teste real: `#[cfg(feature = "x")] mod m { … }`
   → `check_cfg_test == false` → **ainda dispara `[V2]`**. Mata o mutante que conta
   qualquer `cfg` como cobertura de teste. (E confirme que o `#[cfg(test)]` real
   isenta — já coberto pelo par 0054.)

## Método

1. Materializar as fixtures acima (com seu `crystalline.toml`/`00_nucleo/` mínimo;
   cuidado com órfãos em cadeia — armadilha 5 do 0054 — todo prompt da fixture tem
   de ser referenciado).
2. Estender `tests/fixtures.rs` com os `#[test]` novos, afirmando o multiset exato
   de IDs (mesmo contrato do 0054).
3. Re-rodar a mutação no escopo que tinha sobreviventes:
   `cargo mutants -j 4 --file '03_infra/rs_parser.rs'` (e, por garantia,
   `--file '01_core/rules/*.rs'`). **Iterar** até que os ~18 de extração morram.
4. Reclassificar o que sobrar: cada sobrevivente restante do `rs_parser` tem de
   ser **provado equivalente** (posição/metadado sob o oráculo de IDs, ou
   `parse_layer_tag` que nenhuma regra lê) — com a linha e a razão. Nada fica
   "adiado".

## Critérios de Verificação

- [ ] Pré-condição confirmada (clone do 0052; corpo 0054 verde).
- [ ] Fixtures novas: V6 (pass rica idêntica + fail-só-functions + fail-só-types),
      V12 (enum genérico fail + struct genérico pass), V2 (impl-sem-função isento +
      cfg-não-teste dispara).
- [ ] Harness afirma IDs + contagem para cada fixture nova.
- [ ] `cargo mutants` em `rs_parser.rs`: os **~18 de extração mortos**; os
      restantes (posição ~30, `parse_layer_tag` 5) **provados equivalentes**, um a
      um, com linha e razão. **0 sobreviventes não documentados.**
- [ ] (Opcional, conferência barata) `cargo llvm-cov` sobre regras + extração:
      nenhum ramo sem execução que a mutação não tenha tocado.
- [ ] Self-lint = 0; suíte verde fora do `blanket_impl` pré-existente.
- [ ] Nada mascarado.
- [ ] Laudo escrito; e a ressalva do 0054 (buraco de extração V6/V12) marcada como
      **fechada** lá, com ponteiro para este laudo.

## Fora de escopo (prompts seguintes — os detectores contra a linguagem)

Continuam pendentes, em ordem: contador de `Layer::Unknown` em alvo real; oráculo
diferencial contra a computação de dependências da lente (`tekt-cargo-dsm`); corpus
de projetos reais estruturalmente variados. E, à parte, a decisão de merge com o
`master` público (multi-linguagem + Hash Locking ⊕ conserto do 0052).

## Disciplina (do repo)

Prompt nucleado antes do código; linhagem nos arquivos novos; a mutação é a forma
mecânica da prova-de-mordida; nada mascarado; laudo ao fim.
