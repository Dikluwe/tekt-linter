# Passo 0070 (tekt-linter) — Travas mecânicas contra perda de contexto e autoridade semântica

> **Natureza:** comando operacional temporário para o LLM; não é regra da arquitetura  
> **Estado:** executado; ADR e L0 aprovados pelo humano  
> **Numeração:** herdada do `typst-crystalline`; sem significado canônico no linter  
> **Destino:** absorver em ADR/prompts e então arquivar ou eliminar  
> **Índice:** [`00_nucleo/README.md`](README.md)

**Repositório de implementação:** `tekt-linter`.
**Repositório-oráculo:** `typst-crystalline`.
**Precede este passo:** P0066–P0068 (`V21`/`V22`) e a auditoria posterior às
correções de paridade P1129–P1132 no `typst-crystalline`.

## Objetivo

Adicionar ao `crystalline-lint` proteção mecânica para três famílias de fuga que não
são literais empíricos cobertos por V21:

1. apagar o contexto necessário para resolver um valor ainda relativo;
2. perder um campo que participa de uma identidade semântica ao atravessar uma
   fronteira;
3. reconstruir ou sobrescrever, fora do dono, uma decisão semântica já tomada.

O resultado não deve ser uma coleção de regexes específicas do Typst. As regras devem
ter núcleo genérico, configuração/declaração explícita do contrato e fixtures mínimas
que demonstrem tanto os verdadeiros positivos quanto os usos legítimos próximos.

## Estado e proveniência da medição que originou o passo

Medição feita em 2026-08-23, no `typst-crystalline`, em `HEAD 781b207b4a5d`,
**working tree não commitado**. No momento da auditoria estavam alterados, entre outros,
`03_infra/src/font_metrics.rs`, `03_infra/src/shaper.rs`,
`03_infra/src/pipeline.rs` e os três exportadores. Antes de usar qualquer contagem para
fechar este passo, repetir a medição e registar `git rev-parse HEAD`,
`git diff HEAD --stat` e hora exata.

### Achados positivos que as novas regras devem representar

#### A. Apagamento de contexto

Os exportadores recebem `Corners<Length>` ainda capaz de conter componente relativo,
mas resolvem o raio com contexto neutro ou projetam apenas a parcela absoluta:

```rust
radii.top_left.resolve_pt(0.0)
radii.top_left.abs.0
```

Ocorrências-oráculo medidas:

- `typst-crystalline/03_infra/src/export/render.rs`, em `ShapeKind::RoundedRect`;
- `typst-crystalline/03_infra/src/export/svg.rs`, no mesmo braço;
- `typst-crystalline/03_infra/src/export/stream.rs`, no emissor de rounded rectangle.

O caso material `radius: 1em` confirmou perda visível: o cristalino produziu cantos
quadrados enquanto o vanilla ratificado produziu cantos arredondados. V21 não acusou
essas expressões.

#### B. Perda de campo semântico

`FallbackFontMetrics::resolve_font_combo` declara devolver a identidade
`(FontList, FontVariant, FontVariations)`, recebe `style: &TextStyle`, mas injeta:

```rust
FontVariations::default()
```

O caminho paralelo de `FrameItem::Text`/`TextShaped` preserva
`style.variations.clone().unwrap_or_default()`, e os exportadores comparam as variações
como parte da chave. A perda de informação é confirmada por inspeção; o efeito visual
continua por reproduzir com uma fonte variável e um `FrameItem::Glyph`.

Ocorrências-oráculo:

- `typst-crystalline/03_infra/src/font_metrics.rs::resolve_font_combo`;
- `typst-crystalline/03_infra/src/pipeline.rs::collect_fonts_in_items`;
- `typst-crystalline/03_infra/src/export/stream.rs::font_index_for_style`.

#### C. Reentrada/sobrescrita de decisão semântica

Dois padrões foram encontrados:

```rust
style.math || family_name.to_ascii_lowercase().contains("math")
```

reintroduz uma heurística por representação incidental depois de `style.math` já ser o
discriminante explícito; e `ssty_eligible_text` existe como classificador duplicado em
`font_metrics.rs` e `shaper.rs`, acompanhado de comentário dizendo que ambas as cópias
devem concordar.

