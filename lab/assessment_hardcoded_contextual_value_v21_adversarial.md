# Assessment 0017 — revisão adversarial segregada V21

**Produtor:** `adversary/v21/0017`
**Resultado inicial:** `SPEC-GAP`

Os quatro hashes do Assessment 0017 foram validados. Nenhum L1–L4, teste, histórico ou
relatório foi lido durante a derivação.

## SPEC-GAPs congelados

1. API causal não publica integralmente `HasConstants`, configuração nem porta de
   frescura `valid/stale/unknown`.
2. `UNKNOWN` não possui tipo, severidade, mensagem ou política executável.
3. Validade de `Spec`/`Rationale` vazia, whitespace ou malformada não está definida.
4. Identidade de format/strict modules usa semântica de path não publicada.
5. Gramática/root/absoluto/`..`/dois-pontos/precedência de `Ref` estão abertos.
6. Confinamento, symlink e TOCTOU da porta não estão decididos.
7. Encoding, orçamento e limite de leitura não estão definidos.
8. Linha “não vazia”, whitespace, CRLF e contagem de linhas não estão definidos.
9. Direção/associação de scaling, negativos e equivalência lexical de triviais têm
   fronteiras abertas.
10. StaleCitation não fixa severidade, evidência nem emissão única/dupla.
11. Janela de proximidade de três linhas não fixa direção/inclusividade/precedência.

## Matriz preservada

- B1: languages; produto 2³ do predicado; format/strict/test/table; triviais e controles;
  None/Spec/Rationale/Ref nos três estados; níveis; evidência; ordem; pureza e isolamento.
- B2: arquivo/linha válida, ausente, zero, EOF, vazia; root/escape/symlink; erros;
  Unicode/CRLF/UTF-8 inválido/orçamento; determinismo, fingerprint e ausência de efeitos.

Nenhum RED funcional é alegado antes do saneamento. Os dois gates permanecem fail-closed
até L0 V21 e novo L0 da porta publicarem política e API completas.

## Fase D — confronto final

**Delta confrontado:** `cc0691d..505c312`

**Veredito:** `BLOCKED`

Os sete insumos do Assessment 0017 conferem por SHA-256. A ordem causal também está
preservada no histórico: `8c61425` publica L0/API, `f09cee0` congela os gates e somente
`505c312` altera L1/L3/L4. O classificador V21 não contém filesystem, rede, relógio,
ambiente ou processo; consome exclusivamente a porta L1. L4 injeta o adapter L3 no
pipeline normal e o fallback `UnknownCitationFreshness` nos pipelines auxiliares. Os
estados `stale` e `unknown` resultam em V21 Warning explícito e nunca silenciam nem são
promovidos por módulo strict. Não há alteração de parser, configuração ou CLI no delta.

### RED-D1 — confinamento vulnerável a troca concorrente de symlink

O adapter percorre componentes com `symlink_metadata`, mas depois usa `metadata` e
`File::open` novamente pelo path (`03_infra/citation_freshness.rs:55-89`). Entre a
validação e a abertura, outro processo pode substituir um componente ou o arquivo final
por symlink. `File::open` segue o novo vínculo e pode ler fora da raiz. Comparar apenas
tamanho/mtime depois da leitura não restaura confinamento: o alvo externo pode ter os
mesmos valores e os diretórios intermediários nem entram nessa comparação.

Isso contradiz diretamente o L0 da porta (“root symlink ou qualquer componente symlink
é Unknown(Symlink)”) e a alegação de resolução confinada/read-only. A falha é
fail-open quanto à fronteira de leitura, ainda que a classificação V21 posterior seja
fail-closed.

**Fechamento exigido:** resolver por handles confinados, sem seguir symlinks durante a
abertura (por exemplo, travessia `openat`/equivalente com `NOFOLLOW`, verificando cada
componente e vinculando leitura ao handle validado), ou reduzir explicitamente o L0 se
essa garantia não for portável. A correção deve nascer em L0 antes do delta de produção.

### GATE-DEFECT-D1 — B2 não exerce concorrência/confinamento atômico

O gate cobre symlinks já existentes, mas não substituição concorrente entre validação e
abertura, nem verifica identidade do objeto lido. Portanto passa apesar do RED-D1.

### GATE-DEFECT-D2 — matriz B2 incompleta para erros externos

O Assessment exige erro de leitura/metadata e `ConcurrentMutation`. O gate materializado
não provoca erro real de permissão/leitura, remoção ou troca durante resolução, nem
observa `Unknown(ConcurrentMutation)`. Esses casos precisam de testes controlados e
independentes da implementação.

