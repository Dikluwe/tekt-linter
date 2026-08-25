# Assessment 0028-B — superfície pública de `refine-revisions`

**Papel:** B — leitura pública segregada  
**Data:** 2026-08-25  
**Decisão de vigência:** não escolhida por este papel

## Envelope e integridade dos insumos

Foram lidos exclusivamente o Assessment 0028, `README.md` e `USAGE.md`. Não foram
consultados produção, testes, prompt de refinamento, ADR-0019 nem parecer A.

| Insumo | SHA-256 observado | Resultado contra o pin do Assessment 0028 |
|---|---|---|
| `README.md` | `3ff67521214cff672b54941e1d4392b2ab933c51ed69ecc9cf5e55e8989716d6` | `PASS` |
| `USAGE.md` | `245bc38db11a29467e7e72514f488fcb69fd471d401fa7eb1b6823355fa8d4f1` | `PASS` |
| Assessment 0028 | `0cee3337ff3b6672818211291b08addf5be71ba504e4015db4b376b04f85dcc6` | observado; o Assessment não fixa o próprio hash |

## O que um usuário pode inferir hoje

1. Existe um comando público `refine` que recebe dois snapshots já materializados e um
   contrato direcional.
2. `refine` produz `PRESERVED`/exit 0, `VIOLATED`/exit 1 ou `UNKNOWN`/exit 2; o guia
   agrupa entrada inválida no exit 2 para esse comando.
3. O modo `refine` não lê Git, não executa comandos e não usa SMT.
4. Existe um comando `snapshot` capaz de produzir snapshot Rust determinístico a partir
   de uma árvore indicada por path.
5. Não há ocorrência pública de `refine-revisions`, nem sintaxe, exemplo, requisitos,
   efeitos, resultados ou códigos de saída próprios desse nome.
6. A frase “o modo não lê Git” está ligada gramaticalmente a `refine`; ela não informa
   se existe outro modo de refinamento que leia revisões. Um usuário não consegue
   concluir, somente pela documentação pública, se `refine-revisions` é inexistente,
   experimental, omitido por acidente ou vigente.

## Lacunas e ambiguidades da superfície pública

| ID | Classificação | Observação pública |
|---|---|---|
| B-PUB-01 | `SPEC-GAP` | Vigência de `refine-revisions` não é publicada nem negada. |
| B-PUB-02 | `SPEC-GAP` | Se vigente, faltam sintaxe e significado de cada revisão, contrato, path/repositório e formato. |
| B-PUB-03 | `SPEC-GAP` | Se vigente, faltam pré-condições: repositório Git, resolução admissível de revisões e tratamento de revisão ausente ou ambígua. |
| B-PUB-04 | `SPEC-GAP` | Se vigente, não se declara se o comando apenas lê objetos locais ou também altera checkout, índice, working tree, refs ou arquivos. |
| B-PUB-05 | `SPEC-GAP` | Se vigente, não se declara se pode executar Git/subprocessos, consultar rede ou materializar temporários. |
| B-PUB-06 | `SPEC-GAP` | Se vigente, não há mapeamento público de preservado, violado, inconclusivo e erro de entrada para output/exit. |
| B-PUB-07 | `SPEC-GAP` | A seção geral “Comandos CLI” em `USAGE.md` descreve somente a forma raiz e omite da sinopse tanto `snapshot`/`refine` já documentados abaixo quanto o comando em disputa. |
| B-PUB-08 | `SPEC-GAP` | README diz que `refine` não lê Git, mas não delimita explicitamente a afirmação em relação a uma eventual comparação por revisões. |

As lacunas acima descrevem somente o contrato percebido pelo usuário. Elas não provam
que qualquer comportamento exista ou deixe de existir.

## Delta documental mínimo se a autoridade decidir `CONFIRMED`

Sem escolher esse ramo, a superfície mínima coerente seria:

1. Em `README.md`, adicionar `refine-revisions` ao uso rápido e à seção de validação de
   refinamento, com uma frase que o diferencie de `refine`: snapshots fornecidos pelo
   usuário versus revisões obtidas de Git.
2. Restringir “não lê Git” explicitamente ao subcomando `refine`, evitando que a frase
   pareça uma propriedade de toda a funcionalidade de refinamento.
3. Em `USAGE.md`, publicar a sinopse nominal do comando e um exemplo mínimo.
4. Nos dois documentos, declarar os requisitos decididos pelo L0: o que identifica o
   repositório, quais formas de revisão são aceitas, como o contrato é fornecido e se a
   árvore de trabalho participa da comparação.
5. Nos dois documentos, declarar o envelope de efeitos decidido pelo L0, incluindo
   explicitamente leitura local, ausência ou presença de rede/subprocesso e garantias
   sobre checkout, índice, working tree, refs, arquivos e temporários.
6. Em `USAGE.md`, declarar resultados e exits do comando, inclusive revisão inválida,
   repositório ausente e falha de leitura, sem antecipar a política global reservada a
   F09 além do estritamente necessário para o subcomando.

Não é seguro preencher os valores desses itens a partir da superfície pública atual;
eles devem ser transcritos da decisão L0 do papel C.

## Delta documental mínimo se a autoridade decidir `REVOKED`

Sem escolher esse ramo, a superfície mínima coerente seria:

1. Em `README.md`, manter `refine` como comparação de snapshots fornecidos e acrescentar
   uma declaração curta de que comparação direta de revisões Git não integra a
   capacidade pública vigente.
2. Em `USAGE.md`, repetir essa delimitação junto à seção “Comparar snapshots por
   refinamento”, indicando o fluxo suportado: produzir/fornecer snapshots e então usar
   `refine`.
3. Não publicar sintaxe, exemplo ou promessa operacional para `refine-revisions`.
4. Preservar a afirmação de que `refine` não lê Git nem executa comandos; opcionalmente
   trocar “o modo” por “o subcomando `refine`” para remover a ambiguidade de escopo.

Nesse ramo, não há texto público existente de `refine-revisions` a remover. O delta é
uma negação explícita e localizada, suficiente para impedir que silêncio documental
seja confundido com capacidade omitida.

## Conclusão do papel B

**Resultado:** `SPEC-GAP` na superfície pública. README e USAGE são coerentes sobre
`refine`, mas não permitem determinar a vigência nem o contrato de `refine-revisions`.
Os dois ramos admitem saneamento documental pequeno; a escolha entre eles pertence à
autoridade L0 e não a este parecer.
