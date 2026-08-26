# Passo operacional 0115 — refinar taxonomia e precisão do candidato V27

> **Estado:** EXECUTADO — fechado por `lab/assessment_v27_taxonomy.md`
> **Predecessor:** P0114, fechado por `lab/assessment_inverse_decision_metrics.md`
> **Candidato exclusivo:** V27 `MergeableDecisionArms`
> **Resultado:** manter V27 experimental; taxonomia útil, mas somente 2 fortes
> automáticos e um alias real permanece falso negativo deliberado
> **V28:** ID vago; a hipótese de contraponto de V20 foi descartada como erro conceitual
> por decisão humana posterior ao fechamento original

## 1. Pergunta fechada

P0114 demonstrou que corpos estruturalmente iguais em braços separados produzem sinal
real, mas o piloto no Typst Crystalline revelou naturezas diferentes sob a mesma
igualdade:

- aliases genuínos, como `"cyan"` e `"aqua"`;
- decisões de domínio com consequência compartilhada;
- tabelas declarativas cujas entradas preservam proveniência individual;
- equivalências configuradas, vazias, macro-expandidas ou insuficientemente decidíveis.

P0115 responde somente:

> É possível classificar essas ocorrências com proxies sintáticos estáveis, de modo que a
> contagem principal de V27 represente oportunidade de composição e não apenas igualdade
> de corpos?

O passo não implementa regra de produção. O nome V27 continua provisório até o
fechamento. V26 permanece reservado a `NucleusIntegrity`.

## 2. Baseline empírico inicial

### tekt-linter

- commit-base: `3fd957745d1b3dec264d2924c0b35a4804f6e4c2`;
- probe P0114 SHA-256:
  `ffd5277ed690c7894f317dbbc92d3b2969e6bec796b3f316e7f0bfc6ad1786f3`;
- fixtures P0114 SHA-256:
  `00992cfe6c300778c7bea2c458e8a1851cf1bf189f1aacb7690154907c538516`;
- piloto anterior: 5 `PROVEN-SYNTACTIC` e 8 `UNKNOWN`.

### Typst Crystalline

- revisão observada: `7fb5bb6d9ee73298af4fd09bb858cb26e5fdd568`;
- universo de produção: 450 arquivos Rust em `01_core`–`04_wiring`;
- parsing: 450/450;
- V19/V20 vigentes: 349/600;
- V27 bruto: 40 `PROVEN-SYNTACTIC` e 190 `UNKNOWN`, 230 grupos em 71 arquivos;
- concentração: 29/40 fortes em `03_infra/src/fonts.rs`.

O checkout consumidor possuía mudanças documentais no momento da leitura. A execução de
P0115 deve congelar hashes dos arquivos `.rs` realmente medidos; o OID sozinho não é
autoridade suficiente. Nenhum arquivo do consumidor pode ser editado.

O experimento histórico rotulado V28 produziu zero candidato forte. A decisão posterior
não depende dessa ausência: V20 é uma métrica unidirecional de custo, portanto não há
regra inversa de complexidade mínima. O ID V28 permanece vago.

## 3. Taxonomia normativa experimental

Cada grupo bruto recebe exatamente uma classe primária:

| Classe | Definição observável | Contagem principal? |
|---|---|---|
| `ALIAS-EQUIVALENCE` | dois ou mais padrões nominais/literais denotam a mesma entrada pública e compartilham integralmente a consequência | sim |
| `DECISION-EQUIVALENCE` | variantes distintas de uma decisão fechada compartilham consequência sem evidência individual | sim |
| `CONFIGURED-EQUIVALENCE` | braços compartilham a mesma expressão dependente de configuração/estado explícito | separada |
| `DECLARATIVE-TABLE` | braços pertencem a tabela de lookup/mapeamento, especialmente extensa ou portada de fonte externa | não |
| `EVIDENCE-PRESERVING-SEPARATION` | comentário, citação, issue, origem ou rationale pertence a uma entrada específica | não |
| `EMPTY-EQUIVALENCE` | corpos vazios ou neutros coincidem sem benefício positivo demonstrado | não |
| `MACRO-EQUIVALENCE` | igualdade depende de invocação/expansão de macro | não |
| `OVERLAP-OR-SUBSUMPTION` | wildcard, range ou padrão mais amplo sobrepõe outro candidato | não |
| `BINDING-DEPENDENT` | equivalência depende de mapear bindings, moves, refs ou posições de uso | não nesta fase |
| `UNKNOWN` | proxies não permitem uma das classes anteriores com evidência suficiente | não |

As classes não são níveis de severidade. `ALIAS-EQUIVALENCE` e
`DECISION-EQUIVALENCE` significam “candidato forte”, não autorização de autofix.

## 4. Relação com V19

V27 não cancela nem desconta V19. O par mede estados diferentes:

```text
V27 forte  --composição deliberada-->  V19 observado
V19        --separação deliberada-->   ausência de V27, com razão preservada
```

O relatório experimental deve apresentar:

