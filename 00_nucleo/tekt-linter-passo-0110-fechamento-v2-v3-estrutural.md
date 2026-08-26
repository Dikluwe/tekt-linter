# Passo operacional 0110 — fechar V2/V3 estruturais do próprio linter

> **Estado:** ESCRITO — não executado
> **Data:** 2026-08-25
> **Branch:** `codex/p0110-structural-self-lint`
> **Baseline:** `cc357d9c3bfa26a4ce1bd71c72bc9cba5b3b027c`
> **Objetivo terminal:** auto-lint estrutural sem V2/V3, preservando a arquitetura Tekt e
> a API transacional de hashes

## 1. Horizonte finito

Corrigir exatamente dois achados restantes no auto-lint:

1. V2 em `01_core/contracts/citation_freshness.rs`: contrato L1 sem teste inline ou
   arquivo de teste reconhecido pelo self-lint;
2. V3 em `03_infra/hash_writer.rs`: L3 importa `BijectivePair` e `PairSnapshot` de L2.

O passo termina no branch com V2=0, V3=0, V16/V17=0, baseline V19/V20 ratcheted, suíte
integral verde e hashes estáveis. Não inclui merge.

## 2. Decisão arquitetural congelada

`BijectivePair` e `PairSnapshot` são valores puros de domínio/transporte:

- não executam I/O;
- não conhecem CLI, apresentação ou filesystem;
- são consumidos por planejamento L2, implementação L3 e composição L4;
- carregam paths e bytes como dados, sem decidir política de infraestrutura.

Portanto, o owner correto é L1, numa nova entidade `hash_pair`. A topologia final será:

```text
                    L1 entities::hash_pair
                    ↑                  ↑
       L2 fix_hashes                  L3 hash_writer
                    ↖                  ↗
                         L4 wiring
```

L3 não pode importar L2. L2 pode reexportar os tipos L1 temporariamente para preservar a
API pública existente, mas não pode duplicar suas definições. L4 deve compor os dois lados,
sem assumir ownership dos DTOs.

O V2 será resolvido dentro do próprio contrato L1 com `#[cfg(test)]`: os testes exercitam
as variantes fechadas e `UnknownCitationFreshness` usando somente dados em memória. Não se
adiciona filesystem a L1 e não se exclui o arquivo da regra.

## 3. Fora de escopo

- alterar a semântica de frescor, razões stale/unknown ou política fail-closed;
- mudar planejamento, ordenação, deduplicação, rollback ou atomicidade dos hashes;
- mover `HashRewriter`/`TransactionalHashRewriter` para L1;
- alterar as regras V2/V3, configuração, severidade ou diretórios excluídos;
- remover a reexportação pública L2 sem auditoria de consumidores;
- modificar Bateia, Tekt, Typst ou tekt-cargo-dsm;
- recalibrar ou reduzir o baseline V19/V20;
- push, tag, release, instalação global ou merge em `master`.

## 4. A — baseline e L0 hash-pinned

Criar Assessment 0038 antes de writes funcionais e fixar:

| Insumo | SHA-256/OID |
|---|---|
| baseline Git | `cc357d9c3bfa26a4ce1bd71c72bc9cba5b3b027c` |
| saída integral do auto-lint | `7de4b80dc24eb0fda0efa537b76392dc32c642f65dacf59ecb74ed7d23fc4de0` |
| contrato `citation_freshness.rs` | `939fddec5b3de26f3466292a123ca5fbd8cd13c165f588a9c7720736430de4ef` |
| `hash_writer.rs` | `0ff18a279f6cd9f8791fdcc4c3931e57dca7263f024c277562f955a08fd99b0a` |
| `fix_hashes.rs` | `b73350b0372363cce07f5c747cb9d8808847f8e3a4951448c761973df78d6c5d` |
| wiring | `4e2d7777f10ccc70931029bf9b92462b8867640eddab5e77b5713230433d32d3` |
| prompt citation-freshness | `ba81543d25b378349aa9b64b52a232203f8c5b65be05022d52af1729984aa7d1` |
| prompt hash-writer | `d91891b8ae9ba6fa5c38768d12ece0927d3e27668c9cb6d6f8398aa567dfeb35` |
| prompt fix-hashes | `3a11290badb2d83eaa847821a13af64bd47d3dadb13058c694919e2ebaa950fd` |

Congelar também `cargo fmt --check`, suíte completa, `fix-hashes --dry-run`, API pública
dos dois DTOs e todas as referências encontradas por `rg`. Gate A exige worktree limpo e
exatamente V2=1/V3=1.

## 5. B — gate segregado antes da produção

Materializar testes RED antes da migração:

### B1 — contrato de frescor

O gate deve provar:

- as três modalidades `Valid`, `Stale` e `Unknown` permanecem distintas;
- todas as razões stale e unknown preservam identidade, clone e igualdade;
- `UnknownCitationFreshness::resolve` é total para paths vazios, hostis e Unicode e sempre
  devolve `Unknown(Io)`;
- nenhum teste toca disco ou importa L2/L3/L4.

