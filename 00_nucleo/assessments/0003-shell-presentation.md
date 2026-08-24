# Assessment 0003 — apresentação textual e SARIF

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvo:** API pública de apresentação em `02_shell/cli.rs`

## Hipótese

Formatar diagnósticos e decidir exit code parece trabalho mecânico e já possui testes.
Esperamos zero achados. Um RED reclassifica toda saída consumida por CI ou agentes,
porque diferenças de ordem ou perda de conteúdo afetam comparação e automação.

## Alegações sob teste

1. `sort_violations` produz ordem total determinística: severidade descendente, path,
   linha, coluna, rule id e mensagem; permutar a mesma coleção não altera a saída.
2. `format_text` e `format_sarif` preservam a ordem recebida e todo conteúdo de path,
   regra e mensagem, incluindo Unicode, aspas, barras e caracteres de controle.
3. SARIF é JSON 2.1.0 válido, usa coluna base 1, mapeia Fatal/Error para `error`,
   Warning para `warning` e Info para `note`, e possui metadado para todo V0–V25.
4. `should_fail` é monotônico: Fatal sempre bloqueia; Error bloqueia nos dois modos;
   Warning somente em `warning`; Info nunca bloqueia.

## Gate curto

Até quatro propriedades independentes, sem alterar produção. O prompt histórico pode
estar defasado quanto à quantidade de regras; divergência documental é `SPEC-GAP`, não
falha automática do formatador atual.
