# Assessment 0027/C — backlog finito pós-P0097

**Papel:** C, reconciliação dos pareceres congelados A/B1/B2  
**Resultado:** `RESSELADO APÓS RED D1`
**Produção, testes e outros documentos lidos:** não  
**Universo:** somente S1–S6  

## Identidade dos insumos segregados

| Parecer | SHA-256 esperado | SHA-256 recalculado | Resultado |
|---|---|---|---|
| A — cobertura pós-P0097 | `c9c9d6d73573b5f37146faa36ff54790108d91e63bcd560ea1f81ef414ed477f` | `c9c9d6d73573b5f37146faa36ff54790108d91e63bcd560ea1f81ef414ed477f` | `PASS` |
| B1 — delta estrutural | `fac7a67068e6f63a969f3725710026afed3f828275859e8f49cafb6a1ec914e2` | `fac7a67068e6f63a969f3725710026afed3f828275859e8f49cafb6a1ec914e2` | `PASS` |
| B2 — autoridade e promessas | `3f4fee9273c72ca0202e9f2ab95e551f7f53e55c7862493ba34660a148f94e3e` | `3f4fee9273c72ca0202e9f2ab95e551f7f53e55c7862493ba34660a148f94e3e` | `PASS` |

O Assessment 0027 foi usado somente como protocolo de entrada. Os três pareceres
concordam que o único delta pós-P0096 foi o recorte numérico Rust de P0097, que S3 não
recebeu causa de reabertura e que nenhuma seam nova surgiu. As aparentes discordâncias
são de nível, não de fato: A preserva os estados históricos, B1 mede estrutura e B2 mede
autoridade. C aplica a taxonomia final abaixo sem promover ausência de delta a
fechamento e sem promover `SPEC-GAP` a comportamento presumido.

## Destino único de S1–S6

| Seam | Destino | Reconciliação |
|---|---|---|
| S1 — extractor/escritor de snapshot | `L0-BLOCKED` | O comando e seu núcleo estreito são promessa, mas SG-27-B2-01/02 impedem gate integral do writer e do schema. O fechamento do loader não se propaga. |
| S2 — refinamento Git/subprocesso | `L0-BLOCKED` | SG-27-B2-03/06 contradizem vigência, autorização e superfície pública. Nenhum teste com Git real substitui essa decisão. |
| S3 — manifesto, recibo e selo | `CLOSED` | Fechamento específico permanece válido e não há causa concreta para `REOPENED`. Resíduos expressamente adiados ficam aceitos abaixo. |
| S4 — pipeline principal | `MANDATORY` | O caminho público do lint e as saídas observáveis são promessa suficiente. A unidade auditável é comando/caso, nunca o arquivo ou “pipeline inteiro”. |
| S5 — parsers concretos para IR comum | `MANDATORY` | Suporte a nove linguagens é promessa. O adversário D1 reabriu a célula Rust/número negativo em macro; o restante de P0097 conserva somente o fechamento efetivamente provado. A matriz continua finita, com saneamento L0 antes das parcelas deficientes. |
| S6 — preflight/precedência CLI ampliada | `L0-BLOCKED` | SG-27-B2-05 impede um gate global e a resolução de S2 afeta a matriz de comandos. Casos estreitos existentes não fecham a política global. |

Não há seam inteira adicional `REOPENED`, e S3 permanece fechada. Porém D1 encontrou um
`GATE-DEFECT` concreto na sub-seam S5/Rust/números: o gate positivo excluía macro, mas o
caso negativo `emit!(-5)` não foi confrontado e a produção não aplica a mesma supressão
ao ramo `unary_expression`. Essa célula deixa de ser `CLOSED` e passa a `MANDATORY` dentro
de F12. Também não há seam inteira
`ACCEPTED-RESIDUAL`, pois S1, S2, S4, S5 e S6 contêm promessas públicas ainda não
confrontadas.

## Backlog bloqueador fechado em 13 lotes

Os 13 lotes abaixo são o conjunto máximo da campanha atual. Um lote pode ser reduzido
ou encerrado como não aplicável pela decisão L0 de que depende, mas nenhum lote novo é
criado sem gatilho formal de reabertura. Cada gate funcional deve usar expectativa
derivada do L0 hash-pinned, preservar RED causal, corrigir `GATE-DEFECT` antes do
veredito e respeitar L1 decisão, L2 apresentação, L3 efeito/adaptação e L4 composição.

