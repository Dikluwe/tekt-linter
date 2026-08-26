# Assessment 0041-S5 — fronteiras de pureza e wiring

> **Estado:** SHARD FOUND NO ACTIONABLE SURVIVORS
> **Passo:** P0113-S5
> **Branch:** `codex/p0113-s5`
> **Commit-base:** `5877f773d3bf8539065c201d410295ba6703e364`

## Escopo congelado

O shard contém exatamente 32 mutantes nos módulos L1 puros `impure_core`, `pub_leak` e
`wiring_logic_leak`. A lista ordenada de `cargo-mutants 27.1.0` tem SHA-256
`7fdb07f0b0d23d5e67aef4add5a5222fc599fc9476d8a155f864eb30afabe3ce`.

| Artefato | SHA-256 |
|---|---|
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |
| `impure_core.rs` | `7b6569f99f908c21b139e381b332e2c99ebf9e9c463b8d1415ee4585e8f1bfb9` |
| `pub_leak.rs` | `608a0202a5800935e201f061612469c92101ae6aae1d5d75346eeafdfb69948d` |
| `wiring_logic_leak.rs` | `6a52b84dee54301fed14c0a1db66ae8b3784237f9106d9111a186645b397473f` |

Os observáveis e prompts owner estão congelados em `0041-s5-authority-map.tsv`. Nenhuma
edição em produção ou testes é permitida durante a campanha cega.

## Comando congelado

```text
TMPDIR=/tmp/p0113-tmp-s5 CARGO_TARGET_DIR=/tmp/p0113-target-s5 \
  cargo mutants -j 2 --no-shuffle --no-times --output /tmp/p0113-output-s5 \
  --file 01_core/rules/impure_core.rs \
  --file 01_core/rules/pub_leak.rs \
  --file 01_core/rules/wiring_logic_leak.rs
```

## Campanha cega e classificação

O baseline passou no ambiente segregado depois da criação explícita dos quatro diretórios
vazios de fixture requeridos pelo harness. `TMPDIR`, target e saída foram exclusivos do
shard; não houve edição durante a rodada.

| Resultado | Quantidade |
|---|---:|
| `CAUGHT` | 28 |
| `MISSED` | 0 |
| `UNVIABLE` | 4 |
| `TIMEOUT` | 0 |

| Artefato | SHA-256 |
|---|---|
| `mutants.json` | `5edf45a745974780dd753235aaf826e330414f8d038e81355827692140715fd3` |
| `outcomes.json` | `dfabb500d649a807c7b8a6bbe64f53293fbcf6ce9da55cdbfb773c2003e2671e` |
| `caught.txt` | `e846738c9de2020bf5f55c1da0051231fd54dae94522460ccd09277bc9db3050` |
| `missed.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `timeout.txt` vazio | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `unviable.txt` | `c184fd1dd72550759da93402f793e07252e70dc734cf71a4ba23082ee5b016ac` |

Não existe `MISSED`; portanto não há reprodução serial nem `TEST-GAP` a sanear. Os quatro
não mortos estão individualizados em `0041-s5-mutation-verdicts.tsv`. Todos tentam
construir `Violation::default()` e são rejeitados por `rustc` com E0277 porque
`Violation` não implementa `Default`. Foram classificados `TOOL-LIMIT`, sem serem
contados como mortos ou convertidos em evidência semântica.

## Decisão

Os gates existentes mataram todos os 28 mutantes compiláveis. Não houve
`PRODUCTION-RED`, `SPEC-GAP`, `ARCH-RED`, flaky gate ou mudança de produção/testes. O S5
termina como **SHARD FOUND NO ACTIONABLE SURVIVORS**.

## Gates finais

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| unitários dirigidos de `impure_core` | PASS 15/15 |
| unitários dirigidos de `pub_leak` | PASS 9/9 |
| unitários dirigidos de `wiring_logic_leak` | PASS 13/13 |
| `declaration_state_classifiers_assessment` | PASS 6/6 |
| `git diff --check` | PASS |
| mapa de autoridade | `34929ab90faf96a778ad0cc5e1e53aec631a23ad06766f83394b813a19532e31` |
| manifesto de vereditos | `e860265cbe175f253e48da533166f878bab90801048e9426559c8c04be7d02a1` |

Produção e testes permanecem byte a byte iguais ao commit-base. O branch contém somente
evidência segregada; composição, merge e reinstalação não fazem parte deste shard.
