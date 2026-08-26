# Assessment 0038 — fechamento estrutural V2/V3

**Estado:** BASELINE CONGELADO — saneamento pendente
**Data:** 2026-08-25
**Passo:** P0110
**Branch:** `codex/p0110-structural-self-lint`
**Baseline funcional:** `cc357d9c3bfa26a4ce1bd71c72bc9cba5b3b027c`
**Commit do passo:** `39ad2e0`

## L0 hash-pinned

| Insumo | SHA-256 |
|---|---|
| saída integral do auto-lint | `7de4b80dc24eb0fda0efa537b76392dc32c642f65dacf59ecb74ed7d23fc4de0` |
| `citation_freshness.rs` | `939fddec5b3de26f3466292a123ca5fbd8cd13c165f588a9c7720736430de4ef` |
| `hash_writer.rs` | `0ff18a279f6cd9f8791fdcc4c3931e57dca7263f024c277562f955a08fd99b0a` |
| `fix_hashes.rs` | `b73350b0372363cce07f5c747cb9d8808847f8e3a4951448c761973df78d6c5d` |
| wiring | `4e2d7777f10ccc70931029bf9b92462b8867640eddab5e77b5713230433d32d3` |
| prompt citation freshness | `ba81543d25b378349aa9b64b52a232203f8c5b65be05022d52af1729984aa7d1` |
| prompt hash writer | `d91891b8ae9ba6fa5c38768d12ece0927d3e27668c9cb6d6f8398aa567dfeb35` |
| prompt fix hashes | `3a11290badb2d83eaa847821a13af64bd47d3dadb13058c694919e2ebaa950fd` |

Baseline verde para `cargo fmt --check`, ratchet P0109, `fix-hashes --dry-run` e
`git diff --check`; worktree limpo. O auto-lint contém exatamente V2=1, V3=1, V19=68 e
V20=17.

## Classificação segregada

| ID | Achado | Classe | Decisão |
|---|---|---|---|
| R-V2 | contrato L1 sem verificação simultânea reconhecida | RED de produção | testes inline puros |
| R-V3 | L3 importa DTOs de L2 | RED de produção | mover DTOs para entidade L1 |

Não há SPEC-GAP: P0110 congelou L1 como owner dos valores puros compartilhados. Alterar
configuração, regra ou exclusões seria RED do gate.

## Hipóteses adversariais

- H1: teste V2 toca disco ou camada superior;
- H2: DTOs ficam duplicados após a migração;
- H3: compatibilidade pública L2 deixa de compilar;
- H4: fields, derives, bytes ou paths mudam;
- H5: L3 continua importando L2 por rota indireta;
- H6: o novo prompt cria owner compartilhado ou rompe bijeção;
- H7: o resselo toca superfície não causal.

Todas permanecem bloqueantes até os gates finais.