| ID | Seam | Estado inicial | Risco / confiança | Escopo finito | Dependências | Critério de aceite |
|---|---|---|---|---|---|---|
| F01 | S1 | `L0-BLOCKED` | médio / alta | Sanear SG-27-B2-01: destino existente, criação de diretório, modo/permissão, durabilidade e recuperação de temporário. | nenhuma | Uma única autoridade vigente decide todos os pontos, recebe hash e elimina contradições; sem produção. |
| F02 | S1 | `L0-BLOCKED` | médio / média | Sanear SG-27-B2-02: duplicatas, campos extras, budgets de query/capturas e identidade de observável. | nenhuma | Schema fechado, exemplos positivos/negativos e limites determinísticos hash-pinned; sem produção. |
| F03 | S1 | `MANDATORY` | alto / média | Confrontar fonte/contrato Rust → observáveis → snapshot v1 até serialização e publicação atômica no adapter L3, somente no envelope decidido. Não cobre composição nem exit L4. | F01, F02 | Gates independentes cobrem cardinalidade/ausência, confinamento, determinismo, classes de falha e efeito L3; decisões L1 e regressões do componente permanecem verdes. |
| F04 | S2 | `L0-BLOCKED` | alto / alta | Resolver SG-27-B2-03/06: vigência, autorização e promessa pública de `refine-revisions`. | nenhuma | Prompt, ADR e documentação pública têm uma decisão coerente, hash-pinned, que confirma ou revoga o comando e define o envelope Git local. |
| F05 | S2 | `MANDATORY` | crítico / alta | Se F04 confirmar: confrontar OID, framing, symlink/submódulo, budgets, timeout/kill e falha inconclusiva sem shell/rede/mutação; se revogar: remover a promessa/rota vigente e provar ausência pública. | F04 | Gate hostil independente/injetável, sem Git real como único oráculo, passa no envelope confirmado; ou a revogação é confrontada ponta a ponta. |
| F06 | S4 | `MANDATORY` | crítico / alta | Comando `lint`: leitura/parse fail-closed, seleção de parser, Map-Reduce, regras locais/globais, ordenação, apresentação e exit. | F09, F10 e parcelas pertinentes de F12/F13 | Gates por caso observável confrontam entrada→IR→decisão→diagnóstico/exit sem mover decisões entre camadas. |
| F07 | S4 | `MANDATORY` | crítico / média | Comandos mutadores documentados `fix-hashes` e `update-snapshot`: dry-run, escrita, rerun, saída e exit. | F03 quando `update-snapshot` consumir esse contrato; F09 | Gates separam planejamento L2, efeitos L3 e coordenação L4, incluindo falha antes/depois de mutação e resultado determinístico. |
| F08 | S4 | `MANDATORY` | crítico / média | Composição e exit L4 da família refinement: `refine`, `snapshot`, `seal-refinement` e disposição decidida de `refine-revisions`, consumindo a evidência L3 de F03 sem repeti-la. | F03, F04, S3 `CLOSED`, F09; F05 somente se F04 confirmar `refine-revisions` | Cada comando publicado possui ao menos um fluxo positivo, um erro de entrada/efeito e exit confrontado; nenhum comando não vigente é presumido. |
| F09 | S6 | `L0-BLOCKED` | alto / alta | Sanear SG-27-B2-05 em matriz única: parse, config, preflight, reparadores, análise, formatter, quiet, `fail-on`, `emit-resolution` e exits por comando. | F04 | Matriz normativa completa para os comandos vigentes resolve exit 1 versus 2, combinações inválidas e ordem dos efeitos; hash-pinned, sem produção. |
| F10 | S6 | `MANDATORY` | alto / média | Materializar a matriz de F09 em gates de precedência por classes equivalentes, não por combinação cartesiana. | F09 | Cada linha/classe da matriz tem representante positivo e negativo; V0/V8/V10, `--checks`, dry-run, quiet, formato e exit são observados na fronteira pública. |
| F11 | S5 | `L0-BLOCKED` | alto / média | Sanear SG-27-B2-04: elevar C/C++/Zig ao contrato comum e criar/decidir contratos concretos para Go/Java/Elixir. | nenhuma | As seis linguagens têm autoridade concreta uniforme sobre fatos prometidos, localização, duplicatas, erro, limites e fatos não suportados; hashes congelados. |
| F12 | S5 | `MANDATORY` | alto / média | Congelar a matriz consumidor→campo→produtor de Rust/TypeScript/Python e confrontar suas células prometidas ainda não fechadas. Inclui obrigatoriamente `NegativeLiteral` dentro de macro Rust; somente as demais células numéricas provadas por P0097 permanecem `CLOSED`. | nenhuma | O inventário nominal e hash-pinned precede os gates; cada célula tem gate independente por classe semântica/linguagem, exclusões e erro tipado; `emit!(-5)` não produz projeção numérica. |
| F13 | S5 | `MANDATORY` | alto / baixa | Congelar, após F11, a matriz consumidor→campo→produtor de C/C++/Zig/Go/Java/Elixir e confrontar somente suas células declaradas. | F11 | Inventário nominal e hash-pinned precede gates; por linguagem, roteamento total, erro tipado e cada classe declarada são confrontados fonte→IR; “não suportado” explícito vale como célula fechada. |

