# ADR-0018 — Contratos declarativos de preservação semântica

**Status:** ACEITO — aprovado pelo humano em 2026-08-23  
**Data:** 2026-08-23  
**Origem operacional:** comando temporário P0070  
**Regras propostas:** V23 `ContextErasure`, V24 `SemanticFieldLoss`, V25 `DecisionOwnership`

## Contexto medido

A remedição do repositório-oráculo `typst-crystalline` ocorreu em
`2026-08-23T11:49:13-03:00`, no commit
`781b207b4a5de9c2bfbe5819918a193d1d9293e5`, com working tree não commitado:
216 arquivos rastreados alterados, 9.251 inserções e 4.385 remoções, além de arquivos
não rastreados. O comando abaixo, executado com o linter deste repositório, produziu 45
V21 e exit code 0:

```sh
cargo run --quiet -- --checks v21 \
  --config ../typst-crystalline/crystalline.toml ../typst-crystalline
```

Nenhum V21 representou os achados abaixo:

- contexto apagado em `render.rs:448`, `svg.rs:440` e `stream.rs:1539–1542`;
- `style.variations` substituído por `FontVariations::default()` em
  `font_metrics.rs:1382` ao formar uma identidade de fonte;
- autoridade recomposta em `shaper.rs:2498` e classificador
  `ssty_eligible_text` duplicado em `font_metrics.rs:422` e `shaper.rs:59`.

V21 pergunta pela proveniência de um literal que escala contexto. Os novos achados
perguntam se informação ou autoridade declarada sobrevive a uma transformação. Fundir
as perguntas faria V21 abandonar seu predicado estreito, contrariando ADR-0016 e
ADR-0017.

## Decisão proposta

### 1. Três regras, uma infraestrutura de fatos semânticos

Adotar V23–V25 como regras distintas. Elas compartilham apenas uma IR neutra de fatos
de expressão e fluxo local. A taxonomia não será fundida:

- V23: preservação de contexto requerido;
- V24: completude de projeção/identidade;
- V25: unicidade e fronteira de autoridade decisória.

V25 inclui reentrada de canonicalizador porque owner, consumidor e marco resolvido
formam o mesmo contrato de autoridade. Um quarto código só será criado se fixtures
provarem que essa representação é mecanicamente insuficiente.

### 2. Contrato explícito; nenhuma inferência por nomes

O núcleo recebe contratos compilados da configuração e fatos extraídos do AST. Ele não
presume semântica de `0`, `default`, `abs`, `resolve`, `contains`, `math`, `Length`,
nomes de fonte ou paths do Typst.

Cada contrato possui `id` estável e `language`. Seletores de símbolo, função, parâmetro,
slot, campo e sumidouro são dados do projeto analisado, não constantes do núcleo.
Configuração inválida ou ambígua é erro de configuração; não é silenciosamente ignorada.

### 3. Análise limitada e honesta

A primeira versão usa fluxo intraprocedural, sensível a função, limitado a:

- parâmetros e `let` imutáveis;
- acesso a campo e projeção;
- chamadas, argumentos, retornos e slots de tupla/struct;
- composição booleana;
- chamadas a canonicalizadores antes/depois de um marco declarado.

Não há inferência de tipos, aliases interprocedurais, despacho dinâmico, macros opacas
ou fluxo por estado mutável. Quando o contrato exige prova fora desse limite, o caso é
registrado como não analisável e não gera falso positivo.

### 4. Forma proposta da configuração

```toml
[[semantic.context]]
id = "rounded-radius"
language = "rust"
scopes = ["03_infra/src/export/render.rs::shape_to_path"]
sources = ["radii.*"]
resolvers = [{ symbol = "resolve_pt", context_arg = 0 }]
erasing_projections = ["abs"]
sinks = ["rounded_rect_path"]
absolute_sources = []

[[semantic.projection]]
id = "font-identity-variations"
language = "rust"
scope = "03_infra/src/font_metrics.rs::resolve_font_combo"
source = "style.variations"
destination = "return.2"
neutral_forms = ["default", "none", "zero"]
normalization = "preserve"

[[semantic.decision]]
id = "ssty-eligibility"
language = "rust"
owner = "03_infra/src/font_metrics.rs::ssty_eligible_text"
consumers = ["03_infra/src/shaper.rs::*"]
proxies = []
canonicalizers = []
resolved_after = []
```

