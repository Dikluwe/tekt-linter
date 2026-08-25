# Relatório P0094 — auditoria do relatório N16 por módulo

**Data:** 2026-08-25
**Branch:** `codex/audit-n16-summary`
**Baseline:** `fee11de`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Escopo

O lote auditou a seam L2 `n16_summary`: gramática de tags, identidade de localização,
deduplicação, agrupamento, ordenação, percentuais, avisos e seleção do formato pelo
consumidor L4. V16, parsers e leitura L3 permaneceram fora do oráculo funcional.

## SPEC-GAP e saneamento

O adversário A encontrou cinco `SPEC-GAP`, duas contradições e apenas um ponto decidido.
O L0 passou a fixar token único, parser pelo último `:`, fonte como precedente,
componentes de módulo, ordem γ/nome, half-up, vazio, aviso para todo módulo pequeno e
exigência de V16.

B1/B2 recusaram corretamente duas APIs incompletas antes de escrever gates. Após publicar
a seam nominal e `SourceFile`, o primeiro gate revelou um caminho público L1 incorreto;
isso foi classificado `GATE-DEFECT` e corrigido sem criar namespace `core` artificial.

## Matriz causal

| Papel | Evidência |
|---|---|
| A — L0 | hashes validados; G1–G8 saneados |
| B1 — coleção | `bb6cde4e…`, RED 0/4 → GREEN 4/4 |
| B2 — apresentação | `d2e4e61b…`, RED 2/6 → GREEN 6/6 |
| C — produção | `94d71e2` |
| D — fechamento | PASS adversarial |

O commit `9373184` preserva o import defeituoso; `990451c` corrige somente dois imports e
expõe o RED semântico antes da produção. Assim, o erro do gate e o defeito produtivo têm
causalidades separadas.

## Arquitetura Tekt

| Camada | Responsabilidade confirmada |
|---|---|
| L1 | `SourceFile`, `Language` e `Layer`, sem formato |
| L2 | agregação, taxonomia de apresentação, formatter e validação CLI |
| L3 | leitura de fontes e configuração |
| L4 | seleção, injeção e política global de exit status |

L2 não lê filesystem/configuração/ambiente; L4 não duplica classificação, agrupamento ou
percentuais; o formato não cria regra V nem muda severidades.

## Validação

- workspace: 629 unitários e todas as integrações/fixtures PASS;
- gates P0094: 10/10 PASS;
- consumidor: exit 0 com V16, exit 1 sem V16, outras violações preservam exit 1;
- auto-lint V5/V6/V7/V12: nenhuma violação;
- reparador de hashes: `Nothing to fix`;
- hashes L0, `rustfmt` dirigido e `git diff --check`: PASS.

Resíduos: não há gate CLI versionado para o consumidor, apenas smoke adversarial; somas
artificiais acima de `usize::MAX` podem overflow, fora do domínio dos gates e da coleta
real. Nenhum merge ou push foi realizado.
