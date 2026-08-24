# Assessment 0016 — revisão adversarial segregada V16

**Produtor:** `adversary/v16/0016`
**L0 validado:** SHA-256 `5941adf0c444a65e101224dacfdb1fea0cbafebf46a5a9ac6be5bed25063cc08`
**Resultado inicial:** `SPEC-GAP`

## Achados congelados

1. API autorizada nomeia V17–V20, não o módulo/check V16.
2. Não há API black-box para injetar `[wildcard_exceptions]`.
3. IR textual omite `ScrutineeForm::Other`, embora o gate exija sete formas.
4. “Catch-all ativo” não distingue presença sintática de elegibilidade após filtros.
5. Matching de path não define exatidão, base, relativo/absoluto, separadores ou case.
6. Justificativa inválida não define trim/case e fronteira de `ok`.
7. Warnings derivados de HashMap não têm ordem canônica nem posição relativa definida.
8. Escopo de obsolescência para exceções de outros arquivos não está definido.
9. Enum candidato não define prefixo textual, duplicata intrabraço e braços distintos.
10. Termos multilíngues coexistem com escopo Rust-only sem declarar fase futura.
11. “DENY-class” pode ser confundido com Error, embora nível inicial seja Warning.
12. Truncamento do snippet pertence ao parser, mas a responsabilidade no gate é ambígua.

## Ataques preservados

- sete scrutinees; mesmo prefixo em braços distintos, divergentes e duplicata intrabraço;
- produto catch-all/reincorporação/barreira/candidato e todos os BodyForm;
- exceptions válida, vazia, `ok`, tag N16, obsoleta, outro arquivo, path parecido,
  relativo/absoluto e ordens de inserção diferentes;
- ordem de expressões/braços, multiplicidade, evidência, Unicode e limites;
- campos V17–V20 ativados sem vazamento de rule_id.

Nenhum RED funcional é alegado até o L0 tornar o gate integral executável.