O antigo reingresso de `map_glyph` fora da fase dona pertence à mesma família e deve ser
fixture de regressão, mesmo que já não exista no repositório-oráculo.

## Fase 0 — Medir antes de decidir

1. Ler integralmente os L0 vigentes do `tekt-linter` afetados, em especial:
   - `00_nucleo/prompts/linter-core.md`;
   - `00_nucleo/prompts/violation-types.md`;
   - `00_nucleo/prompts/unsourced-constant.md`;
   - prompts de CLI/config/parser que a solução realmente tocar.
2. Ler ADR-0016 e ADR-0017 para não fundir estas regras com V21/V22.
3. Reexecutar V21 no `typst-crystalline` e confirmar que os casos A–C continuam falsos
   negativos.
4. Produzir uma pequena matriz `padrão → regra → verdadeiro positivo/falso positivo`.
5. Não contar ocorrências por busca textual como prova de cobertura sem inspecionar a
   árvore sintática e o contrato de cada ocorrência.

## Fase 1 — Decisão arquitetural e nucleação obrigatória

As novas regras, seus números, a sintaxe de configuração/anotação e sua entrada ou não
no conjunto padrão de `--checks` são contrato público e comportamento por defeito do
linter. Portanto:

1. escrever um ADR que decida a taxonomia e o mecanismo genérico;
2. escrever/atualizar os Prompts L0 correspondentes;
3. incluir cenários Dado/Quando/Então positivos e negativos;
4. apresentar ADR e L0 ao humano e **PARAR**;
5. só continuar depois da confirmação humana e do resselo dos hashes.

Esta parada não pode ser substituída por este Passo de Execução: este documento planeia;
o L0 legitima o código.

## Taxonomia recomendada a validar na Fase 1

### V23 — `ContextErasure`

Detecta uma operação declarada como resolução contextual quando recebe um contexto
neutro inventado, ou quando um valor contextual é reduzido a uma projeção parcial antes
do sumidouro.

O contrato deve ser configurável/declarativo. A regra não deve presumir que todo `0.0`,
todo campo `abs` ou todo método `resolve_*` é incorreto.

Deve conseguir expressar mecanicamente:

- tipos/campos-fontes contextuais;
- métodos de resolução e posição do argumento de contexto;
- projeções que apagam componentes;
- sumidouros em que o valor resolvido é consumido;
- exceções auditáveis apenas quando o contrato declara que o valor é absoluto.

### V24 — `SemanticFieldLoss`

Detecta projeções declaradas de uma entidade para uma chave/DTO/identidade nas quais um
campo obrigatório da origem é abandonado, substituído por `Default::default()`, `None`,
zero ou outro neutro.

Não tentar inferir por nome que qualquer `default()` é perda. O vínculo
`origem.campo → destino.campo/slot` deve ser explícito em configuração, anotação ou tipo
de contrato analisável. Estados inválidos devem ser irrepresentáveis quando uma solução
por tipo for viável; a regra existe para fronteiras em que isso não seja possível.

### V25 — `DecisionOwnership`

Garante que uma decisão semântica nomeada tenha um único dono e que consumidores usem
esse dono em vez de:

- repetir o predicado;
- combinar o resultado explícito com proxy por string/nome;
- executar novamente o canonicalizador depois da fase proprietária.

O mecanismo deve usar identidade explícita da decisão (registro configurado ou anotação
estruturada), não similaridade textual entre funções. A solução precisa distinguir
`owner`, `consumer` e, se necessário, `resolved value`/`canonicalizer`.

Se a investigação mostrar que V25 reúne contratos mecanicamente diferentes demais,
separar `CanonicalizerReentry` somente mediante decisão no ADR; não criar um quarto
código por conveniência de implementação.

## Fora de escopo e guardas contra ruído

As novas regras **não** podem proibir genericamente:

- `Default::default()` ou `unwrap_or_default()`;
- literais zero;
- acesso a campos chamados `abs`;
- buscas por nome de família/font metadata;
- `Length` ou outro valor contextual dentro de uma estrutura transportada;
- classificadores parecidos apenas porque têm texto semelhante.

Também ficam fora deste passo as correções dos achados no `typst-crystalline`. Eles são
oráculos RED e devem ser corrigidos em passo próprio depois que o linter os reconhecer.

