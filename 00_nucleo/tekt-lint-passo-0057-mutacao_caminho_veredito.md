# Laudo 0057 — estender a mutação ao caminho do veredito (fechar a higiene)

**Onde roda**: clone canônico do `tekt-linter` (com o conserto do 0052).
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/mutacao-caminho-veredito.md`](prompts/mutacao-caminho-veredito.md)
**Continuação de**: 0054–0056. Último passo da rede de regressão "para dentro".
**Camadas tocadas**: nenhuma regra/lógica mudou. Só `tests/` (8 fixtures + 8 testes)
e `crystalline.toml` (exceção de órfão do prompt).

---

## Lacuna que este laudo fecha

A mutação de 0054–0056 cobriu `01_core/rules/*.rs`, `rs_parser.rs` e
`crate_registry.rs`. Mas o veredito (multiset de `ruleId` no SARIF) também é
produzido por **config, descoberta de arquivos, leitura de prompt, despacho e
emissão** — nunca antes mutados. As fixtures os exercitavam em caixa-preta, mas
isso ≠ provar que todo mutante veredito-mudante morre. Este laudo põe esse caminho
sob mutação.

## Escopo confirmado por rastreio

`config.rs`, `walker.rs`, `prompt_walker.rs`, `prompt_reader.rs`,
`prompt_snapshot_reader.rs`, `main.rs`, `cli.rs`.
**Fora**, por decisão (selo finito e honesto): parsers de outras linguagens
(c/cpp/py/ts/zig — o corpo é Rust) e o caminho de *fix/update*
(`hash_writer`/`snapshot_writer`/`fix_hashes`/`update_snapshot`, e os ramos
`--fix-hashes`/`--update-snapshot` de `main.rs`). **O selo vale para lint de Rust.**

## Mutação — `174 mutantes` (`cargo-mutants 27.1.0`, `-j 4`)

Run inicial: `66 sobreviventes`. As 8 fixtures novas mataram os **11
veredito-mudantes**; run final: **59 sobreviventes, 0 que mudam veredito**
(`95 caught + 59 missed + 20 unviable = 174`, confere).

### Veredito-mudantes mortos (11) — fixtures novas

| Sobrevivente | Fixture | Mecânica |
|---|---|---|
| `config.rs:125` `l1_allowed_for_language` → `{}` | `v14b_pass` | `thiserror` (externo permitido) em L1; zerar a lista → V14 espúrio |
| `config.rs:139` `layer_for_module` arm `L4` | `vmod_l4_fail` | `[module_layers] wiremod="L4"`; L2 `use crate::wiremod` → V3; apagar arm → Unknown → sem V3 |
| `walker.rs:120` `resolve_file_layer` arm `L0` | `vl0_pass` | `.rs` em `00_nucleo` (L0) → sem violação; apagar arm → Unknown → V8 alien |
| `prompt_reader.rs:80/91` `exists` → `true` | `v01b_fail` | header aponta p/ prompt inexistente → V1; `exists`=true esconderia |
| `prompt_reader.rs:87` `read_hash` (Arc) → `Some(...)` | `v05b_pass` | hash correto → sem V5; hash errado → drift espúrio |
| (reforço) `allow_adapter_structs`, exclusão, recursão | `v12c_fail`, `vexcl_pass`, `vnest_fail` | botão de config V12; violação em dir excluído → 0; V1 em subdir aninhado |

### Sobreviventes restantes (59) — classificados, 0 mudam veredito

| Natureza | nº | Itens e prova |
|---|---|---|
| **Fora-de-escopo** | **51** | `main.rs` ramos `--fix-hashes`/`--update-snapshot` + impls `HashRewriter`/`SnapshotRewriter` (28) e despacho `Language::{Py,Ts,C,Cpp,Zig}` (5); `walker.rs` `language_for_path` e `check_adjacent_test` de linguagens não-Rust (18). Multi-linguagem e caminho de *fix* — escopados fora por decisão. |
| **Fora-do-oráculo** | **3** | `config::level_for` arms `fatal`/`error` (2): mudam o **nível** de V7/V11, não o `ruleId`. `cli::sarif_rule` (1): monta `tool.driver.rules` (metadado), não os `results[].ruleId` que o harness lê. |
| **Inerte / equivalente** | **5** | `config::layer_for_module` arm `L0` (1): um import-alvo L0 nunca difere de `Unknown` no veredito (L0 nunca é alvo proibido; `crate::` é isento de V14). `read_hash:31` guard de 10MB (4): mutam o limiar `10*1024*1024`; inalcançável para um prompt de tamanho normal → hash idêntico. |

## O selo, agora

**0 sobreviventes que mudam veredito em todo o caminho lint→veredito de Rust**
(regras + classificação + config + walker + prompt-IO + despacho + emissão SARIF).
O corpo de fixtures é **completo para vereditos de lint de Rust**, sem ressalva.

Continua **fora** (nomeado, não fechado): multi-linguagem, caminho de *fix/update*,
e o **oráculo de posição** (linha:coluna de violação e de erro de sintaxe) — trilhas
à parte.

## Critérios de Verificação

- [x] Pré-condição confirmada; escopo confirmado por rastreio (não pela lista do prompt).
- [x] `missed.txt` lido; total = `caught + missed + unviable` (95+59+20=174).
- [x] Cada sobrevivente em exatamente uma natureza; soma = 59 (11 mortos + 51 fora-de-
      escopo + 3 fora-do-oráculo + 5 inerte/equivalente, vs 66 do run inicial).
- [x] Fixtures novas (8) para botões de config e bordas de walker, bite-proof,
      harness afirmando IDs + contagem.
- [x] 0 sobreviventes que mudam veredito em config/walker/prompt-IO/despacho/SARIF.
- [x] Laudo registra o que ficou fora (multi-linguagem, fix/update) e que o selo é
      **"lint de Rust"**.
- [x] 0056 atualizado: selo estende de *{regras+classificação}* para *todo o caminho
      lint→veredito de Rust*, com ponteiro para cá.
- [x] Self-lint = 0; suíte verde (478 unit + 46 fixtures); nada mascarado.

## Histórico de Revisões

- 2026-06-08 — Mutação do caminho do veredito (7 ficheiros, 174 mutantes). 8 fixtures
  novas mataram os 11 veredito-mudantes (config/walker/prompt-IO). 59 restantes
  classificados: 51 fora-de-escopo, 3 fora-do-oráculo, 5 inerte/equivalente. Selo
  estendido para "lint de Rust". 0056 atualizado in loco.
