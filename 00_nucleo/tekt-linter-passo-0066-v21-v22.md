# Passo 0066 (tekt-linter) — V21 reescrita + V22 nova: caçar vs vigiar

**Repositório**: `tekt-linter` (não `typst-crystalline` — esta é uma mudança na
ferramenta, aplicada depois a todos os repos que a usam, incluindo `typst-crystalline`).
**Precede este passo**: Passo 0063 (implementação V16-V20), Passo 0065 (reconciliação
ADR-0016), e o achado do `typst-crystalline` (P1053/P1054) que motivou esta revisão —
V21 original (info, "todo literal sem comentário") produziu 5921 ocorrências, ruído
inutilizável.
**Pré-condição**: `git status` limpo no `tekt-linter`.

---

## Princípio reitor: duas ferramentas, não uma regra alargada

V21 (estreita) e V22 (larga) não são a mesma regra com sensibilidade diferente — são
instrumentos com propósitos distintos. V21 caça casos accionáveis, um a um. V22 vigia
tendência agregada, por módulo, sem produzir ruído por ocorrência. Não fundir as duas.

---

## Parte 1 — V21 reescrita: `HardcodedContextualValue`

### Predicado (a generalização correcta, não a instância tipográfica)

**Literal numérico, operando de `*`/`/`, com uma variável/campo de "fonte contextual" —
E o resultado alimenta um "sumidouro geométrico".**

A classe do erro é *"escalar contextual tratado como fixo"*, não *"métrica de fonte
tratada como fixo"* — os quatro casos confirmados (`gap`, `Ascent`, `StemV`, `raw.rs
0.9×`) são a instância tipográfica dessa classe, não a classe inteira.

### Configuração (`[v21.context_vars]`, ajustável por repo consumidor)

Fonte contextual — nome de variável/campo contém, case-insensitive:
```
size, style, em, font_*, weight, ascent, descent,
width, height, depth, frame, region, page, margin, padding, container
```

Sumidouro geométrico — atribuição/campo/argumento com nome geométrico, **ou** construtor
de medida:
```
gap, inset, offset, pos, x, y, width, height, thickness, ascent, descent, ...
Length::*, Pt::*, Em::*, Ratio::*
```

(Listas de exemplo — cada repo consumidor pode estender via `[v21.context_vars]` no seu
próprio `crystalline.toml`, sem editar o `tekt-linter`.)

### Exclusões, formalizadas (não implícitas)

1. **Sintaxe fixa de formato** — módulos que escrevem formato externo (operadores PDF,
   tokens de stream, tags SVG) declaram-se em `[v21.format_syntax_modules]`. Dentro
   desses módulos, literais não são avaliados individualmente pela V21 — a citação
   correcta é uma só, no cabeçalho do módulo (referência à especificação do formato), não
   repetida por ocorrência.
2. **Tabelas de dados** — `match` com ≥5 braços, todos da forma `literal => literal`
   (proxy heurístico para "tabela de tradução", não decisão geométrica) — excluído do
   predicado.
3. **Testes/fixtures** — todo ficheiro `tests.rs`/`fixture`/`integration_tests.rs`, já
   fora de âmbito desde a V21 original, mantido.

### Nível e ratchet

`warning` — accionável, worklist pequena esperada (dezenas, não milhares, per os 4 casos
já confirmados como base de calibração). Ratchetável a `error` depois de zerada, mesmo
padrão de V16.

---

## Parte 2 — V22 nova: `ProvenanceInventory`

**Não é a V21 sem o predicado estreito** — é uma ferramenta diferente: métrica agregada
por módulo, não aviso por ocorrência.

### O que mede

Para cada módulo: `(literais com proveniência citada) / (total de literais no módulo)`,
excluindo as mesmas categorias da Parte 1 (formato fixo, tabelas de dados, testes).

### Nível e output

`info`, opt-in (não corre por defeito em `crystalline-lint .` sem flag explícita — é
ferramenta de vigilância, não de gate). Output: uma linha por módulo com o rácio, não uma
linha por literal.

### O sinal que dá, que a V21 sozinha não dá

Se um módulo ganha, num só passo, dezenas de literais novos sem proveniência — mesmo que
nenhum bata o predicado estreito da V21 (não é multiplicação por variável contextual,
por exemplo) — o **delta** no rácio agregado é visível. É a mesma lógica de
"co-mudança sem mecanismo plausível" já adoptada no método de fatiamento (P1023): um
salto no agregado é motivo para olhar, mesmo sem saber ainda o quê.

---

## Parte 3 — Mecanismo de crescimento do predicado (memória institucional)

**A garantia que a V21 dá não é "zero placeholders" — é "nenhum placeholder de classe já
nomeada volta a entrar sem rasto".** Isto só se mantém verdade se o predicado crescer
quando uma classe nova for encontrada.

Regra de processo, a registar no `README`/ADR do `tekt-linter`:

> Quando um placeholder de "fechar buraco" for encontrado que a V21 **não** apanhou
> (falso negativo confirmado), o passo de correcção do bug **tem de** incluir uma segunda
> entrega: a extensão do predicado da V21 (nova entrada em `context_vars`/sumidouros, ou
> nova heurística) que o teria apanhado. Corrigir o bug sem estender a regra é dívida
> aberta, não passo fechado.

Isto espelha como o `clippy` cresce — cada lint novo é a memória de um modo de falha que
alguém nomeou uma vez.

---

## Fase de validação

```
cargo test --workspace   # suite do tekt-linter
crystalline-lint --checks v21 <repo-alvo>   # confirmar contagem estreita, não 5921
crystalline-lint --checks v22 <repo-alvo>   # confirmar output agregado por módulo
```

Depois de mesclado no `tekt-linter`: passo próprio no `typst-crystalline` para correr a
V21 reescrita e tratar a worklist real (dimensionamento a decidir consoante a contagem
que sair — mesmo padrão do V16-V20).

---

## Resultado esperado

`tekt-linter` com V21 accionável (predicado de escalar-contextual→sumidouro-geométrico,
não só instância tipográfica) e V22 como vigilância agregada, sem sobreposição de
propósito. Processo formal para o predicado crescer a cada classe nova descoberta,
registado, não deixado à memória de quem executou o passo.
