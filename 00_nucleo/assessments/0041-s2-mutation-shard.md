# Assessment 0041-S2 — projeções e relações semânticas

> **Estado:** SHARD FOUND NO ACTIONABLE SURVIVORS
> **Passo:** P0113-S2
> **Branch:** `codex/p0113-s2`
> **Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## Escopo congelado

O shard contém exatamente 15 mutantes, na ordem produzida por `cargo-mutants 27.1.0`,
nos cinco módulos L1 puros `dangling_contract`, `context_erasure`,
`decision_ownership`, `infrastructure_error` e `semantic_field_loss`. A lista tem
SHA-256 `c2f26f19ec04cb601b7b767ca5c6420ece76fcc6376e62fee55b02233281c40d`.

| Artefato | SHA-256 |
|---|---|
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |
| `dangling_contract.rs` | `b26f111661dc12ad3d42f08e8d673cc164d64c0b870e364c8be0e1e458f98351` |
| `context_erasure.rs` | `bd75c60264f083f50105ea7e6902302968b67fdfa305db31940300e5756b6f70` |
| `decision_ownership.rs` | `51dc9421df6cd1273be547f645e94fadd8824eb92fc29f864c75940df5c7bf6f` |
| `infrastructure_error.rs` | `03fdbacd6541a5ff737504a43f8b2b46dae126014cd2cadc8c51d704656b7298` |
| `semantic_field_loss.rs` | `0020d4f71607951d0a867e294838d94b86c8477960cbc44788b164847ab4b0c0` |

Os prompts owner e seus contratos observáveis estão materializados em
`0041-s2-authority-map.tsv`. Nenhuma edição de produção ou teste é autorizada durante a
campanha cega.

## Comando congelado

```text
cargo mutants -j 2 --no-shuffle --no-times --output /tmp/p0113-output-s2 \
  --file 01_core/rules/dangling_contract.rs \
  --file 01_core/rules/context_erasure.rs \
  --file 01_core/rules/decision_ownership.rs \
  --file 01_core/rules/infrastructure_error.rs \
  --file 01_core/rules/semantic_field_loss.rs
```

## Defeito de gate segregado

A primeira tentativa parou no baseline, antes de executar mutantes. Quatro fixtures
dependem de diretórios `00_nucleo/prompts` vazios que existem no workspace de origem, mas
Git não materializa em novos worktrees. A falha foi classificada `GATE-DEFECT`; sua saída
foi preservada em `/tmp/p0113-output-s2-gate-defect-round1`. Os diretórios vazios foram
recriados sem alteração rastreada, e um target novo impediu reutilização de binários cujo
`CARGO_MANIFEST_DIR` estava pinado à cópia temporária da tentativa anterior.

O segundo baseline passou integralmente. O escopo, as fontes, os prompts e a lista de
mutantes permaneceram byte a byte inalterados.

## Campanha cega e classificação

| Resultado | Quantidade |
|---|---:|
| `CAUGHT` | 11 |
| `MISSED` | 0 |
| `UNVIABLE` | 4 |
| `TIMEOUT` | 0 |

| Artefato | SHA-256 |
|---|---|
| `mutants.json` | `b82038cd3b7972ee4a802c4e3a62f26c515b74e0034684af41e87f40da5c4fb1` |
| `outcomes.json` | `9e6a04a06651b6cbb6c0043150dfc813e831eeb3f7ed8615890a951f3e4e0f6f` |
| `caught.txt` | `c4385f639208ee79d3efeb3ba4ed92308a4c71b322819de69d82d73218faf983` |
| `missed.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `timeout.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `unviable.txt` | `f9aa54a8120688c600c6097009de41c9eb008c674f6a2c9cafe8c9e37b1ebefd` |

Como `MISSED=0`, não há reprodução serial nem `TEST-GAP` a sanear. Os quatro não mortos
foram classificados individualmente em `0041-s2-mutation-verdicts.tsv`: todos tentam
construir `Violation::default()`, mas `Violation` deliberadamente não implementa
`Default`; `rustc` rejeita cada substituição com E0277. São `TOOL-LIMIT`, não mutantes
semanticamente mortos e não evidência sobre comportamento.

## Decisão

Todo mutante compilável do universo S2 foi morto pelos gates existentes. Não houve
`TEST-GAP`, `PRODUCTION-RED`, `SPEC-GAP`, `ARCH-RED`, flaky gate ou alteração de produção
e testes. O shard termina como **SHARD FOUND NO ACTIONABLE SURVIVORS** e pode ser composto
no acumulador somente depois dos gates dirigidos abaixo.

## Gates finais

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| unitários dirigidos de `dangling_contract` | PASS 14/14 |
| `semantic_preservation_v23_v25_assessment` | PASS 5/5 |
| `infrastructure_error_projection_assessment` | PASS 7/7 |
| `git diff --check` | PASS |
| mapa de autoridade | `72607b68a26f9242719b5c6234c6b3308e58ca419e45ddab06fc8cca53eb611a` |
| manifesto de vereditos | `b9be1a2245880a429eeaee2215c3e1e0b2c4b214b7ef27e5e3f771439264de17` |

Produção e testes continuam byte a byte iguais ao commit-base. O fechamento acrescenta
somente evidência segregada; merge, reinstalação e composição do P0113 permanecem fora
deste branch de shard.
