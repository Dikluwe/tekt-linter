# Prompt L0 — Regra V21 `UnsourcedConstant` (proveniência de constantes em geometria/exportação)
Hash do Código: 734f6e1b

> Família de decisões mecânicas (ADR-0016 rev. 1). Irmã de V16–V20: onde elas
> tornam as *decisões* enumeráveis, V21 torna os *dados que alimentam as
> decisões* rastreáveis. Causalidade: este prompt é a origem de
> `01_core/rules/unsourced_constant.rs` e da extensão de IR correspondente.

**Decisão-mãe:** ADR-0016 (mesma família; V21 usa o nível `info` criado lá).
**Idioma desta fase:** Rust (`languages = ["rust"]`), mesmo desenho neutro de
IR: parsers sem extractor devolvem colecção vazia.

---

## 1. Pergunta que a regra responde

«Este valor literal/constante que determina geometria, metadado de exportação
ou output visual tem proveniência citada — referência, especificação ou
justificativa de design?»

Modo de falha visado: o placeholder que sobrevive porque **já tem nome** —
`const MIN_LEADING: Length = …` — e ninguém sabe de onde o número saiu. Uma
passagem manual depende de quem procura ter pensado nos sítios certos; a
varredura de AST é contínua e mecânica.

## 2. Escopo (proxy sintáctico em duas camadas)

Um ficheiro/função está em escopo se:

1. **Path do módulo** contém `layout/`, `export/`, `math/`, `shaper`,
   `geom` (lista `[v21_scope.modules]` em `crystalline.toml`, editável); **ou**
2. **Tipo de retorno da função** menciona `Frame`, `FrameItem`, `Length`,
   `Point`, `Size`, `Transform`, `Color`, `Paint` (lista `[v21_scope.types]`)
   — comparação por nome, sem resolução de tipos (coerente com ADR-0016 §4).

Funções de teste (`#[cfg(test)]`, `#[test]`) ficam fora de escopo: literais de
teste são asserções, não decisões de produção.

## 3. Alvos (a lente alargada)

O extractor (`rs_parser.rs`, query tree-sitter-rust) recolhe, dentro do
escopo:

| # | Alvo | Nota de AST |
| :-- | :---- | :---- |
| T1 | Literais numéricos no corpo de funções | `number_literal` |
| T2 | Literais de string no corpo de funções | `string_literal` |
| T3 | Definições `const`/`static`/assoc-const em módulos de escopo | `const_item`/`static_item` — **aqui moram os placeholders baptizados** |
| T4 | Literais em padrões de match e guards | `match_pattern` numérico/range |
| T5 | Strings de formato com especificador numérico | `format!("{:.2}")` — a precisão de coordenadas vive *dentro* da string |
| T6 | Literais negativos | `unary_expression` sobre literal — **não é** `number_literal` na árvore |

Fora de alvo: literais em closures dentro do escopo **entram** (são corpo de
função para este efeito); `..Default::default()` fica registado como
limitação conhecida (requer resolução de tipos; ADR futura se doer).

## 4. Allowlist de triviais (anti-ruído, obrigatória)

`0`, `1`, `-1`, `2`, `100`, `0.0`, `1.0`, `""`, strings de 1 char
(separadores), e literais em posição de expoente/índice óbvio. Lista em
`[v21_trivial]` para ajuste sem tocar na regra. Sem esta allowlist o
typst-layout geraria milhares de infos e a regra morria no primeiro dia —
lição herdada da R5/isenção de tabelas.

## 5. Gramática de citação (o que apaga o aviso)

Comentário na linha imediatamente anterior ou na mesma linha do alvo, numa
de três formas (regex exacta no extractor):

```
// ref: <caminho>:<linha>        — citação de fonte no repo (spec, oracle, fonte tipográfica)
// spec: <norma> §<secção>       — citação externa (PDF 1.7 §8.4, CSS Fonts L4, OpenType…)
// rationale: <frase>            — decisão de design sem fonte externa (legítimo; proíbe
//                                 a regra de incentivar fontes inventadas)
```

A forma `rationale:` existe por honestidade: muitas constantes tipográficas
são escolhas, não citações. O que a regra exige é *proveniência declarada*,
não bibliografia forçada.

## 6. Anti-apodrecimento (reutiliza a maquinaria de excepções)

