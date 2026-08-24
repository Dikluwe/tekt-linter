# Assessment 0010 — classificadores de fronteira V3/V9

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `forbidden_import.rs`, `pub_leak.rs`

## Hipótese

V3 e V9 recebem `Import` já resolvido por L3 e apenas aplicam matrizes e conjuntos.
Esperamos zero achados. Este assessment não revalida resolução de path, crate ou subdir;
ele verifica se a evidência já classificada é transformada corretamente em diagnóstico.

## Alegações sob teste

1. V3 implementa exatamente a matriz pública das sete camadas de origem pelas sete de
   destino; `Unknown` como destino nunca viola e L0/Lab/Unknown como origem são isentas.
2. `check_test_imports=false` remove somente imports com `is_test_origin=true`; `true`
   restaura a mesma matriz. `ImportKind` não altera pertinência.
3. V3 produz uma `Error` por ocorrência proibida e preserva multiplicidade, ordem,
   source path, linha, camadas e import path na evidência.
4. V9 viola exatamente quando origem é L2/L3, destino é L1 e `target_subdir` é `Some`
   não pertencente a `L1Ports`; todas as demais combinações são isentas.
5. Portas e subdirs usam igualdade textual exata, preservando caixa, Unicode, NFC/NFD
   e prefixos próximos. `None` não é convertido em subdir.
6. O guard de teste, `ImportKind`, cardinalidade, ordem e evidência de V9 seguem as mesmas
   garantias de V3; permutar entradas só permuta o mesmo multiconjunto de diagnósticos.

## Gate curto

Até seis propriedades independentes, com produtos cartesianos finitos e permutações.
Produção não é alterada. Resultado por alegação: `PASS`, `RED` ou `SPEC-GAP`.

## Segregação

- B escreve e executa o gate sem ler os dois alvos.
- C lê contrato e produção após o primeiro gate sem ler testes de B.
- O orquestrador congela achados antes de qualquer correção.

## Parada

Se houver RED, registrar evidência e parar antes de modificar L1. Se tudo passar, emitir
laudo e avançar. Não fazer merge, instalação ou release.
