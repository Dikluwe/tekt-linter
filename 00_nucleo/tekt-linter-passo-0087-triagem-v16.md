# Passo operacional 0087 — triagem segregada V16

> **Natureza:** envelope operacional temporário
> **Estado:** contrato congelado; triagem em execução
> **Branch:** `codex/audit-wildcard-saturation-v16`
> **Base:** `master@2b7e19f`
> **Assessment:** 0016

## Objetivo

Auditar integralmente o classificador puro V16, incluindo enum candidato, filtros,
classificação de corpos, exceções e determinismo, sem reabrir V17–V20.

## Protocolo

Contrato e L0 hash-pinned precedem gate. Adversário e verificador cegos derivam ataques
sem ler produção. SPEC-GAP é saneado primeiro em L0; RED autoriza correção mínima somente
depois de congelado. Adversário final verifica também causalidade e pureza Tekt.

## Limites

Permitidos: L0 V16, classificador L1, gate e documentação. Parser L3, config, CLI, wiring,
SARIF, instalação, release e merge estão proibidos.

## Saídas

Assessment 0016, gate independente, relatório adversarial e relatório P0087, terminando
em `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.
