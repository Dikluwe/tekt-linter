# Assessment 0041-S1 — mutação de linhagem e propriedade de prompt

**Estado:** SHARD FOUND NO ACTIONABLE SURVIVORS  
**Data:** 2026-08-26  
**Passo:** P0113-S1  
**Branch:** `codex/p0113-s1`  
**Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## Universo L0 hash-pinned

O shard contém somente `prompt_header.rs`, `prompt_stale.rs` e
`multi_prompt_header.rs`. A repetição local com `cargo-mutants 27.1.0` enumerou exatamente
32 mutantes, na mesma ordem do passo.

| Insumo | SHA-256 |
|---|---|
| lista dos 32 mutantes (`/tmp/p0113-s1-mutants-list.txt`) | `fc11fde2de1888dffbcc34aed0f0e5c77061184e21136bc4db331d422fb17d31` |
| `prompt_header.rs` | `2f6bc4879206868ad12b156a3a8726d5203567843748a1f310858d17de14efba` |
| `prompt_stale.rs` | `050a4173ae5e0d309d7b4bb7763010678b03e88f542ada15adb12509a2b32091` |
| `multi_prompt_header.rs` | `c66b3c31cf2e7e8a83c1f0c0e065a13c0610f54a490b8fcaeceb0bd0b78b4514` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

Ferramentas: `cargo-mutants 27.1.0`; worktree `/tmp/p0113-s1`; saída segregada
`/tmp/p0113-output-s1`; no máximo dois jobs internos. O mapa
`0041-s1-authority-map.tsv` fixa prompts owner, observáveis e gates antes da execução.

Nenhuma produção, prompt ou teste foi alterado na abertura.

## Campanha cega

Uma primeira tentativa paralela foi abortada antes de testar mutantes: o baseline não
mutado encontrou três colisões `AlreadyExists` na fixture temporária de
`git_refinement_object_containment_assessment`. Essa tentativa é `GATE-DEFECT`, não
resultado do S1. Ela foi preservada integralmente em
`/tmp/p0113-output-s1-gate-defect-parallel/mutants.out`; seu `outcomes.json` tem SHA-256
`08ad3cf555da17c960d99af7008f62657955f61f14e2c582ef658a670bc6d234`.

A repetição segregada aceitou o baseline e percorreu todo o universo com dois jobs, sem
shuffle e sem edição intermediária:

| Resultado | Quantidade |
|---|---:|
| `CAUGHT` | 29 |
| `MISSED` | 0 |
| `UNVIABLE` | 3 |
| `TIMEOUT` | 0 |

| Artefato em `/tmp/p0113-output-s1/mutants.out` | SHA-256 |
|---|---|
| `mutants.json` | `9ff31c0a865dadde79d86ea1b1d1701956f130fc7c759d7ebb9a9fd32a7d646d` |
| `outcomes.json` | `280cd257cb79a6e98dde98cbd4acaee199555068dc26ed5d0ba59956f9beb7aa` |
| `caught.txt` | `83690e21ffe89f89c73df0034bb63a5c48b6c0951c8c9ab260a6088b8f851c8e` |
| `missed.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `unviable.txt` | `7f0ce85120c6fc5eabddfd4d5e4ce980aa22d2183f148163efddd527232e318c` |
| `timeout.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Reprodução, classificação e fechamento

Não houve `MISSED`, portanto não existe survivor a reproduzir ou `TEST-GAP` a sanear. Os
três `UNVIABLE` estão classificados integralmente em
`0041-s1-mutation-verdicts.tsv`: um exige `Default` inexistente para `Violation`; dois
exigem `Default` inexistente para `std::cmp::Ordering`. São limitações sintáticas do
gerador, sem alegação de morte semântica.

Gates dirigidos finais:

| Gate | Resultado |
|---|---|
| unitários de `prompt_header` | PASS — 7 |
| unitários de `prompt_stale` | PASS — 13 |
| unitários de `multi_prompt_header` | PASS — 8 |
| `lineage_header_classifiers_assessment` | PASS — 6 |
| `provenance_rules_assessment` | PASS — 6 |
| `prompt_ownership_wiring_assessment` | PASS — 7 |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |

P0113-S1 fecha sem `TEST-GAP`, `PRODUCTION-RED`, `SPEC-GAP`, `ARCH-RED` ou mudança de
produção/teste/prompt. O `GATE-DEFECT` da tentativa inicial está congelado, mas não
permanece bloqueante porque a campanha completa posterior teve baseline verde e zero
survivors acionáveis.
