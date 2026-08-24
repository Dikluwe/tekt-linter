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

## Resultado da triagem

O gate independente terminou com duas propriedades verdes e duas descobertas:

- catálogo único V0–V25, referências, níveis, posições e ordem recebida no SARIF
  passaram;
- a tabela completa e a monotonicidade de `should_fail` passaram;
- `sort_violations` não desempata coluna, rule id ou mensagem; permutar violações com
  a mesma severidade/path/linha muda a saída final;
- paths Unix com bytes não UTF-8 perdem identidade tanto no texto quanto no SARIF,
  embora mensagens e paths UTF-8 hostis façam round-trip corretamente.

O primeiro RED é defeito contra a alegação explícita de ordem total determinística. O
segundo fica como `SPEC-GAP`: SARIF representa URI como string Unicode, e o contrato
ainda não escolhe percent-encoding, URI de bytes ou rejeição explícita. Corrigir sem
essa decisão apenas substituiria uma conversão implícita por outra.

Também foi confirmada uma divergência histórica sem falha atual: o prompt ainda fala
em V0–V12, enquanto a API publica catálogo coerente V0–V25. A atualização documental
deve acompanhar a futura correção, mas não invalida o gate atual.