## Fase 2 — Testes RED antes da implementação

Após aprovação e resselo do L0, escrever fixtures atomizadas. Cada fixture deve primeiro
falhar pela ausência/comportamento incorreto da regra.

### V23: positivos obrigatórios

```rust
let radius = contextual_radius.resolve_pt(0.0);
let radius = contextual_radius.abs.0;
```

### V23: negativos obrigatórios

```rust
let radius = absolute_radius.resolve_pt(0.0); // contrato declara absolute-only
let tracking = tracking.resolve_pt(style.size.val());
let zero = 0.0;
```

### V24: positivo obrigatório

Uma projeção declarada `style.variations → FontIdentity.variations` que retorna
`FontVariations::default()` apesar de receber `style.variations`.

### V24: negativos obrigatórios

- a projeção preserva o campo;
- a origem não contém o campo opcional;
- o contrato declara explicitamente que o destino normaliza esse campo;
- `default()` fora de uma projeção registrada.

### V25: positivos obrigatórios

- consumidor declarado recompõe `explicit || name.contains("math")`;
- segundo owner para a mesma identidade de decisão;
- chamada ao canonicalizador depois do marco de valor resolvido;
- fixture representativa do antigo `map_glyph` downstream.

### V25: negativos obrigatórios

- consumidor chama o owner;
- heurística por string em domínio onde ela é a fonte canônica declarada;
- dois classificadores pertencentes a decisões com identidades diferentes;
- canonicalização dentro da fase proprietária.

Além dos testes da regra, cobrir dispatcher, `--checks v23,v24,v25`, `all`, configuração,
níveis, saída textual e catálogo SARIF. Uma seleção de V23 não pode ativar V24/V25.

## Fase 3 — Materialização

Somente depois dos testes RED:

1. implementar as entidades/contratos em L1;
2. estender o parser apenas com fatos sintáticos genéricos necessários;
3. manter leitura de configuração em L3 e composição em L4;
4. registrar V23–V25 no dispatcher, CLI, níveis e SARIF conforme decidido no ADR;
5. atualizar README/USAGE e CHANGELOG;
6. executar `--fix-hashes` para todos os L0 alterados;
7. instalar/rebuildar o binário usado pelo repositório-oráculo antes da validação cruzada.

Evitar análise de tipos fingida por nomes. Se o tree-sitter não fornecer prova suficiente,
preferir contrato explícito ou reduzir honestamente o escopo da primeira versão.

## Fase 4 — Validação cruzada

### No `tekt-linter`

```bash
cargo test --workspace
cargo run -- --checks v23,v24,v25 .
cargo run -- .
```

Resultado exigido: testes verdes e auto-lint sem violações.

### No `typst-crystalline`

Executar cada regra separadamente e em conjunto. Antes da correção dos achados, o
resultado esperado é:

- V23 identifica os três caminhos de raio contextual, sem acusar resoluções legítimas
  de tracking/text metrics que usam `style.size`;
- V24 identifica a perda de `style.variations` em `resolve_font_combo`, sem acusar a
  preservação usada pelos braços `Text`/`TextShaped`;
- V25 identifica a autoridade paralela `style.math || family.contains("math")` e/ou a
  duplicação declarada de `ssty_eligible_text`, conforme o mecanismo aprovado;
- V21 mantém o comportamento anterior.

Registar separadamente verdadeiro positivo, falso positivo e caso não analisável. Zero
falsos positivos nos exemplos negativos é gate; quantidade bruta de achados não é gate.

## Critérios de aceitação

1. ADR e L0 aprovados antes de qualquer alteração L1–L4.
2. Cada regra possui contrato mecânico genérico, com fonte e sumidouro/owner declarados;
   nenhuma depende de paths ou nomes exclusivos do Typst no núcleo.
3. Fixtures positivas falham antes da implementação e passam depois.
4. Fixtures negativas cobrem os padrões vizinhos legítimos e permanecem silenciosas.
5. CLI, seleção isolada, `all`, configuração, níveis e SARIF estão cobertos.
6. Auto-lint do `tekt-linter` termina sem violações.
7. Validação no `typst-crystalline` reconhece os oráculos indicados e não transforma
   V21 numa regra diferente.
8. Toda medição numérica usada no relatório final inclui commit/working tree, diff stat
   e, quando relevante, hora exata.