O RED obrigatório é V2 ainda emitido antes do bloco `#[cfg(test)]`; não inventar falha
semântica se os valores já obedecerem ao contrato.

### B2 — topologia dos DTOs

O gate deve provar:

- `BijectivePair` e `PairSnapshot` existem uma única vez em L1;
- L2 e L3 importam L1, nunca um ao outro;
- a reexportação L2 continua compilando consumidores históricos;
- construção, clone, igualdade, paths e bytes permanecem integrais;
- o self-lint emite V3 antes da migração e zero depois dela.

Classificação inicial: ambos são RED de produção. Divergência sobre o owner L1 é
SPEC-GAP e bloqueia a migração; ajuste de hash após mudança autorizada é gate.

## 6. C — fechar V2 sem ampliar L1

Adicionar testes inline a `citation_freshness.rs`, abaixo da implementação vigente. Não
alterar enums, trait ou comportamento produtivo salvo RED independente descoberto pelo
gate B1.

Critérios C:

- testes direcionados verdes;
- V2 deixa de ser emitido para o arquivo;
- zero imports de I/O, tempfile ou camada superior em L1;
- prompt causal enriquecido apenas se o critério observável ainda não cobrir totalidade e
  fail-closed; caso contrário, nenhuma edição L0.

Commit isolado: `test(p0110): close citation freshness coverage`.

## 7. D — nuclear os DTOs transacionais em L1

1. criar `01_core/entities/hash_pair.rs` com lineage L1 e testes inline;
2. criar um prompt owner individual `00_nucleo/prompts/entities/hash-pair.md`;
3. declarar o módulo em `01_core/entities/mod.rs`;
4. mover, sem redesenhar, `PairSnapshot` e `BijectivePair` de `02_shell/fix_hashes.rs`;
5. em L2, fazer `pub use crate::entities::hash_pair::{BijectivePair, PairSnapshot};` para
   compatibilidade pública;
6. em L3, importar diretamente `crate::entities::hash_pair`;
7. em L4, preferir import direto de L1 para os DTOs e manter os traits em L2;
8. atualizar testes internos para distinguir a rota canônica L1 da reexportação compatível;
9. não alterar fields, visibilidade, derives ou semântica dos valores.

O prompt novo deve declarar responsabilidade, restrições e critério observável. Ele não é
um Núcleo Tekt compartilhado: há um owner e um consumer materializado, mantendo a bijeção
prompt⇄código.

Critérios D:

- exatamente uma definição de cada DTO;
- zero import L3→L2;
- V3=0;
- V9/V14/V15/V26=0;
- testes de planejamento, execução, rollback e wiring verdes;
- API via `shell::fix_hashes::{BijectivePair, PairSnapshot}` continua compilando.

Commit isolado: `refactor(p0110): move transactional hash values to L1`.

## 8. E — confronto adversarial

Antes do resselo, confrontar:

| Hipótese | Evidência exigida |
|---|---|
| R1: V2 foi silenciado por exclusão | config e lista de arquivos inalteradas; teste inline descoberto |
| R2: tipos foram copiados, não movidos | busca nominal encontra uma definição por DTO |
| R3: reexport cria ciclo ou ownership falso | grafo de imports L1←L2/L3 e compilação de consumidor legado |
| R4: migração muda bytes/paths | igualdade estrutural e testes transacionais existentes |
| R5: L1 ganhou I/O ou política | inspeção de imports e corpo da nova entidade |
| R6: wiring passou a decidir domínio | diff L4 limitado a imports/construção tipada |
| R7: novo prompt viola bijeção | V15/V26 e manifesto owner→prompt únicos |

Todo RED é congelado antes da correção. SPEC-GAP exige decisão; não usar exceção de config.

## 9. F — resselo e fechamento

1. executar `fix-hashes --dry-run` e congelar a lista exata de pares;
2. exigir que ela contenha apenas owners realmente editados/criados e pins transitivos;
3. executar resselo real uma vez;
4. repetir dry-run e exigir `Nothing to fix`;
5. executar `cargo fmt --check`, `cargo test`, ratchet P0109, auto-lint completo,
   `git diff --check` e status;
6. exigir V1/V2/V3/V5/V7/V9/V14/V15/V16/V17/V26=0;
7. exigir V19=68/V20=17 exatamente, salvo RED novo explicitamente classificado;
8. registrar hashes finais, commits e confronto no Assessment 0038.

Estado terminal: `READY TO MERGE` com zero RED/SPEC-GAP e worktree limpo. O warning
histórico `print_tree` permanece fora do escopo.

## 10. Commits previstos

1. `audit(p0110): freeze structural self-lint baseline`
2. `test(p0110): close citation freshness coverage`
3. `refactor(p0110): move transactional hash values to L1`
4. `test(p0110): prove hash value topology and compatibility`
5. `chore(p0110): reseal structural owners`
6. `docs(p0110): close V2 V3 structural audit`

O passo termina no branch fechado. Merge, push, tag, release e instalação exigem
autorização posterior.
