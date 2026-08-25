# Relatório P0091 — auditoria do roteamento MultiParser

**Data:** 2026-08-24
**Branch:** `codex/audit-multiparser-routing`
**Baseline pós-P0090:** `13180b1`
**Resultado:** `READY WITH RESIDUAL AUDIT`

## Descoberta

O lote começou bloqueado por especificação. Os L0 autorizados só roteavam Rust,
TypeScript e Python; as outras seis linguagens existiam na implementação e em prompts de
regras, mas não numa matriz causal de composição. Também havia contradição entre “L4
seleciona” e “L4 zero lógica de negócio”, sem API L1 ou seam segura para spies.

O L0 passou a normatizar:

- enum completo de nove linguagens mais `Unknown`;
- `ParserSlot` e `parser_slot(Language)` puros em L1;
- `ParserSet` total sobre nove ports `LanguageParser` obrigatórios;
- chamada única, mesmo empréstimo, propagação exata e fallback sem parser;
- L4 restrito à instanciação dos adapters e início da composição.

## RED e correção

O gate cego compilou somente até detectar a ausência das três APIs normativas. Depois do
resselamento de um gap documental no schema de `ParsedFile`, esse foi o único RED. O
confronto confirmou que `MultiParser` continha o `match` de decisão em L4.

O commit `74de284` introduziu a política e composição sobre ports em
`01_core/contracts/language_parser.rs`. O `MultiParser` privado permanece em L4 apenas
como proprietário dos tipos concretos L3 e constrói um `ParserSet` por empréstimo; não
repete matriz, fallback ou precedência.

## Evidência

| Gate | Evidência | Resultado |
|---|---|---|
| B1/B2 inicial | `tests/multiparser_routing_assessment.rs` | 3/3 PASS |
| B2 independente | `tests/multiparser_composition_assessment.rs` | 3/3 PASS |
| workspace | 628 unitários + integrações + 83 fixtures | PASS |
| auto-lint | V4/V5/V11/V12 | nenhuma violação |
| lineage | `--fix-hashes --dry-run` | `Nothing to fix` |

O B2 independente usa nove spies, sentinelas não vazias em todos os campos de
`ParsedFile`, comparação integral por `PartialEq`, nove caminhos `Ok`, nove `Err` e
`Unknown` com zero chamadas. Nenhum parser real funciona como oráculo.

## Fechamento adversarial

O primeiro D aprovou a produção, mas encontrou dois `GATE-DEFECTs`: identidade comum de
B1/B2 e cobertura parcial do `Ok`. Ambos foram fechados antes da repetição. O segundo D
confirmou arquitetura Tekt, causalidade do RED, hashes, ausência de I/O/tipos L3 em L1,
wrapper L4 transparente e regressão global.

O residual é epistemológico: Git não prova sozinho a identidade/ausência de leitura do
agente cego. O histórico segregado e o gate produzido não mostram contaminação. Não houve
merge, push, instalação ou release.