```text
V19 total
V27 bruto
V27 por classe
V27 forte = ALIAS-EQUIVALENCE + DECISION-EQUIVALENCE
V27 não acionável = tabela + evidência + empty + macro + overlap + binding
V27 unknown
```

É proibido produzir `V19 - V27`, score líquido ou alegação de qualidade baseada somente
na contagem.

## 5. A — congelamento reproduzível

Antes de alterar o probe:

1. criar `lab/assessment_v27_taxonomy.md`;
2. congelar commit, status, ferramentas e hashes dos artefatos P0114;
3. gerar manifesto `path | sha256` dos 450 `.rs` do Typst Crystalline;
4. preservar a saída TSV bruta que produziu 40/190;
5. congelar uma linha por grupo com path, span, owner, padrões, fingerprint e classe
   inicial `UNCLASSIFIED`;
6. confirmar V19=349 e V20=600 no mesmo corpus;
7. confirmar que V27/V28 não existem no registro de produção do linter;
8. executar baseline do tekt-linter: fmt, testes, dry-run de hashes e auto-lint.

Qualquer divergência de bytes do corpus é `CORPUS-DRIFT`; não atualizar números antigos
silenciosamente.

## 6. B — autoridade manual segregada

Classificar manualmente 100% dos 40 fortes antes de modificar o detector. Para os 190
`UNKNOWN`, classificar:

- todos os grupos de arquivos que concentrem cinco ou mais ocorrências;
- todos os casos de macro, corpo vazio, wildcard, tupla e binding identificados;
- amostra determinística mínima de 30 casos cobrindo todas as formas residuais.

Materializar:

```text
id | path | line | owner | patterns | body_fingerprint | nearby_evidence |
initial_confidence | human_class | rationale | transformable | reviewer_attack
```

O classificador manual deve ler o contexto local e comentários associados. Igualdade de
AST não decide `DECLARATIVE-TABLE` nem `EVIDENCE-PRESERVING-SEPARATION`.

Um confronto separado tenta reclassificar cada caso forte procurando:

- comentário pertencente a uma variante;
- tabela portada ou comparada com upstream;
- ordenação normativa;
- evolução independente provável e explicitamente documentada;
- sobreposição de padrões;
- diferenças de guard, atributos, binding, move ou controle;
- corpo igual apenas por macro ou estado configurável.

Discordância não resolvida termina em `UNKNOWN`, nunca em forte por maioria.

## 7. C — proxies a testar

O probe refinado pode usar somente proxies explícitos e falsificáveis:

### C1 — tabela declarativa

Sinais cumulativos, não absolutos:

- `match` com muitos braços predominantemente literais;
- owner ou comentário indicando `table`, `lookup`, `mapping`, `port`, `upstream` ou
  inventário;
- repetição de construtores de dados sem fluxo de controle;
- comentários/citações distribuídos entre entradas;
- alta relação braços/corpos distintos.

Nenhum limiar entra por conveniência. Testar fronteiras `N-1/N/N+1` e casos pequenos que
continuam sendo tabelas.

### C2 — evidência individual

Associar comentários contíguos ao braço seguinte sem apagar sua posição. Comentário com
URL, issue, `SAFETY`, prompt/passo, nome próprio da variante ou rationale explícita deve
bloquear classe forte, salvo prova de que se aplica integralmente ao grupo.

Comentários de layout ou puramente repetitivos podem ser neutros, mas essa neutralidade
precisa de critério verificável; não se presume.

### C3 — alias

`ALIAS-EQUIVALENCE` exige evidência nominal adicional à igualdade do corpo, por exemplo:

- comentário que declara alias/sinônimo;
- par conhecido no mesmo domínio e nomeado localmente como alias;
- teste público que exige ambas as grafias para o mesmo resultado;
- agrupamento já usado no mesmo módulo para a mesma identidade.

Sem evidência positiva, literais iguais permanecem `DECISION-EQUIVALENCE` ou `UNKNOWN`.
O detector não mantém dicionário universal de sinônimos.

### C4 — decisão fechada

`DECISION-EQUIVALENCE` requer padrões unitários e disjuntos, ausência de evidência
individual, mesmo guard e fingerprint integral do corpo. Literais em tabela grande não
entram nessa classe apenas por serem disjuntos.

### C5 — configuração, bindings e macros

Manter canais separados. Este passo mede sua frequência e falsos positivos, mas não os
promove para a contagem forte. Expandir macros, resolver tipos ou provar ownership fica
fora de escopo.

## 8. D — matriz adversarial obrigatória

Adicionar fixtures cobrindo, no mínimo:

1. alias declarado e não declarado com o mesmo corpo;
2. enum fechado com duas variantes equivalentes;
3. enum com comentário específico em somente uma variante;
4. tabela pequena e tabela extensa;
5. tabela portada de upstream com comentários por entrada;
6. comentários de layout versus URL/issue/rationale;
7. braços adjacentes e não adjacentes;
8. configuração compartilhada;
9. corpo vazio, macro, `return`, `break`, `continue` e `?`;
10. bindings iguais por nome, iguais por posição e incompatíveis;
11. wildcard/subsunção, range, guard e or-pattern preexistente;
12. permutação de braços e deslocamento por formatação;
13. Unicode e representações textualmente próximas mas distintas;
14. caso real reduzido de `fonts.rs`, preservando seus comentários;
15. casos reduzidos de `parse_color`, `keyword` e `ordering`.

