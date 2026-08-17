# Prompt L0 — Regra V21 `HardcodedContextualValue` (escalares contextuais em variáveis de contexto e sumidouros geométricos)
Hash do Código: 734f6e1b

> Família de decisões mecânicas (ADR-0016 / ADR-0017 / Passos 0066 e 0068).
> Causalidade: este prompt é a origem de `01_core/rules/unsourced_constant.rs`
> e da infraestrutura de extração em `03_infra/rs_parser.rs`.

**Decisão-mãe:** ADR-0016 (Rev. 1) e ADR-0017 (Diferença Categórica V16 vs. V21).
**Idioma desta fase:** Rust (`languages = ["rust"]`).

---

## 1. Pergunta que a regra responde

«Este escalar numérico que multiplica ou divide uma variável de contexto de entrada
para determinar um sumidouro geométrico possui proveniência declarada — referência de
especificação (`// spec:`), oráculo de implementação (`// ref:`) ou decisão explícita
de design (`// rationale:`)?»

Modo de falha visado: fatores de escala e números mágicos contextuais arbitrários
embutidos em rotinas de layout (`layouter.regions.current.cursor_y += style.size * 0.6`)
sem justificativa ou origem rastreável, gerando acoplamentos implícitos frágeis.

---

## 2. Predicado Estrito de V21 (A Lente Estreita)

Diferente de uma varredura cega por literais no código, a regra V21 opera sobre um
**predicado relacional estrito em 3 eixos**:

1. **Operação de Escalonamento Binário (`is_in_binary_scaling`):**
   O literal numérico deve ser operando de multiplicação (`*`) ou divisão (`/`)
   em expressões de atribuição simples (`=`) ou composta (`+=`, `-=`, `*=`, `/=`).
2. **Fonte Contextual (`context_var`):**
   A variável escalada deve ser uma fonte contextual de layout:
   `size`, `em`, `style`, `font`, `weight`, `ascent`, `descent`, `width`, `height`,
   `depth`, `frame`, `region`, `page`, `margin`, `padding`, `container`.
3. **Sumidouro Geométrico (`geometric_sink`):**
   O resultado da operação deve alimentar uma propriedade de dimensão ou posição:
   `cursor_y`, `cursor_x`, `cursor`, `gap`, `inset`, `offset`, `pos`, `x`, `y`,
   `width`, `height`, `thickness`, `ascent`, `descent`, `length`, `pt`, `em`,
   `ratio`, `abs`, `point`, `size`, `frame` — incluindo cadeias de campos profundos
   (ex: `layouter.regions.current.cursor_y`, 3+ níveis).

---

## 3. Isenções e Filtros Anti-Ruído

1. **Módulos de Sintaxe de Formato Fixo (`format_syntax_modules`):**
   Arquivos em `export/pdf`, `export/svg` são isentos — operadores PDF e tags SVG
   utilizam constantes canônicas de formato que não representam lógica contextual.
2. **Origem em Testes e Tabelas de Dados:**
   Constantes originadas em `#[cfg(test)]`, módulos de fixture ou tabelas de
   tradução direta (`is_in_data_table`) são ignoradas.
3. **Allowlist de Literais Triviais (`trivial_literals`):**
   Valores neutros canônicos (`0`, `1`, `-1`, `2`, `100`, `0.0`, `1.0`, `""`,
   strings de 1 caractere) são ignorados.

---

## 4. Gramática de Citação e Resolução

V21 vigia um fato estático (escalar auditável); portanto, nos termos do ADR-0017,
o silenciamento por citação explícita é legítimo. A anotação deve constar na linha
do alvo, na linha imediatamente anterior ou na janela de proximidade de 3 linhas:

```
// ref: <caminho>:<linha>        — citação de oráculo/fonte no repositório
// spec: <norma> §<secção>       — citação de padrão formal externo (CSS, OpenType, PDF)
// rationale: <justificativa>    — decisão intencional de design sem fonte externa
```

### Anti-Apodrecimento (StaleCitation)
Quando a citação usa `// ref: <caminho>:<linha>`, a regra verifica a existência do
arquivo e a validade da linha referenciada. Se o alvo for movido, excluído ou ficar
vazio, V21 emite um aviso específico de citação obsoleta (`StaleCitation`).

---

## 5. Diagnóstico e Níveis de Severidade

- **Nível Padrão:** `Warning`.
- **Módulos Estritos (`[v21_strict]`):** Promovido a `Error` para módulos onde a
  auditoria de proveniência já foi completamente saneada.
