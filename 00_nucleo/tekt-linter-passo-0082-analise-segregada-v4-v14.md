# Passo operacional 0082 — análise segregada de V4 e V14

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** em execução
> **Branch:** `codex/segregated-materialization`
> **Predecessor:** P0081

## Objetivo

Analisar V4 e V14 antes do merge com oráculos derivados de L0 e materializados por
agentes independentes, sem permitir que as expectativas sejam copiadas da produção.

## Papéis isolados

1. **B4 — verificador V4:** valida o hash, lê somente o L0 autorizado e contratos de
   entidades necessários, não lê `impure_core.rs`, testes, fixtures, lab ou histórico.
2. **B14 — verificador V14:** valida o hash, lê somente o L0 autorizado e contratos de
   entidades necessários, não lê `external_type_in_contract.rs`, testes, fixtures, lab ou
   histórico.
3. **C — adversário:** somente depois do congelamento dos gates, confronta L0, gates e
   produção, procurando testes tautológicos, divergências e casos de fronteira.

Os papéis B4 e B14 não leem o trabalho um do outro. Cada um pode criar apenas seu gate
novo em `tests/`. Nenhum papel altera produção.

## Insumos normativos autorizados

### B4

- caminho: `00_nucleo/prompts/rules/impure-core.md`;
- SHA-256: `efc1998d377cedd2b698b9be0ea138b52cffa7303fea7a14ba8bb67256e8e2b4`;
- seções: `Especificação`, `Listas de símbolos proibidos por linguagem`, `Estrutura da
  Violação Gerada`, `Restrições (L1 Pura)` e `Critérios de Verificação`.

### B14

- caminho: `00_nucleo/prompts/rules/external-type-in-contract.md`;
- SHA-256: `b81bd7281e09851e7586d22c561d8ac0e94f738467d460c148d28fdec52b0338`;
- seções: `Especificação`, `Nova struct — L1AllowedExternal`, `Extracção do nome do
  pacote`, `Verificação`, `Configuração`, `Restrições` e `Critérios de Verificação`.

## Alegações mínimas

- V4: todas as entradas normativas de cada linguagem, igualdade, prefixos `::` e `.`,
  near misses, linguagem desconhecida, escopo de camada, ordem, multiplicidade e
  localização.
- V14: deny-by-default, whitelist, isenções Rust, apenas `Unknown`, escopo de camada,
  origem de teste nos dois estados da configuração, extração de pacote, ordem,
  multiplicidade, mensagem e localização.
- Os gates declaram localmente suas expectativas; não importam tabelas ou helpers privados
  dos alvos.

## Parada e fechamento

Qualquer `RED` ou `SPEC-GAP` é congelado antes de correção. Depois do confronto C, emitir
assessment e decidir separadamente se há saneamento. Não fazer merge, instalação ou
release neste passo.