- `ref: <ficheiro>:<linha>` é verificado: o ficheiro existe e a linha ainda
  contém conteúdo compatível (heurística: linha não-vazia, mesma chave de
  contexto). Span obsoleto → aviso próprio `StaleCitation` (mesmo mecanismo
  de `[wildcard_exceptions]` do ADR-0016 — mesma função, dois consumidores).
- Entradas `spec:`/`rationale:` não apodrecem mecanicamente; revisão humana
  em release.

## 7. Diagnóstico e nível

- Nível: **info** (V21 é métrica de proveniência, não defeito). Ratchet a
  `warning` por módulo quando saneado (`[v21_strict]` = lista de módulos).
- Mensagem: cita o snippet verbatim do literal/constante + o termo da
  linguagem (`decision_arm_term_for` generaliza para
  `source_term_for(language, construct)`) + qual das três formas de citação
  falta.
- Métricas reportadas (sempre, mesmo sem violações): rácio de constantes com
  proveniência por módulo de escopo — o número a acompanhar no tempo.

## 8. Critérios de aceitação

1. Corrida sobre typst-crystalline: relatório por módulo de escopo com
   contagens por alvo (T1–T6) e rácio de proveniência; o número total é
   declarado baseline datada (sem pretensão de zero no nascimento).
2. Fixtures: (a) `const` sem comentário dispara; (b) `// ref:` válido apaga;
   (c) `ref:` obsoleto dispara `StaleCitation`; (d) `-0.5` (T6) é detectado;
   (e) `format!("{:.3}")` (T5) é detectado; (f) triviais da allowlist nunca
   disparam; (g) função de teste nunca dispara.
3. Precisão da allowlist: amostra de 30 infos; se < 80% forem constantes que
   um revisor humano quereria citadas, a allowlist alarga-se e re-corre.
4. Auto-validação verde; zero V21 em projectos TS/Python de referência.

## 9. Validação

```bash
cargo test -p crystalline-lint --lib
cargo test --test fixtures
crystalline-lint .
crystalline-lint --checks v21 /caminho/typst-crystalline
```

## 10. Fora de escopo (registado)

- Resolução através de `..Default::default()` e de constantes de crates
  externas (precisa de tipos — ADR futura se a dor se confirmar).
- Verificação *semântica* da citação (a regra verifica existência e
  frescura, não que a fonte diz o que a constante afirma — isso é revisão).
- Ratchet a `error`: V21 nunca sobe além de warning por módulo; proveniência
  ausente não é defeito de compilação.

---

## 11. Fundamentação Teórica de V21 (HardcodedContextualValue)

1. **Where-Provenance de Fatores de Escala Contextual:**
   * **Buneman et al. (2001)** (*Why and Where: A Characterization of Data Provenance*): A teoria de *Where-Provenance* estabelece que fatores de transformação aplicados a variáveis de contexto de entrada requerem autoridade de proveniência rastreável. A regra V21 vigia especificamente escalares que multiplicam variáveis de contexto (`em`, `font_size`, `frame`) para alimentar sumidouros geométricos (`Length`, `gap`, `offset`), exigindo citação formal de norma (`// spec:`), referência a oráculo (`// ref:`) ou decisão de design (`// rationale:`).
2. **Verificação Mecânica de Frescura de Vínculos (Anti-Apodrecimento):**
   * **Erata et al. (2017, 2024)** (*A Tool for Automated Reasoning about Traces Based on Configurable Formal Semantics*): Vínculos de rastreabilidade informais sofrem de degradação rápida (*trace decay*). Conforme fixado no ADR-0017, como V21 vigia um fato estático (escalar auditável), o silenciamento por citação inline `// ref:` é legítimo, mas acompanhado da verificação contínua de frescura que dispara `StaleCitation` se a âncora referenciada for alterada ou removida.
3. **Prevenção de Fórmulas e Escalares Ocultos (Contextual Magic Numbers):**
   * **Fowler (1999)** (*Refactoring: Improving the Design of Existing Code*): Fatores escalares embutidos diretamente em operações de cálculo de layout sem documentação formal de derivação constituem números mágicos contextuais. V21 emite `Warning` para forçar a explicitação da origem do multiplicador ou sua extração para constantes com proveniência formal.
