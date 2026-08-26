# Passo operacional 0114 — experimentar contrapesos de V19/V20 no lab

> **Estado:** EXECUTADO — fechado por `lab/assessment_inverse_decision_metrics.md`
> **Escopo:** somente `lab/`, fixtures e assessment; nenhuma integração no registro do
> linter
> **Candidatos históricos avaliados:** V27 `MergeableDecisionArms` e a hipótese então
> chamada V28 `FragmentedPatternProjection`
> **Resultado vigente:** V27 seguiu para P0115; a hipótese V28 foi posteriormente
> descartada por erro conceitual e o ID V28 voltou a ficar vago

> **Nota de vigência (2026-08-26):** as seções V28 abaixo preservam o histórico do
> experimento, não uma candidata ativa. V20 é métrica unidirecional de teto: maior
> profundidade é custo, enquanto especificidade e segurança de tipos são propriedades
> distintas. Não existe contrapeso de complexidade mínima. O probe e as fixtures V28
> foram removidos; `V28 = VACANT`.

## 1. Pergunta fechada

V19 observa alternativas já condensadas em um `or-pattern`; V20 observa profundidade já
condensada em um padrão. Este passo testa as perguntas simétricas:

1. existem braços separados cuja equivalência observável permite representá-los como um
   único `or-pattern`?
2. existe uma projeção estrutural única espalhada por testes aninhados que pode ser
   representada por um único padrão composto?

Os nomes V27/V28 são provisórios e servem apenas para tornar os resultados distinguíveis
no lab. O passo não reserva IDs, não adiciona configuração, não altera README, SARIF,
CLI, parsers, `RuleRegistry`, `HasDecisionArms` ou contagens do auto-lint.

O resultado procurado não é reduzir aritmeticamente V19/V20. As quatro observações podem
coexistir. V19/V20 medem complexidade condensada; os candidatos medem fragmentação
potencialmente dispensável.

## 2. Hipóteses falsificáveis

### H27 — braços fundíveis

Dois ou mais braços de uma mesma decisão são candidatos a fusão quando possuem:

- corpo estruturalmente equivalente após normalização definida e auditável;
- mesmo resultado, efeitos sintaticamente visíveis e forma de controle;
- guards ausentes ou estruturalmente idênticos;
- bindings compatíveis e usados nas mesmas posições;
- nenhuma relação de precedência/sombreamento que mude ao fundir;
- padrões representáveis por `p1 | p2` na linguagem analisada.

H27 é refutada se a análise marcar como fundível um par cuja fusão muda compilação,
ownership/borrow, binding, guard, ordem, efeitos, cobertura ou diagnóstico observável.

### H28 — projeção fragmentada

Uma cadeia de `match`/`if let`/`let else` é candidata a composição quando:

- forma uma cadeia linear de sucesso, sem ação observável entre seus níveis;
- cada valor intermediário serve apenas para a projeção seguinte;
- falhas intermediárias possuem o mesmo destino observável;
- o binding final e sua utilização são preserváveis;
- existe um padrão composto equivalente aceito pela linguagem;
- a composição não altera lifetime, temporários, moves, borrows ou drop order.

H28 é refutada se a análise compuser cadeias com efeitos intermediários, destinos de
falha distintos, reutilização de intermediários, guards diferentes ou semântica de posse
divergente.

## 3. Relação com V19/V20

| Eixo | Lado já observado | Lado experimental |
|---|---|---|
| largura | V19: alternativas dentro de um braço | V27: braços equivalentes ainda separados |
| profundidade | V20: níveis dentro de um padrão | V28: níveis de uma projeção ainda separados |

Os candidatos não geram crédito, score negativo ou cancelamento. O relatório deve
preservar quatro contagens e suas interseções. Um trecho transformado por V27 poderá
passar a aparecer em V19; um trecho transformado por V28 poderá passar a aparecer em
V20. Isso é resultado esperado, não regressão.

O balanceamento só pode ser interpretado por estados:

| Estado | Significado |
|---|---|
| `COMPACT` | forma condensada observada por V19/V20 |
| `FRAGMENTED-CANDIDATE` | contrapeso encontrou composição plausível |
| `JUSTIFIED-SEPARATION` | separação contém diferença semântica necessária |
| `JUSTIFIED-COMPOSITION` | condensação representa uma única decisão/projeção |
| `UNKNOWN` | evidência estática insuficiente |

## 4. Isolamento arquitetural do experimento

Criar, no máximo:

```text
lab/inverse_decision_metrics_probe.rs.txt
lab/inverse_decision_metrics_fixtures.rs.txt
lab/assessment_inverse_decision_metrics.md
```

Os arquivos `.rs.txt` são artefatos experimentais e não entram no crate. Se for necessário
executar código, o assessment deve materializar uma cópia em diretório temporário ou usar
um crate standalone sob `lab/` explicitamente excluído do workspace. Nenhum módulo de
L1–L4 pode importar o probe.