### Contagem e dependências

- Total máximo: **13 lotes**.
- Lotes inicialmente `L0-BLOCKED`: **5** — F01, F02, F04, F09 e F11.
- Lotes funcionais `MANDATORY`: **8** — F03, F05–F08, F10, F12 e F13.
- Seams fechadas sem lote: **1** — S3.
- Sub-seams reabertas: **1** — S5/Rust/`NegativeLiteral` em macro, absorvida por F12.

A ordem parcial mínima é: F01+F02→F03; F04→F05 e F09; F09→F10; F11→F13;
F03+F09→F07; F03+F04+F09→F08, acrescentando F05 somente no ramo confirmatório de F04;
e F09+F10 mais as parcelas pertinentes de F12/F13→F06. F12 pode avançar
independentemente onde o L0 já for suficiente. Essa
ordem permite paralelismo sem confundir autoridade, gate e produção.

## Fechamentos estreitos e resíduos aceitos

Permanecem `CLOSED`:

- S3 no envelope já fechado;
- células S5/Rust/projeção numérica efetivamente provadas por P0097, exceto
  `NegativeLiteral` sob macro, reaberta por D1 e incorporada a F12.

Permanecem `ACCEPTED-RESIDUAL`, sem gerar lote na campanha:

- S3: sandbox atestável, assinatura criptográfica, identidade/serviço remoto,
  orquestração automática, política de conflito e certificado posterior;
- S3: fsync de diretório e preservação explícita de modo, como riscos operacionais já
  nomeados, enquanto não houver gatilho concreto;
- S5/P0097: variantes de macro fora da matriz explícita que não contradigam uma exclusão
  prometida; tokens negativos deixam de ser residual e passam a F12;
- cobertura universal de gramáticas, ambientes e combinações cartesianas;
- novas linguagens, novos formatos e fatos sem consumidor ou promessa vigente;
- `gix`, wrapper arbitrário, SMT, fetch, checkout, LFS, submódulos e persistência
  diagnóstica futura, salvo se F04 os tornar parte explícita do envelope.

## Critério objetivo de encerramento da campanha

A campanha termina quando, e somente quando:

1. F01–F13 estiverem `CLOSED` ou formalmente eliminados por decisão L0 hash-pinned;
2. não restar `SPEC-GAP` sobre comportamento público vigente de S1–S6;
3. cada promessa mantida tiver travessia confrontada da fonte ao IR, decisão,
   diagnóstico/efeito e exit, conforme as camadas que realmente atravessa;
4. todos os gates finais forem independentes, hash-pinned, verdes e acompanhados de RED
   causal ou justificativa verificável de ausência de delta funcional;
5. regressões proporcionais, auto-lint, hashes e diff-check estiverem verdes;
6. todo risco não coberto constar nominalmente como `ACCEPTED-RESIDUAL` com gatilho de
   reabertura.

Depois disso, a auditoria deixa de ser campanha contínua. Trabalho novo só entra por um
dos gatilhos: mudança de L0/promessa pública; mudança de produtor, consumidor ou travessia
de camada; incidente/entrada hostil nova; evidência de `GATE-DEFECT`; nova linguagem,
comando ou formato oficialmente suportado; ou invalidação de hash/evidência de
fechamento. O gatilho cria uma nova campanha versionada, não amplia estes 13 lotes.

## Único candidato P0099

**P0099 — saneamento L0 da vigência Git/refine-revisions (F04).**

É o único candidato porque resolve o bloqueio normativo mais forte, determina se F05
existe no ramo de confirmação ou revogação, desbloqueia a matriz CLI F09 e delimita F08.
Deve ser exclusivamente documental, segregado e hash-pinned; não pode ler a produção
como autoridade nem criar gate funcional antes da decisão. Nenhum segundo candidato é
selecionado.

## Veredito C

O candidato final permanece condicionado ao novo confronto D. O horizonte continua
finito em 13 lotes: o RED de macro negativa foi absorvido por F12, sem criar uma seam ou
lote adicional. S1, S2 e S6 não podem avançar funcionalmente antes do L0; S4 e S5
permanecem obrigatórias em recortes delimitados; S3 continua fechada.

## RED D1 e resselamento

D1 bloqueou a primeira reconciliação porque ela tratava `NegativeLiteral` sob macro como
residual apesar de a produção não aplicar a exclusão de `macro_invocation` ao ramo
`unary_expression`. Também faltavam risco/confiança por lote, F13 na dependência de F06,
fronteira clara entre F03/L3 e F08/L4, condicionalidade F08→F05 e congelamento nominal das
matrizes F12/F13.

Todos os pontos foram corrigidos acima. O RED de produção pertence ao futuro F12; P0098
permanece documental e não altera gate nem parser. Esta versão substitui integralmente a
reconciliação anterior e aguarda D2.
