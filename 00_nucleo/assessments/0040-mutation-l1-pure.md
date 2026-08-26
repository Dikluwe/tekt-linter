# Assessment 0040 — mutação dirigida do núcleo L1 puro

**Estado:** BATCH FOUND NO ACTIONABLE SURVIVORS
**Data:** 2026-08-26
**Passo:** P0112
**Branch:** `codex/p0112-mutation-l1-pure`
**Baseline/merge P0111:** `e3dab595dbbe25e2690a482e0840335593ef7a5d`

## Precondição e instalação

- P0111 e a especificação P0112 foram integrados em `master` por merge não fast-forward;
- branch P0112 nasceu diretamente do mesmo commit de `master`;
- `cargo install --path . --force` substituiu o binário instalado;
- SHA-256 do binário: `ec54607d4de92edacfac2e25c0ed390b3743c58cb86eb88e969ab0021833ec32`.

## L0 hash-pinned

| Insumo | SHA-256 |
|---|---|
| lista repetida dos 38 mutantes | `16d066ef9a3cda926b2db4ec04739f1115ffd8800308bc2043b2491648213f35` |
| manifesto das duas campanhas imediatamente anteriores | `92cda0db21fe88e928455cf65cabca3918c54217729c4639323dd6398539567f` |
| `l1_allowed_external.rs` | `dbbbb809e0c66f7fcbb7a349695037be381352a4d7ed743ce14ffcebd97a9bfd` |
| `alien_file.rs` | `bd7b31b9c2547205087c7c170a53f8d86a97e465aaf93d1fe0ba3580f6212368` |
| `test_file.rs` | `6072016519b73d6df8831dc4cfe7cc8f97ee94fd3116e8eda963e338fca86d1f` |
| `quarantine_leak.rs` | `b50466a01b76c9c4eebdf92fa05685d6022843266b867cc11ac736c933a8ba97` |
| `orphan_prompt.rs` | `ecba43f507a415762d7a25e8b0bca0aff23d52d632c261bf4f6ad928e52989e4` |
| `mutable_state_core.rs` | `7ec860d6197e44a0f3dd1bd8f96d97a27f831fa3888035219ae43ff17d6a2382` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

Ferramentas: `rustc 1.92.0`, `cargo 1.92.0`, `cargo-mutants 27.1.0`.

As saídas `mutants.out` e `mutants.out.old` de P0111 foram preservadas como
`mutants.out.pre-p0112-current` e `mutants.out.pre-p0112-old`. O manifesto externo contém
17 hashes. Nenhuma campanha histórica foi removida.

## Baseline operacional

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test --all-targets` | PASS — 635 unitários e toda a suíte de integração |
| ratchet P0109 | PASS 2/2 dentro da suíte |
| auto-lint | exit 0; V19=68 e V20=17 |
| hash da saída do auto-lint | `92c51980f1574d87359a810c27c29b40c1a84b5a7119bfab2690d1277ab622c8` |
| `fix-hashes --dry-run` | `Nothing to fix` |
| hash da saída do dry-run | `bf3511c5c7cb1a9202071d8a2d60204a30f29cc11ee58ff2963a063a9f835b25` |

Não existe RED, `GATE-DRIFT` ou `SPEC-GAP` no baseline. Nenhuma produção ou teste foi
editado antes da rodada cega.

## Rodada cega e classificação

Os 38 mutantes fixados foram executados uma vez com quatro jobs e sem shuffle. Resultado:

| Resultado | Quantidade |
|---|---:|
| `CAUGHT` | 22 |
| `MISSED` | 0 |
| `UNVIABLE` | 16 |
| `TIMEOUT` | 0 |

| Artefato | SHA-256 |
|---|---|
| `mutants.json` | `6f691944ddf506ecb656eb7ab38540ff8d7ad40c99bcaf7cce3918b7fd80af1b` |
| `outcomes.json` | `a7b1d1a77c59cf813228eddd1246ad716dc030ecd92ae1d68563f6c013da3c8b` |
| `caught.txt` | `6bc5ee5706e935610a0f22523099720ce988f77ff9d729bbace5ff35e42e5269` |
| `missed.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `unviable.txt` | `51143160b8dad4398951ad85621ea85a3c5e2e510f11e455934efe89d8079ba8` |
| `timeout.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Como `MISSED=0`, não existe sobrevivente a reproduzir serialmente nem `TEST-GAP` a
sanear. Os 16 inviáveis foram inventariados integralmente em
`0040-mutation-verdicts.tsv` como `TOOL-LIMIT`: onze substituições não satisfazem o tipo ou
lifetime de `Self`; cinco tentam construir `Violation::default()`, inexistente por contrato.
Inviável não foi contado como morto nem convertido em alegação semântica.

## Decisão

Neste universo fechado, os testes existentes mataram todo mutante compilável. P0112
responde **não há sobreviventes acionáveis**. Isso não prova os módulos, não atribui score
de 100% e não autoriza campanha infinita. Não houve `PRODUCTION-RED`, `SPEC-GAP` ou
`ARCH-RED`; produção e testes permanecem byte a byte inalterados.

Um eventual P0113 exige nova decisão baseada no sinal combinado: P0111 encontrou 57 gaps,
enquanto este lote puro pequeno encontrou zero. Não há promoção automática para CI.

## Gates finais

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test --all-targets` | PASS — 635 unitários e toda a suíte de integração |
| ratchet P0109 | PASS 2/2 |
| auto-lint completo | exit 0; hash idêntico ao baseline; V19=68/V20=17 |
| `fix-hashes --dry-run` | `Nothing to fix`; hash idêntico ao baseline |
| `git diff --check` | PASS |
| manifesto `0040-mutation-verdicts.tsv` | `b31a21ae03ad88937ee74ea59906709ec64f5d221c40b431111a27cba3d9db60` |

P0112 termina sem RED, gate, `SPEC-GAP` ou mudança funcional. Branch apto a merge; a
integração não está incluída automaticamente neste fechamento.