O experimento pode ler a fonte Rust com parser independente já disponível no ambiente,
mas deve registrar versão e árvore usada. Não pode modificar o parser de produção para
facilitar a hipótese. A insuficiência do IR atual é resultado do experimento, não licença
para enriquecê-lo neste passo.

## 5. A — congelamento do baseline

Antes de implementar o probe, registrar no assessment:

1. commit-base e estado do worktree;
2. SHA-256 de V19, V20, seus prompts e ADR-0016;
3. versão do compilador e dos parsers/oráculos usados;
4. saída atual de V19/V20 sobre cada fixture e sobre o próprio linter;
5. confirmação de que V26 já pertence a `NucleusIntegrity` e de que V27/V28 ainda não
   existem no registro de produção;
6. `cargo test`, `cargo fmt --check`, `fix-hashes --dry-run` e auto-lint atuais.

Mudança de baseline durante o ensaio é `GATE-DRIFT`; repetir A em vez de atualizar
resultados silenciosamente.

## 6. B — matriz mínima de fixtures V27

Cada linha deve conter entrada, transformação candidata, saída compilável esperada e
veredito do probe.

### Positivos obrigatórios

1. dois braços adjacentes com corpo literal idêntico;
2. três braços com a mesma chamada e mesmos argumentos;
3. corpos equivalentes apesar de espaços, comentários e parênteses redundantes;
4. padrões sem binding com corpo de bloco estruturalmente idêntico;
5. braços equivalentes separados por outro braço, com prova de que a ordem é preservável.

### Negativos obrigatórios

1. texto parecido, mas literal, callee ou argumento diferente;
2. mesmo corpo textual com bindings de nomes/origens incompatíveis;
3. um guard ausente, distinto ou com efeito;
4. braço anterior que sombreia um padrão posterior;
5. `return`, `break`, `continue`, `?`, panic ou macro divergente;
6. borrow/move diferente ou binding por `ref`/`mut` incompatível;
7. atributos ou comentários normativos que exijam evidência separada;
8. corpos vazios iguais com intenção desconhecida — classificar `UNKNOWN`, não positivo;
9. igualdade textual produzida por macro sem equivalência demonstrável;
10. linguagens não habilitadas — silêncio.

O oráculo mínimo é compilar antes/depois e comparar testes observáveis. Compilação não
prova equivalência, portanto todo positivo também precisa de inspeção estrutural do diff.

## 7. C — matriz mínima de fixtures V28

### Positivos obrigatórios

1. cadeia pura de dois `if let` com único corpo final;
2. cadeia pura de três níveis `Option`/enum;
3. `match` intermediário cujo único caminho produtivo continua a projeção e cujas falhas
   convergem para o mesmo destino;
4. binding intermediário usado exclusivamente como scrutinee do nível seguinte;
5. composição que preserve exatamente o binding final.

### Negativos obrigatórios

1. logging, contador, alocação, chamada ou mutação entre níveis;
2. destinos de falha diferentes (`return`, erro, fallback ou mensagem distinta);
3. intermediário usado fora da projeção seguinte;
4. guard em qualquer nível;
5. borrow/move/drop order potencialmente diferente;
6. `await`, `?`, macro ou closure entre níveis;
7. cadeia que atravessa função, módulo ou trait;
8. mais de uma ação terminal;
9. padrão composto não aceito pelo compilador;
10. nesting meramente lexical, sem relação de projeção — silêncio.

Padrões V20 já profundos não são positivos V28 por si só. V28 exige fragmentação entre
expressões; V20 observa profundidade dentro de um padrão.

## 8. D — detectores experimentais e evidência

### D1 — V27 provisório

Construir assinatura canônica do corpo sem usar apenas texto bruto. A assinatura deve
preservar, no mínimo:

```text
node kinds | paths/callees | literals | control flow | macro identity |
binding-use positions | guard | effect markers
```

Whitespace, comentários não normativos e parênteses transparentes podem ser removidos.
Identificadores não podem ser renomeados livremente: uma eventual equivalência alfa deve
mapear cada binding de padrão à mesma posição de uso e rejeitar captura ou origem
incompatível.

Saída experimental por grupo:

```text
candidate | path | decision_span | arm_spans | patterns | body_fingerprint |
binding_map | blockers | confidence | verdict
```

### D2 — V28 provisório

Modelar a cadeia como grafo linear de projeções. Cada aresta registra scrutinee, padrão,
binding produzido, usos, caminho de falha e efeitos entre níveis. Emitir candidato somente
quando houver uma única composição sintática demonstrável.

Saída experimental por cadeia:

```text
candidate | path | outer_span | depth | projection_chain | final_binding |
failure_equivalence | effect_barriers | ownership_risk | confidence | verdict
```

`confidence` pode ser `PROVEN-SYNTACTIC`, `PLAUSIBLE` ou `UNKNOWN`. Somente
`PROVEN-SYNTACTIC` entra na contagem principal; os demais permanecem visíveis.

