# Prompt L0 — Calibração de V16/V20 (correção pós-validação cruzada)

> Revisão do prompt L0 `wildcard-saturation.md` (ADR-0016). Esta correcção
> nasce da primeira corrida real sobre o typst-crystalline (passo 0063):
> V16 reportou 197 avisos vs 176 de referência (+12%), e dois desvios de
> classificação foram identificados. Causalidade: o comportamento muda aqui
> primeiro; o código em `01_core/rules/` e `03_infra/rs_parser.rs` materializa.

**Decisão-mãe:** ADR-0016 (PROPOSTO) — esta calibração responde à questão em
aberto nº 1 (concordância ≥ 95% por categoria) e desbloqueia o status ACEITO.

---

## 1. Problema a corrigir (diagnóstico da corrida de referência)

**Desvio 1 — `format!` produtor de mensagem de erro classificado como
saturação.** Braços como `_ => format!("cannot apply {op:?} to {a} and {b}")`
foram reportados como DENY-class (saturação arbitrária). Mas um braço que
produz uma mensagem de erro **falha ruidosamente** — é o oposto do modo de
falha que V16 visa (adopção silenciosa). A causa: `ErrorBarrier` detecta
`panic!`/`bail!`/`unreachable!` mas não macros produtoras de mensagem.

**Desvio 2 — isenção de contexto-tabela de V20 por calibrar.** V20 reportou
515 infos contra a expectativa de «dezenas» após isenção. Ou o proxy de
«tabela regular» está demasiado estrito, ou há mais aninhamento ad-hoc real
do que a amostra inicial sugeria — decidir com dados, não por preferência.

**Desvio 0 (registar, já correcto):** `None` foi classificado como default
neutro na corrida — formalizar: `None` entra na tabela de neutros.

## 2. Alteração à especificação de V16

### 2.1 Nova forma de corpo: `MessageProducer`

`BodyForm` ganha a variante `MessageProducer`: corpo do braço é invocação de
`format!`, `format_args!`, `write!`/`writeln!`, ou chamada a função cujo nome
começa por `error`/`err_`/`cannot_`/`expected_` (heurística de nome, Rust-only).

### 2.2 Reclassificação no pipeline

`MessageProducer` é tratado como **barreira ruidosa → ISENTO de V16**, com a
mesma justificativa das barreiras de erro: o critério de V16 é *silêncio*.
Um braço que grita (`cannot apply ...`) não adopta silenciosamente — falha na
hora. (Nota de escopo: «mensagens de erro deviam ser erros tipados» é uma
regra futura distinta; não é trabalho de V16.)

Pipeline corrigido (ordem estrita):

```
catchall detectado
 ├─ reincorporação ...................... ISENTO
 ├─ scrutinee aberto (MethodCall/Index/Literal)  ISENTO
 ├─ body_form = ErrorBarrier ............ ISENTO (falha ruidosa em compile/runtime)
 ├─ body_form = MessageProducer ......... ISENTO (falha ruidosa; NOVO)
 └─ enum candidato
     ├─ EnumPath | LiteralOther ......... DENY-class
     ├─ LiteralNeutral (inclui `None`) .. WARN-class neutro
     ├─ Call ............................ INFO delegação
     └─ EmptyBlock | Continue ........... WARN-class walker
```

### 2.3 Tabela de neutros (forma final)

`false`, `true`, `0`, `0.0`, `()`, tupla de neutros, `Default::default()`,
`None`, `String::new()`, `Vec::new()`, `vec![]` (macro construtora vazia —
distinguir de macros de pânico na extracção).

## 3. Alteração à especificação de V20

### 3.1 Proxy de contexto-tabela (forma final)

Um match é **tabela regular** (isento de V20) se ≥ 80% dos braços cumprem:
mesma aridade de tupla (ou todos não-tupla), os padrões de cada posição
partilham a mesma família de construtores (ex.: todos `Some(...)` na posição
1), e os corpos são da mesma forma dominante. Guards **não** desqualificam uma
tabela (linhas de tabela podem ter guardas simples).

### 3.2 Auditoria obrigatória da amostra

Antes de fechar a calibração: amostra aleatória de 20 dos 515 infos,
classificação manual (ad-hoc real vs tabela não reconhecida). Se < 80% forem
ad-hoc reais → o proxy em 3.1 está errado, ajustar e re-correr. Se ≥ 80% →
o número 515 é honesto e regista-se no relatório como a superfície real de
aninhamento ad-hoc do repositório.

## 4. Critérios de aceitação da correcção

1. Re-corrida sobre typst-crystalline: V16 DENY-class ∈ [20, 32] (referência
   26 ± margem); o caso `cannot apply` está isento; concordância por
   categoria ≥ 95% (tabela V16 × subcategoria × syn vs linter no relatório).
2. V20: auditoria da amostra de 20 documentada; decisão proxy-vs-número
   registada com a percentagem medida.
3. Fixture nova: `tests/fixtures/error_message_arm.rs` — braço
   `_ => format!("cannot ...")` em enum fechado **não** dispara V16.
4. Regressão: V17 = 29 e V18 = 2 inalterados; V19 ∈ [250, 280].
5. Auto-validação verde; não-regressão TS/Python (zero V16–V20) inalterada.
6. Com 1–5 verdes: ADR-0016 promovido a **ACEITO** no relatório do passo.

## 5. Validação

```bash
cargo test -p crystalline-lint --lib
cargo test --test fixtures
crystalline-lint .
crystalline-lint --checks v16,v17,v18,v19,v20 --format sarif /caminho/typst-crystalline
```

## 6. Fora de escopo (registado para não reabrir neste passo)

- O refactor dos casos DENY no typst-crystalline (26±) é passo separado,
  naquele repositório, depois do ADR-0016 ACEITO.
- «Mensagens de erro como erros tipados» é regra futura candidata (V21?),
  não calibração de V16.