- **Mensagem Formatada:** Cita o literal, a variável contextual e o sumidouro
  geométrico afetados, instruindo a inclusão da anotação de proveniência.

---

## 6. Critérios de Verificação

```
Dado literal 0.6 multiplicando style.size e atribuído a cursor_y sem anotação
Quando V21::check() for chamado
Então retorna Violation V21 Warning

Dado layouter.regions.current.cursor_y += layouter.style.size * 0.6 (campo profundo)
Quando V21::check() for chamado
Então retorna Violation V21 Warning identificando size e cursor_y

Dado literal com comentário // ref: vanilla/layout.rs:120 válido
Quando V21::check() for chamado
Então retorna vec![] — silenciado legitimamente

Dado literal com // ref: apontando para arquivo ou linha inexistente
Quando V21::check() for chamado
Então retorna Violation V21 StaleCitation

Dado literal trivial 0.0 ou 1.0 em operação contextual
Quando V21::check() for chamado
Então retorna vec![] — trivial da allowlist
```

---

## 7. Fundamentação Teórica de V21 (HardcodedContextualValue)

1. **Where-Provenance de Fatores de Escala Contextual:**
   * **Buneman et al. (2001)** (*Why and Where: A Characterization of Data Provenance*): A teoria de *Where-Provenance* estabelece que fatores de transformação aplicados a variáveis de contexto de entrada requerem autoridade de proveniência rastreável. A regra V21 vigia especificamente escalares que multiplicam variáveis de contexto (`em`, `font_size`, `frame`) para alimentar sumidouros geométricos (`Length`, `gap`, `offset`), exigindo citação formal de norma (`// spec:`), referência a oráculo (`// ref:`) ou decisão de design (`// rationale:`).
2. **Verificação Mecânica de Frescura de Vínculos (Anti-Apodrecimento):**
   * **Erata et al. (2017, 2024)** (*A Tool for Automated Reasoning about Traces Based on Configurable Formal Semantics*): Vínculos de rastreabilidade informais sofrem de degradação rápida (*trace decay*). Conforme fixado no ADR-0017, como V21 vigia um fato estático (escalar auditável), o silenciamento por citação inline `// ref:` é legítimo, mas acompanhado da verificação contínua de frescura que dispara `StaleCitation` se a âncora referenciada for alterada ou removida.
3. **Prevenção de Fórmulas e Escalares Ocultos (Contextual Magic Numbers):**
   * **Fowler (1999)** (*Refactoring: Improving the Design of Existing Code*): Fatores escalares embutidos diretamente em operações de cálculo de layout sem documentação formal de derivação constituem números mágicos contextuais. V21 emite `Warning` para forçar a explicitação da origem do multiplicador ou sua extração para constantes com proveniência formal.

---

## 8. Fundamentação Teórica de V22 (ProvenanceInventory)

1. **Proveniência como Propriedade Auditável (motivação teórica):**
   * **Buneman et al. (2001)** (*Why and Where: A Characterization of Data Provenance*): O princípio fundamental de que artefatos e valores derivados devem carregar linhagem auditável (*where-provenance*) motiva a necessidade de acompanhar a rastreabilidade dos dados do sistema. A regra V22 opera com um instrumento pragmático de engenharia de software — uma razão percentual de literais anotados sobre o total por módulo —, sem pretender reconstruir a linhagem formal de cada valor individual.
2. **Padrões de Co-Evolução e Desalinhamento (motivação teórica):**
   * **Rahimi & Cleland-Huang (2015)** (*Patterns of Co-evolution between Requirements and Source Code*): O catálogo empírico de desvios entre especificações e código demonstra que a perda de rastreabilidade ocorre ao longo da manutenção contínua. A regra V22 aplica essa vigilância na forma de uma métrica de tendência agregada por módulo — uma queda no rácio sinaliza a necessidade de auditoria, mesmo que nenhuma linha isolada atinja o predicado estrito de V21.
3. **Observabilidade Estrutural por Subsistema:**
   * **Ducasse & Pollet (2009)** (*Software Architecture Reconstruction: A Process-Oriented Taxonomy*): A governança arquitetural se beneficia de métricas agregadas por subsistema que complementem regras locais de gate. V22 adapta esse princípio ao contexto de acompanhamento contínuo de código, oferecendo observabilidade macroscópica em formato condensado de linha única por módulo (nível `Info`, opt-in).