## 9. E — confronto por transformação

Para cada positivo alegado:

1. gerar ou escrever manualmente a transformação mínima em cópia temporária;
2. exigir parsing e compilação antes/depois;
3. executar testes dirigidos e comparar stdout, stderr e exit status quando aplicável;
4. comparar V19/V20 antes/depois;
5. registrar mudança de linhas, braços e profundidade;
6. tentar refutar equivalência com pelo menos uma mutação em padrão, guard, binding e
   caminho de falha;
7. reverter a cópia temporária; produção permanece intocada.

O objetivo é confirmar a relação esperada:

```text
V27 aplicado -> braços diminuem; alternativas V19 podem aumentar
V28 aplicado -> etapas diminuem; profundidade V20 pode aumentar
```

Se o candidato só puder ser validado por conhecimento de tipos ou efeitos indisponível,
classificá-lo `UNKNOWN`. Não inferir benefício a partir da redução de linhas.

## 10. F — piloto em código real

Depois de todas as fixtures passarem, executar o probe em dois universos separados:

1. o próprio `tekt-linter`;
2. um único consumidor Rust escolhido e congelado no assessment.

Amostra obrigatória:

- revisão manual de 100% dos candidatos se houver até 50;
- se houver mais de 50, revisar todos os `PROVEN-SYNTACTIC`, todos os `UNKNOWN` e amostra
  determinística suficiente para cobrir cada cluster restante;
- registrar falsos positivos, falsos negativos encontrados pela busca manual e casos sem
  decisão possível.

Nenhuma transformação é aplicada aos repositórios. O piloto produz somente inventário e
patches ilustrativos no assessment.

## 11. Critérios de promoção ou rejeição

### `PROMOTE-SPEC`

Um candidato pode receber passo posterior de especificação quando:

- zero falso positivo na matriz adversarial;
- zero mudança comportamental nas transformações aceitas;
- precisão manual de 100% em `PROVEN-SYNTACTIC` no piloto;
- pelo menos três ocorrências reais não triviais em dois módulos ou projetos;
- identidade e localização estáveis sob formatação;
- custo e dados necessários ao parser explicitamente medidos;
- mensagem explica benefício e bloqueadores sem prometer equivalência semântica geral.

Promoção significa escrever ADR/prompt e decidir IDs; não significa integrar diretamente.

### `KEEP-LAB`

Manter experimental quando houver sinal útil, mas depender de tipos, efeitos, CFG ou
ownership não disponíveis no IR, ou quando a precisão útil exigir revisão humana.

### `REJECT`

Rejeitar quando a redução de fragmentação não for decidível por proxies estáveis, houver
falsos positivos de alto impacto, o sinal real for raro, ou a forma sugerida piorar
legibilidade sem critério observável.

V27 e V28 são avaliados separadamente. Um pode ser promovido e o outro rejeitado.

## 12. Assessment de fechamento

`lab/assessment_inverse_decision_metrics.md` deve conter:

1. hashes e baseline;
2. definição executável das normalizações;
3. matriz completa de fixtures e resultados;
4. inventário do piloto com classificação individual;
5. patches antes/depois dos verdadeiros positivos;
6. falsos positivos, falsos negativos e `UNKNOWN`;
7. custo de parser/IR e tempo de execução;
8. efeito observado nas contagens V19/V20;
9. decisão independente para V27 e V28;
10. riscos e pergunta residual que bloquearia produção.

Estado terminal do passo:

- `LAB CLOSED — PROMOTE V27/V28 SPEC`;
- `LAB CLOSED — PROMOTE ONE SPEC`;
- `LAB CLOSED — KEEP EXPERIMENTAL`;
- `LAB CLOSED — REJECT CANDIDATES`;
- `BLOCKED` por `SPEC-GAP` ou oráculo insuficiente.

## 13. Fora de escopo

- adicionar V27/V28 ao linter ou escolher definitivamente esses IDs;
- alterar V19/V20, seus limiares, mensagens, severidades ou baseline;
- fazer V27 cancelar V19 ou V28 cancelar V20;
- produzir score agregado de qualidade;
- autofix em código real;
- mudar `HasDecisionArms`, parsers ou schema de configuração;
- usar histórico Git/co-change, análise interprocedural ou resolução completa de tipos;
- modificar Tekt, Bateia ou tekt-cargo-dsm;
- merge, push, release ou instalação global.

## 14. Commits previstos

1. `lab(p0114): freeze inverse decision metrics hypotheses`
2. `lab(p0114): add adversarial inverse metrics fixtures`
3. `lab(p0114): implement isolated inverse metrics probes`
4. `lab(p0114): confront candidates by source transformation`
5. `docs(p0114): close inverse decision metrics experiment`

Se o probe não precisar ser versionado para reproduzir a conclusão, os commits 2–4 podem
ser condensados, mas o assessment deve continuar reproduzível a partir do commit-base.
