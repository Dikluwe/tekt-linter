# Assessment P0116 — implementação conservadora de V27

> **Veredito:** LAB CLOSED — V27 DETECTOR ONLY
> **Regra:** V27 `MergeableDecisionArms`
> **Disponibilidade:** catálogo text/SARIF e `--checks v27`; opt-in
> **Nível:** `Info`
> **Autofix:** ausente
> **V28:** `VACANT`

## 1. Regime e atestação

P0116 foi executado sob protocolo completo como ensaio de materialização segregada por
ordem e artefatos. O executor teve acesso integral ao repositório, portanto o resultado
é **executado sem atestação de isolamento**. Não se alega independência forte entre autor
do contrato, implementador, adversário e verificador.

Entradas congeladas antes da implementação:

| Entrada | Identidade |
|---|---|
| baseline tekt-linter | `07ac97813892a297cad2b477222ccc2d07a59bc0` |
| P0116 inicial | SHA-256 `f59ed1b2eb3c4171c60b6dd696a64d7b7b2427098b9d5bca95150e30ce456f71` |
| ADR-0020 | SHA-256 `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| política segregada | SHA-256 `4069959f9d1c77d3b582527693b1255a378d72b0f35bbed9d96feff129e15cfb` |

Política de `Unknown`: macro, placeholder, `cfg`, corpo vazio, range, catchall, IR ausente,
braço não adjacente ou qualquer binding não produz achado acionável.

## 2. Materialização

### L1

- `DecisionArm` recebeu `DecisionArmMergeability` opcional;
- a evidência contém árvore canônica do corpo e guard, bindings, macro, `cfg` e placeholder;
- V27 agrupa somente braços consecutivos e emite um achado por grupo maximal;
- fingerprint isolado não é autoridade: a árvore canônica integral é comparada;
- tokens anônimos, inclusive operadores, permanecem na serialização;
- `None` é `Unknown`, nunca equivalência implícita.

### L3

O parser Rust extrai estrutura integral visitando nós nomeados e não nomeados, remove
somente comentários/whitespace ausentes da AST, canonicaliza referências a bindings e
preserva guard. Macros e atributos condicionais são marcados antes de L1.

### L2/L4

- catálogo SARIF contém V0–V27 uma vez cada;
- `--checks v27` ativa somente V27;
- `--checks all` inclui V27;
- a lista padrão continua V0–V26: o detector permanece opt-in durante o laboratório;
- V27 não altera exit code por ser `Info`;
- nenhum caminho de escrita de source ou autofix foi criado.

## 3. Defeito encontrado pelo piloto e refinamento

A primeira execução sintática produziu 208 grupos em 57 arquivos. Ela revelou que nome e
modo iguais não provam compatibilidade de tipo:

```rust
(Value::Int(a), Value::Int(b)) => a.cmp(b),
(Value::Float(a), Value::Float(b)) => a.cmp(b),
```

Um `or-pattern` exige que `a` e `b` tenham tipos compatíveis entre alternativas. A IR de
tree-sitter não possui autoridade de tipos. Converter esse caso em V27 seguro seria
confundir `Unknown` com prova.

O gate foi refinado para bloquear qualquer binding. Um segundo ataque encontrou bindings
de shorthand estrutural (`FrameItem::Text { pos, .. }`) que o primeiro extrator não
classificava. O parser passou a reconhecer `shorthand_field_identifier`; uma fixture
impede regressão. Depois dos dois refinamentos, o piloto caiu para 59 grupos em 28
arquivos.

Essa redução é evidência de fail-closed, não perda acidental de sinal. Suporte a bindings
exige HIR/type authority ou contrato equivalente em passo posterior.

## 4. Oráculos e ataques

A suíte cobre os doze ataques normativos:

| Mutação/ataque | Testemunha que rejeita |
|---|---|
| apagar operadores anônimos | `==` versus `!=` |
| aceitar apenas hash/fingerprint raso | argumento e ordem de filhos diferentes |
| ignorar guard | guards diferentes e guard ausente |
| ignorar modo de binding | `move` versus `ref` |
| ignorar usos/posição de binding | argumentos permutados |
| ignorar adjacência | braço intermediário bloqueia grupo |
| ignorar `cfg` | atributo condicional bloqueado |
| aceitar macro textual | duas invocações `emit!()` bloqueadas |
| aceitar placeholder | dois `todo!()` bloqueados |
| emitir pares sobrepostos | três braços produzem um grupo maximal |
| usar ordem não determinística | duas execuções retornam bytes equivalentes |
| perder or-pattern preexistente | sugestão preserva `A | B | C` |

Também há ataques para comentários/rustfmt, `return`, corpo vazio, range, catchall,
Unicode, binding de tupla e shorthand de struct.

Os doze ataques foram rejeitados pela suíte, mas não houve executor independente nem
ferramenta que aplicasse doze patches mutantes um a um. Portanto o assessment registra
**gate adversarial 12/12**, não mutation score atestado.

## 5. Piloto Typst Crystalline

| Observável | Resultado |
|---|---:|
| revisão observada | `dc47c9c32b8b6769a58622c98b885cb094337508` |
| arquivos Rust em `01_core`–`04_wiring` | 450 |
| V19 | 349 |
| V20 | 600 |
| V27 antes do bloqueio integral de bindings | 208 grupos / 57 arquivos |
| V27 conservadora final | 59 grupos / 28 arquivos |
| escrita no consumidor | zero |

O corpus mudou desde P0115, cuja revisão era `7fb5bb6...`; os números deste assessment
pertencem somente à revisão acima. O piloto executou o binário real com
`--checks v27 --format sarif` e não aplicou sugestões.

Os achados finais incluem braços sem binding como aliases literais, variantes unitárias,
wildcards internos e tabelas declarativas. A transformação pode continuar inadequada por
intenção ou sobreposição, apesar da igualdade do corpo. Como a revisão humana integral
dos 59 grupos e a compilação de patches temporários não foram materializadas, eles não
recebem classificação `MachineApplicable`.

## 6. Gates executados

- baseline anterior: 635 testes unitários e suíte integral verdes;
- após implementação: 640 testes unitários e toda a suíte `--all-targets` verde;
- assessment V27 parser+regra: 4/4;
- catálogo SARIF: 28 regras únicas, V0–V27;
- V27 no catálogo: exatamente uma;
- piloto V19/V20: 349/600, sem regressão de contagem;
- piloto V27 final: 59/28;
- `cargo fmt --all`: PASS;
- ciclo real `--fix-hashes`: zero drift após atualização;
- consumidor: somente leitura.

## 7. Limitações e decisão

V27 está implementada como detector conservador opt-in, mas não satisfaz os critérios de
promoção integral de P0116 porque:

1. não há autoridade de tipos para bindings;
2. os 59 grupos finais não tiveram revisão manual de 100%;
3. patches de laboratório não foram compilados em cópia temporária;
4. o gate adversarial não teve execução mutante independente;
5. o executor único impede atestação de isolamento.

Por isso, o único veredito proporcional é:

```text
LAB CLOSED — V27 DETECTOR ONLY
```

V27 pode ser pesquisada e medida via `--checks v27`, sem afetar a execução padrão. Um
passo posterior pode promover casos sem binding após revisão do corpus ou integrar uma
autoridade HIR/type-aware para recuperar bindings com segurança. V28 permanece vaga.