## Evidência executada

- gates B1/B2: 9/9 e 7/7, PASS;
- `cargo test --workspace --quiet`: 628 unitários e toda a matriz de integração PASS,
  incluindo regressão V22;
- busca mecânica de I/O nos arquivos L1 V21/porta: zero ocorrências;
- hashes dos sete insumos do Assessment: PASS;
- `git diff --check cc0691d..505c312`: PASS;
- delta: apenas L0/assessment, porta L1, classificador V21, adapter L3, injeção L4 e
  gates/fixtures relacionados; a reordenação em `03_infra/mod.rs` é mecânica.

## Residuais

- Direção/associação do operador e janela de citações continuam atribuídas ao parser L3
  e permanecem fora do P0088 conforme L0 saneado.
- O limite fixo de 4 MiB é injetado em L4 conforme a API publicada; tornar esse orçamento
  configurável é melhoria posterior, não RED deste lote.
- O warning legado de `print_tree` em `ts_parser.rs` permanece alheio ao delta.

Não recomendar merge. Fechar RED-D1 e os dois GATE-DEFECTs, repetir B2 e então solicitar
novo adversário final.

## Fase D reaberta — fechamento pós-correção

**Sequência confrontada:** `310d14e` L0 → `9fb185b` gates → `d286cd9` produção

**Veredito final:** `READY WITH RESIDUAL AUDIT`

### Fechamento dos achados

- **RED-D1: FECHADO.** Em Linux, `open_confined` abre a raiz e percorre cada componente
  relativamente ao handle de diretório anterior, com `O_NOFOLLOW`; o arquivo final é
  lido e revalidado pelo mesmo handle. A troca posterior do nome no filesystem não muda
  o objeto já aberto e a corrida não pode redirecionar a leitura para o marcador externo.
- **GATE-DEFECT-D1: FECHADO.** B2 agora executa 20.000 resoluções enquanto um diretório
  interno é alternado com symlink para uma raiz externa. O único conteúdo interno é
  vazio, portanto qualquer `Valid` denunciaria escape; nenhuma tentativa foi válida.
- **GATE-DEFECT-D2: FECHADO.** B2 também alterna remoção e recriação do alvo vazio por
  10.000 resoluções e exige fail-closed. Junto aos casos existentes de alvo-diretório,
  raiz inválida, symlink, UTF-8 e orçamento, cobre erros de abertura/metadata e mutação
  sem exigir uma razão única onde o L0 permite `Symlink`, `Io` ou
  `ConcurrentMutation`.

A correção respeita causalidade Tekt: a garantia atômica foi publicada e resselada no
L0 antes do gate e da produção. Nenhuma lista, predicado, severidade ou mensagem V21 foi
movida para L3; L1 continua sem I/O; L4 e V22 não mudaram nesta sequência.

### Evidência da reabertura

- sete hashes do Assessment 0017: PASS, incluindo porta
  `459a7c29e8ad2f842ba4ef2d6cece4d50b5fd76a6da72067f212d99f8c155402`;
- corridas concorrentes B2 repetidas três vezes: 6/6 PASS;
- gate B2 completo: 9/9 PASS;
- gate B1: 9/9 PASS;
- `cargo test --workspace --quiet`: 628 unitários, 83 fixtures e toda a integração PASS;
- regressão V22 incluída na suíte: PASS;
- busca mecânica de I/O em V21 e na porta L1: zero ocorrências;
- `git diff --check 505c312..d286cd9`: PASS;
- delta de fechamento restrito ao L0/assessment, porta L1, adapter L3, gate B2 e este
  relatório; nenhum parser, config, CLI ou wiring foi alterado.

### Residuais finais

- O caminho atômico usa `/proc/self/fd` e flags Linux. Em plataformas não Linux, o
  adapter retorna deliberadamente `Unknown(Io)`, preservando fail-closed, mas a
  disponibilidade funcional e a portabilidade do gate B2 precisam de lote próprio.
- Detecção de conteúdo alterado no mesmo handle continua baseada em tamanho/mtime; isso
  pode perder mutação adversarial com metadados preservados, porém não reabre o escape de
  confinamento: a leitura permanece no handle validado e resultado suspeito ainda não
  concede acesso fora da raiz.
- Permanecem os residuais anteriores de parser/janela, orçamento L4 configurável e o
  warning legado alheio em `ts_parser.rs`.

Não reabrir P0088 pelos residuais acima. O lote pode seguir para relatório de fechamento
e revisão de merge separada.
