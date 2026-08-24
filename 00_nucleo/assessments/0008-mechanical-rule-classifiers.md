# Assessment 0008 — classificadores mecânicos V2/V8/V10/V11

**Estado:** SANEADO PELO P0076 — gate 6/6
**Data:** 2026-08-24
**Alvos:** `test_file.rs`, `alien_file.rs`, `quarantine_leak.rs`,
`dangling_contract.rs`

## Hipótese

Estas regras L1 não interpretam sintaxe nem acessam I/O: classificam booleans, camadas
e conjuntos já produzidos pelas fronteiras saneadas. Esperamos zero achados. Um RED
mostraria que até predicados mecânicos podem distorcer evidência entregue aos agentes.

## Alegações sob teste

1. V2 emite exatamente uma `Error` somente para `L1 && !has_test_coverage`; todas as
   demais camadas e cobertura positiva são isentas, preservando path e posição pública.
2. V8 produz exatamente uma `Fatal` por alien canônico recebido, sem perder, inventar
   ou reordenar identidade, e o vazio é identidade.
3. V10 emite uma `Fatal` por import cujo alvo é `Lab` quando a origem está em L1–L4;
   L0, Lab e Unknown são isentos, e outros alvos nunca são V10.
4. V10 preserva multiplicidade, path do arquivo, linha e texto do import; permutar a
   entrada só pode permutar diagnósticos equivalentes, não mudar seu multiconjunto.
5. V11 calcula exatamente `declared - (implemented ∪ blanket)`, propaga o nível injetado
   e produz ordem determinística, sem depender da ordem de inserção ou de duplicatas.
6. Nos quatro classificadores, rule id, severidade/nível, localização e evidência da
   mensagem permanecem completos para entradas Unicode e representações textuais
   distintas; nenhuma normalização nova ocorre em L1.

## Gate curto

Até seis propriedades independentes e zero alterações de produção. Usar dublês das
traits públicas, tabelas exaustivas pequenas e permutações. Fronteiras já decididas em
L3 não devem ser reimplementadas no teste. Resultado por alegação: `PASS`, `RED` ou
`SPEC-GAP`.

## Segregação

- B escreve e executa o gate sem ler os quatro alvos.
- C, após o primeiro gate, lê contrato e produção, mas não os testes de B.
- O orquestrador congela achados antes de qualquer saneamento.

## Parada

Se houver RED, registrar evidência e parar antes de alterar L1. Se tudo passar, emitir
laudo de triagem e avançar. Não fazer merge, instalação ou release.

## Resultado da triagem

O gate B terminou em 5 PASS / 1 RED / 0 ignored. V2, V8 e V10 passaram integralmente;
V11 calcula corretamente `declared - (implemented ∪ blanket)`, preserva nível e
representações textuais, mas emite diretamente na ordem de um `HashSet`.

O adversário C, sem ler o gate, construiu 128 índices semanticamente idênticos e observou
128 sequências públicas distintas. O achado viola a alegação 5 sem `SPEC-GAP`.

A produção permaneceu intacta. O saneamento seguinte deve ordenar textualmente as traits
pendentes antes de materializar violações, preservando distinções byte-sensitive.

## Fechamento P0076

V11 passou a ordenar as traits pendentes pelo `Ord` nativo de `str` antes de construir
violações. O gate endurecido passou 6/6 e o adversário, após 512 reconstruções do mesmo
conjunto, encerrou com **NÃO REABRIR**. O RED acima permanece como evidência histórica.
