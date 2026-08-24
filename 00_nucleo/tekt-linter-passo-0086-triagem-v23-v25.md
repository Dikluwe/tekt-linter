# Passo operacional 0086 — triagem segregada V23–V25

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; READY WITH RESIDUAL AUDIT
> **Branch:** `codex/audit-semantic-preservation-v23-v25`
> **Base:** `master@ce15824`
> **Assessment:** 0015

## Objetivo

Auditar os três classificadores puros de preservação semântica antes da superfície mais
ampla de contratos, parsers e wiring. O lote não reabre assessments 0001–0014.

## Segregação

1. Orquestrador congela alegações, baseline, três L0 de regra e o L0 causal da IR por
   SHA-256.
2. Adversário deriva ataques lendo somente o pacote autorizado e não edita arquivos.
3. Verificador diferente materializa gate black-box sem ler produção/testes existentes.
4. Gate e achados são congelados antes de qualquer correção.
5. SPEC-GAP é resolvido primeiro em L0/assessment; RED funcional autoriza somente a
   correção mínima correspondente.
6. Adversário final confronta L0, gate, produção e delta antes do fechamento.

Identidades de produtor registram segregação operacional, não prova formal de sandbox.

## Limites

Permitido: classificadores V23–V25, seus L0, gate e documentação do lote. Proibido:
alterar parsers, carregamento de configuração, CLI/wiring, SARIF, instalar, publicar ou
fazer merge. Um achado que exija essas superfícies vira residual ou novo passo.

## Saídas

- `00_nucleo/assessments/0015-semantic-preservation-v23-v25.md`;
- gate de integração independente;
- relatório adversarial em `lab/`;
- `00_nucleo/relatorio-p0086-triagem-v23-v25.md`;
- veredito `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Resultado

O P0086 fechou a taxonomia e o mapeamento puro V23–V25 sob quatro L0 hash-pinned. O
protocolo encontrou SPEC-GAP antes de produção, exigiu o L0 causal de `rule_traits.rs`
antes da variante nova e corrigiu um GATE-DEFECT de evidência antes do fechamento. O gate
final passou 5/5 e o adversário declarou `NÃO REABRIR` para L1.

A extração L3 e a integração permanecem residuais obrigatórias; este passo não as declara
auditadas. Estado final: `READY WITH RESIDUAL AUDIT`. Nenhum merge foi executado.
