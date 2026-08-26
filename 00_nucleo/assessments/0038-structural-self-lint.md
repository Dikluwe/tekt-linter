# Assessment 0038 — fechamento estrutural V2/V3

**Estado:** READY TO MERGE — zero RED/SPEC-GAP aberto
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

## Execução e confronto

- `1abde37`: adicionou dois testes puros ao contrato de frescor; V2 passou de 1 para 0;
- `d812380`: moveu os dois DTOs para `entities::hash_pair`, preservou a reexportação L2 e
  alterou L4 somente nos imports; V3 passou de 1 para 0;
- `caaca09`: resselou exclusivamente o novo par owner/prompt.

H1–H7 foram refutadas:

- os testes L1 não importam I/O nem camada superior;
- busca nominal encontra uma definição de cada DTO, ambas em L1;
- testes históricos continuam compilando pela reexportação de L2;
- fields, derives, paths e bytes foram movidos sem alteração;
- `hash_writer.rs` não importa L2;
- o novo prompt tem owner exclusivo e V15/V26 estão zerados;
- o resselo tocou somente `hash_pair.rs` e `hash-pair.md`.

## Gates finais

| Gate | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo test` | PASS — 634 unitários e toda a suíte de integração/doc-tests |
| testes `citation_freshness` | PASS 2/2 |
| testes `hash_pair` | PASS 2/2 |
| transação/bijeção histórica | PASS 10/10 |
| ratchet P0109 | PASS 2/2 |
| auto-lint completo | exit 0; V19=68 e V20=17 |
| V1/V2/V3/V5/V7/V9/V14/V15/V16/V17/V26 | zero |
| `fix-hashes --dry-run` | `Nothing to fix` |
| `git diff --check` | PASS |

SHA-256 da saída integral final do auto-lint:
`92c51980f1574d87359a810c27c29b40c1a84b5a7119bfab2690d1277ab622c8`.

Não resta RED, gate ou SPEC-GAP do P0110. O branch está apto ao merge conjunto dos
fechamentos P0109/P0110.
