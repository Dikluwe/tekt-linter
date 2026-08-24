# Relatório P0084 — fechamento pré-merge

**Data:** 2026-08-24
**Branch:** `codex/segregated-materialization`
**Veredito:** READY WITH RESIDUAL AUDIT

## Conclusão

O branch foi fechado como um ciclo coerente, não como auditoria integral do linter. Seus
REDs e SPEC-GAPs conhecidos foram rastreados aos saneamentos P0072–P0083, e os assessments
0001–0012 agora refletem os gates finais existentes.

O adversário independente não encontrou RED funcional atual. Seu bloqueio inicial era
documental e mecânico: estados antigos, saídas ainda ausentes e whitespace. Esses pontos
foram corrigidos sem alterar produção.

## Evidência final

- base: `75a56656a2e8cd0df4d0678eab9e78291ec34506`;
- HEAD congelado do protocolo: `1b7e18f`;
- superfície congelada: 207 arquivos, +14.771/-568 linhas;
- suíte: 628 unitários, 83 fixtures e todos os gates de integração verdes;
- hashes do linter: estáveis;
- auto-lint V1/V5/V7: limpo;
- diff contra a base: sem whitespace inválido;
- Rust novo: formatado;
- smoke Typst: regras tocadas e passagem sem V5/V6 com exit 0;
- worktree Typst: fingerprint idêntico antes/depois;
- 415 hashes Typst apenas reportados em dry-run.

## Resíduos deliberadamente externos

1. Auditoria das regras/componentes não incluídos nos assessments 0001–0012.
2. Formatação global de arquivos legados que já divergem do rustfmt atual.
3. Atualização dos 415 hashes do Typst Cristalino, que possui trabalho em andamento.
4. Matrizes condicionais de plataforma e permissões já registradas nos relatórios dos
   lotes correspondentes.

Esses resíduos não contradizem as propriedades fechadas neste branch e devem entrar em
branches menores. Nenhum merge, instalação ou release foi executado.