## Entrega e continuação

O relatório final deve separar:

- regras materializadas;
- achados confirmados no repositório-oráculo;
- falsos positivos encontrados e como o contrato os eliminou;
- limitações honestas da análise sintática;
- correções ainda pendentes no `typst-crystalline`.

Não corrigir silenciosamente os casos-oráculo durante este passo: eles serão a entrada
RED do passo seguinte no `typst-crystalline`.

---

## Registro de execução — Fases 0 e 1

**Medição:** 2026-08-23T11:49:13-03:00.  
**Oráculo HEAD:** `781b207b4a5de9c2bfbe5819918a193d1d9293e5`.  
**Working tree:** não commitado; `216 files changed, 9251 insertions(+), 4385 deletions(-)`,
mais arquivos não rastreados.  
**V21:** 45 diagnósticos, exit 0; nenhum representa A, B ou C.

Inspeção de AST/fonte confirmou:

- A: `render.rs:448`, `svg.rs:440`, `stream.rs:1539–1542`;
- B: perda em `font_metrics.rs:1382`, com preservações paralelas em
  `pipeline.rs:1005` e `export/stream.rs:203`;
- C: proxy em `shaper.rs:2498` e owners duplicados em
  `font_metrics.rs:422`/`shaper.rs:59`.

**Nucleação proposta:** ADR-0018 e prompts `context-erasure.md`,
`semantic-field-loss.md`, `decision-ownership.md`.

**Gate:** aprovado explicitamente pelo humano; execução retomada nas Fases 2–4.

## Relatório final — Fases 2 a 4

### Regras materializadas

- **V23 — `ContextErasure`:** contratos configuráveis reconhecem argumento contextual
  neutro e projeção que apaga a componente contextual, somente nos escopos declarados.
- **V24 — `SemanticFieldLoss`:** contratos relacionam campo de origem e slot de retorno
  e reconhecem a substituição por valor neutro numa projeção registrada.
- **V25 — `DecisionOwnership`:** contratos registram owner, consumidores, valor já
  resolvido e canonicalizador; a regra reconhece owner duplicado, proxy que recompõe a
  decisão e reentrada do canonicalizador.

As três regras foram integradas ao dispatcher, níveis, `--checks`, `all`, catálogo
SARIF e conjunto padrão. A implementação usa observações sintáticas genéricas; nomes e
paths do Typst existem apenas na configuração de validação do oráculo.

O checkpoint RED das fixtures produziu V23 `0/2`, V24 `0/1` e V25 `0/3` antes da
materialização das regras. Depois da implementação, os testes unitários e as fixtures
positivas, negativas e de seleção isolada passaram.

### Validação no repositório-oráculo

Com contratos explícitos aplicados ao estado medido acima, o binário encontrou:

- V23: **3** diagnósticos;
- V24: **1** diagnóstico;
- V25: **3** diagnósticos — os dois proxies observados e um segundo ponto verdadeiro
  do mesmo proxy em `shaper.rs:2413`.

Os exemplos negativos declarados permaneceram silenciosos. Não houve falso positivo
nos escopos configurados; o ruído genérico foi evitado porque zero, `default()`, acesso
a campo e busca textual só geram observação quando participam de um contrato nomeado.

### Limitações honestas

A primeira versão analisa Rust, é intraprocedural e sintática. Ela não resolve tipos,
expansão de macros, dispatch dinâmico ou fluxo interprocedural. A perda de campo é
reconhecida em slots de retorno em tupla declarados; owners, consumidores e escopos
precisam ser configurados explicitamente. Essas restrições evitam apresentar inferência
por nomes como prova semântica.

### Verificação e instalação

`cargo test --workspace` passou com **565 testes unitários** e **83 fixtures de caixa
preta**. O auto-lint terminou sem warnings ou erros. Os hashes L0 foram resselados e o
binário foi reinstalado por `cargo install --path . --force --locked` em
`/home/dikluwe/.cargo/bin/crystalline-lint`; a validação cruzada foi repetida com esse
binário instalado.

### Continuação

Nenhum arquivo do `typst-crystalline` foi corrigido neste passo. Os sete diagnósticos
confirmados permanecem como entrada RED para a correção própria no repositório-oráculo.