Cada fixture congela classe esperada, cardinalidade, evidência, localização e estabilidade
do fingerprint. O gate deve falhar se uma tabela virar alias somente porque dois corpos
coincidem.

## 9. E — repetição no Typst Crystalline

Depois de fechar a matriz, repetir exatamente o manifesto congelado. O relatório deve
comparar, sem esconder migrações:

```text
classe | baseline manual | detector refinado | TP | FP | FN | UNKNOWN
```

Revisar integralmente toda emissão forte nova. Para `DECLARATIVE-TABLE` e
`EVIDENCE-PRESERVING-SEPARATION`, demonstrar que `fonts.rs` não infla a contagem forte.

Produzir patches ilustrativos somente para pelo menos:

- um alias forte;
- uma decisão fechada forte;
- uma tabela corretamente não acionável;
- um caso `UNKNOWN` que não pode ser transformado com segurança.

Os patches ficam no assessment; não são aplicados ao consumidor.

## 10. Critérios de promoção

V27 recebe `PROMOTE-SPEC` somente se:

- zero falso positivo de alto impacto na matriz adversarial;
- precisão de 100% na contagem forte revisada do corpus congelado;
- todas as 40 ocorrências fortes originais tiverem destino explicado;
- tabelas/evidência não entrarem na contagem forte;
- existirem pelo menos três fortes em dois módulos após a filtragem;
- identidade sobreviver a formatação e deslocamento de linha;
- custo do parser e campos adicionais de IR estiverem medidos;
- mensagem declarar “equivalência estrutural candidata”, não equivalência semântica;
- implementação futura puder nascer como `info` e sem autofix.

Se houver sinal útil sem separação precisa, terminar `KEEP-LAB`. Se a taxonomia depender
materialmente de tipos, efeitos ou conhecimento de domínio não representável, terminar
`SPEC-GAP`. `REJECT` exige evidência própria e não pode ser inferido do destino de V28.

## 11. V28 vaga por decisão posterior

O fechamento original suspendeu V28. A decisão humana posterior substitui essa suspensão:

```text
V28 = VACANT — no inverse-complexity rule
```

A hipótese foi descartada porque:

- métricas de complexidade são tetos, não objetivos mínimos;
- especificidade e segurança de tipos não são aumento desejável de complexidade;
- exigir complexidade mínima cria incentivo perverso para ramificações e nesting
  artificiais.

Permanece proibido:

- reintroduzir `FragmentedPatternProjection` sob V28 ou outro ID como inverso de V20;
- exigir complexidade, nesting ou ramificação mínimos;
- usar ausência de achados como fundamento da decisão conceitual;
- comparar quantitativamente a hipótese descartada com V20;
- tratar especificidade ou segurança de tipos como complexidade desejável.

Uma pesquisa futura sobre outra propriedade pode usar o próximo ID disponível, inclusive
V28, mas não herda a hipótese `FragmentedPatternProjection`.

## 12. Assessment e estados terminais

`lab/assessment_v27_taxonomy.md` deve conter:

1. baseline e manifesto do corpus;
2. autoridade manual e confronto dos 40 fortes;
3. amostra estratificada dos 190 `UNKNOWN`;
4. definição executável de cada proxy;
5. matriz adversarial completa;
6. tabela de confusão por classe;
7. inventário refinado do Typst Crystalline;
8. efeito projetado sobre V19, sem subtração;
9. custo de IR/parser e riscos residuais;
10. veredito de V27 e registro de que V28 está vaga.

Estados permitidos:

- `LAB CLOSED — PROMOTE V27 SPEC`;
- `LAB CLOSED — KEEP V27 EXPERIMENTAL`;
- `LAB CLOSED — REJECT V27`;
- `BLOCKED — V27 SPEC-GAP`.

Em todos eles, V28 permanece `VACANT`.

## 13. Fora de escopo

- implementar ou registrar V27;
- editar V19, V20, V26 ou V28;
- criar autofix;
- aplicar refatorações ao Typst Crystalline;
- usar score líquido ou meta de redução de linhas;
- expandir macros, resolver tipos ou executar análise interprocedural;
- modificar L1–L4, CLI, SARIF, configuração ou README;
- alterar Tekt, Bateia ou tekt-cargo-dsm;
- commit, merge, push, tag, release ou instalação global.

## 14. Commits previstos

1. `lab(p0115): freeze V27 taxonomy corpus`
2. `lab(p0115): classify mergeable decision candidates`
3. `lab(p0115): add adversarial V27 taxonomy fixtures`
4. `lab(p0115): refine isolated V27 probe`
5. `docs(p0115): close V27 taxonomy experiment`

P0115 não escreve passo de V28 automaticamente.