Listas adicionais podem declarar escopos do mesmo contrato. Glob de path serve apenas
para selecionar escopo; correspondência semântica ocorre sobre nós AST e símbolos
normalizados, nunca por busca textual do arquivo.

### 5. Severidade e ativação

- V23: `warning` por padrão;
- V24: `warning` por padrão;
- V25: `warning` por padrão;
- entram em `all`, mas um contrato vazio produz zero achados;
- seleção isolada `--checks v23`, `v24` ou `v25` não ativa as demais;
- níveis continuam configuráveis em `[rules]`;
- SARIF registra as três regras separadamente.

Sem contratos declarados, V23–V25 são conservadoras e silenciosas. Isso evita que a
instalação de uma versão nova transforme nomes comuns em violações.

## Matriz de decisão

| Padrão | Regra | Resultado | Razão mecânica |
|---|---|---|---|
| fonte contextual → `resolve_pt(0.0)` onde arg 0 é contexto | V23 | positivo | contexto neutro inventado |
| fonte contextual → `.abs` → sumidouro declarado | V23 | positivo | projeção apaga componente declarado |
| fonte absolute-only → `resolve_pt(0.0)` | V23 | negativo | exceção faz parte do contrato |
| `tracking.resolve_pt(style.size.val())` | V23 | negativo | contexto deriva da fonte declarada |
| zero fora de resolução registrada | V23 | negativo | literal isolado não tem semântica |
| `style.variations → return.2`, mas slot recebe default | V24 | positivo | campo obrigatório substituído por neutro |
| slot recebe `style.variations...unwrap_or_default()` | V24 | negativo | origem preservada; default só resolve ausência |
| default fora de projeção registrada | V24 | negativo | não existe contrato aplicável |
| contrato declara normalização para default | V24 | negativo | abandono autorizado explicitamente |
| consumidor recompõe `explicit || proxy` | V25 | positivo | proxy adiciona autoridade fora do owner |
| dois owners para o mesmo `id` | V25 | positivo | unicidade violada |
| canonicalizador após `resolved_after` | V25 | positivo | decisão reaberta após marco resolvido |
| consumidor chama o owner | V25 | negativo | autoridade preservada |
| heurística é owner declarada de outro `id` | V25 | negativo | identidades distintas |
| canonicalização dentro do owner | V25 | negativo | operação ocorre na fase proprietária |

## Consequências

**Positivas:** núcleo genérico; contratos auditáveis; zero comportamento por heurística
de nome; distingue verdadeiro negativo de caso não analisável; permite reconhecer os
oráculos sem codificar Typst no linter.

**Negativas:** configuração é mais verbosa; a IR e o índice global crescem; V25 exige
agregação cross-file; correspondência interprocedural permanece fora da primeira versão.

**Risco controlado:** seletores demasiado amplos podem criar ruído. O parser valida
unicidade de `id`, existência mínima de owner/slot e formas suportadas antes de executar.

## Alternativas rejeitadas

- Regexes ou listas internas de nomes: específicas do oráculo e frágeis.
- Estender V21: mistura proveniência de literal com preservação de informação.
- Inferir tipos por nomes: prova falsa e incompatível com a IR neutra.
- Exigir anotações no código do oráculo: impediria reconhecer os casos RED existentes;
  a configuração externa é necessária nesta fase.
- Similaridade textual para achar classificadores duplicados: acusa coincidências e não
  expressa autoridade.

## Gate de aprovação

ADR e prompts aprovados pelo humano em 2026-08-23. Os prompts V23–V25 tornam-se
vigentes; fixtures RED precedem a implementação e os hashes são resselados ao final da
materialização.
