# Assessment 0041-S4 — mutação de padrões, limites e profundidade

**Estado:** SHARD FOUND NO ACTIONABLE SURVIVORS  
**Data:** 2026-08-26  
**Passo:** P0113-S4  
**Branch:** `codex/p0113-s4`  
**Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## Escopo e autoridade congelados

O shard executou somente `deep_pattern_nesting`, `compound_guard`, `range_pattern` e
`or_pattern_alternatives`, todos L1 puros. O mapa de autoridade está em
`0041-s4-authority-map.tsv`. Nenhuma política foi movida para outra camada.

| Insumo | SHA-256 |
|---|---|
| lista congelada dos 35 mutantes | `12683903693d10a66432545f60954cf0a04b3b0ab279404b0b63590074f68cf7` |
| `deep_pattern_nesting.rs` | `9b32c9892f59ab319495868cd0968c63f90ab147d343853886827f127db923aa` |
| `compound_guard.rs` | `fdea0d4734a9bd707c97f6e99cfdfce57f3b6306802221453a1436c1335846b1` |
| `range_pattern.rs` | `750969e0b6c789eee94a9dd6120328baeb68923a3cbf71417103a4764d0f5f53` |
| `or_pattern_alternatives.rs` | `2579227c98b7b4274a08b65844a8f66f4166d650bb320f9972cf14929ccebb99` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |
| prompt `deep-pattern-nesting.md` | `b4a4acabc362920561bec12f95ddcb99a0fcf68c5d53759b9ff3e6ab1d5060e5` |
| prompt `compound-guard.md` | `d5ef806723eea38137c8c71ace80057cd7c8e79aa7d4ef7696fa3b72b9ea1a98` |
| prompt `range-pattern.md` | `d4d826afc3c97c516f680a820c7d124ee9d86cd595df4c35946ba14f76ee3769` |
| prompt `or-pattern-alternatives.md` | `91c409539ea603c2e4ae1aa4932e6bedddb209991652b4a345dbbe7e3b159620` |

Ferramenta: `cargo-mutants 27.1.0`. Antes da execução foram restauradas localmente as
quatro folhas vazias de fixture que Git não materializa em worktrees. Elas permaneceram
sem conteúdo e não integram o diff.

## Rodada cega segregada

Comando executado com `TMPDIR=/tmp/p0113-tmp-s4`,
`CARGO_TARGET_DIR=/tmp/p0113-target-s4`, saída exclusiva em
`/tmp/p0113-output-s4`, dois jobs, `--no-shuffle` e `--no-times`.

| Resultado | Quantidade |
|---|---:|
| `CAUGHT` | 34 |
| `MISSED` | 0 |
| `UNVIABLE` | 1 |
| `TIMEOUT` | 0 |

| Artefato | SHA-256 |
|---|---|
| `mutants.json` | `afa500c5accf1682831b10e4bcca2041fb4b398378e3d469f3f814bddfb53ce7` |
| `outcomes.json` | `d3054155e1a22682629d00dbbb96d6a0b949ed22f563d3e8746cd6594989d066` |
| `caught.txt` | `1f5bc157223024e206e290ed1a4c5f09b526854f3dcda3d4bb8e9dc712ff1e11` |
| `missed.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `timeout.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `unviable.txt` | `c258bfb078b510581cf5cdf4b2697b7c3eda754da78aa45bfbb3fcc86501898f` |
| `debug.log` | `4948246ebc9e4eab99223787a98af592a346236df4e15c44404bad5cb107248f` |

Como `MISSED=0`, não há sobrevivente a reproduzir em série e não existe `TEST-GAP` a
sanear. O único não morto tenta construir `Violation::default()`, mas `Violation` não
implementa `Default`; está integralmente registrado em `0041-s4-mutation-verdicts.tsv`
como `TOOL-LIMIT`. Ele não foi contado como mutante morto nem como evidência semântica.

## Decisão

Os gates existentes mataram todos os 34 mutantes compiláveis do universo fechado,
incluindo inversões dos máximos inclusivos, profundidade, precedência de contexto,
exceções de caminho, linguagem, ordem e cardinalidade. Não houve `TEST-GAP`,
`PRODUCTION-RED`, `SPEC-GAP`, `ARCH-RED`, timeout ou flakiness. Produção e testes
permanecem byte a byte inalterados.

P0113-S4 termina como **SHARD FOUND NO ACTIONABLE SURVIVORS**. Esta conclusão vale apenas
para os 35 mutantes congelados e não constitui certificação global das quatro regras.

## Gates finais

Após a campanha, o primeiro teste dirigido encontrou o último mutante ainda compilado no
cache do target segregado, embora a fonte já estivesse restaurada e sem diff. O target
`/tmp/p0113-target-s4` foi limpo integralmente com `cargo clean --target-dir` e todos os
gates foram repetidos a partir das fontes restauradas. O evento é `GATE-DEFECT` operacional
saneado; não altera os resultados da rodada cega.

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| teste público dirigido | PASS — 10/10 |
| `cargo test --all-targets` | PASS — 635 unitários e toda a suíte de integração |
| ratchet P0109 | PASS — 2/2 dentro da suíte |
| auto-lint completo | exit 0; V19=68, V20=17, demais regras zero |
| `fix-hashes --dry-run` | `Nothing to fix` |
| `git diff --check` | PASS |
| manifesto de autoridade | `a05817b15c45a1cd1bfd2f3d8962e7b9edb9d595c39139b85c34e8a8722b643b` |
| manifesto de vereditos | `f6d7a31aac95ed536faa37a8e2bc6af7a13d81022539a82c55e6fee56c9a8410` |
