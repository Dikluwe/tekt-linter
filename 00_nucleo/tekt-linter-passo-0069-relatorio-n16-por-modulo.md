# Passo 0069 (tekt-linter) — Relatório agregado de taxonomia N16 por módulo

**Repositório**: `tekt-linter`.
**Precede este passo**: Passo 0064 (calibração V16/V20), Passo 0066 (V21/V22, taxonomia
N16[α/β/γ] introduzida no ADR-0017), Passo 0068 (referência ao ADR no código-fonte de
V16).
**Motivação**: reconstrução manual da tabela cruzada por directório (lotes P1080/1082/1083,
92 casos N16[α/β/γ] classificados) já expôs um sinal real — `introspect/` concentra 66.7%
de γ (2/3), `layout/` tem o maior número absoluto de γ (4/20, 20%), `entities/`/`stdlib/`/
`eval/`/`parse/` têm zero γ. Essa reconstrução não tem lugar fixo no repositório — se um
`N16[γ]` novo for anotado no mês que vem, a tabela fica desactualizada em silêncio,
ninguém sabendo que precisa recalcular. Este passo formaliza a tabela como saída de
ferramenta, não como exercício manual pontual.
**Pré-condição**: `git status` limpo no `tekt-linter`; taxonomia `N16[α/β/γ]` do ADR-0017
já em uso nas anotações existentes (reclassificação de `// neutro:` para `N16[α/β/γ]`,
registada como "fora de âmbito" do ADR-0017 mas pré-condição funcional deste passo).

---

## Princípio reitor: relatório, não regra de gate

Esta tabela **não é uma regra V nova**. Não bloqueia CI, não tem predicado de violação,
não é acionável linha a linha. É extensão de saída de relatório sobre dados que V16 já
extrai (a tag `N16[α/β/γ]` de cada anotação de excepção). Mesmo princípio de V22: vigiar
tendência agregada, sem ruído por ocorrência.

## Ponto em aberto (decidir antes de implementar)

Encaixe do mecanismo: como flag de saída de V16 (`--checks v16 --format n16-summary`,
opção adoptada neste rascunho, porque é V16 quem produz as tags N16), ou como terceiro
modo de V22 (que já faz agregação por módulo, mas sobre dados de proveniência, não sobre
a taxonomia N16 — dados diferentes, mesmo padrão de agregação). Este passo assume a
primeira opção; se a decisão for outra, ajustar a Parte 1 antes de implementar.

## Parte 1 — Mecanismo

1. Novo modo de saída para o relatório de V16: `--format n16-summary` (alternativa a
   `--format sarif`, não substituto).
2. Para cada anotação `N16[α|β|γ]` já reconhecida pelo parser existente (Passo 0068,
   Parte 3), extrair o caminho do ficheiro e classificar por directório de primeiro nível
   dentro de `01_core/src/` (ou equivalente por linguagem, quando aplicável).
3. Agrupar por esse directório. Directórios sem nenhuma anotação `N16` não aparecem na
   tabela (não é preenchido com zero — ausência de dado não é o mesmo que zero confirmado).

## Parte 2 — Formato da tabela de saída

Colunas obrigatórias, nesta ordem: `módulo`, `total`, `α`, `β`, `γ`, `% γ`.

Regra de exibição do percentual: **contagem bruta é a informação primária**; o percentual
é secundário e vem sempre entre parênteses, nunca isolado — evita que quem lê o relatório
compare 66.7% (n=3) com 20% (n=20) como se tivessem a mesma confiança estatística.

Linha de aviso obrigatória para qualquer módulo com `total < 5`:
```
⚠ amostra pequena (n=3) — percentual pouco confiável, 1 caso muda o resultado em ~33pp
```
Limiar de 5 configurável via `[n16_summary] min_sample_size = 5` no `crystalline.toml`.

Linha de totais no rodapé (soma de todos os módulos, sempre presente, sem aviso de
amostra pequena mesmo que o total geral também seja baixo — o aviso é por módulo,
não para o agregado).

## Parte 3 — Ordenação

Ordenar por `γ` absoluto decrescente, não por `% γ` — o número absoluto de casos de maior
risco (γ) é a prioridade de leitura; o percentual é contexto, não critério de ordenação.
(`layout/`, com 4 casos absolutos de γ, deve aparecer antes de `introspect/`, com 2, mesmo
o percentual de `introspect/` sendo maior.)

## Parte 4 — Nível e escopo

`info`, opt-in — mesmo padrão de V22 (não corre por defeito em `crystalline-lint .` sem a
flag explícita). Não altera a severidade base de V16. Não introduz um V-número novo no
catálogo de regras — é um modo de relatório, catalogado no README na secção de formatos
de saída, não na tabela de verificações V0-V22.

## Fase de validação

1. Recorrer a `crystalline-lint --checks v16 --format n16-summary` contra o
   `typst-crystalline` já classificado (lotes P1080/1082/1083) e conferir, linha a linha,
   contra a tabela reconstruída manualmente:

   | Módulo | Total | α | β | γ | % γ |
   | :--- | :--- | :--- | :--- | :--- | :--- |
   | `introspect/` | 3 | 0 | 1 | 2 | 66.7% |
   | `math/layout/` | 2 | 0 | 1 | 1 | 50.0% |
   | `layout/` | 20 | 1 | 15 | 4 | 20.0% |
   | `03_infra/` | 12 | 0 | 11 | 1 | 8.3% |
   | `entities/` | 28 | 0 | 28 | 0 | 0.0% |
   | `stdlib/` | 19 | 0 | 19 | 0 | 0.0% |
   | `eval/` | 6 | 0 | 6 | 0 | 0.0% |
   | `parse/` | 1 | 0 | 1 | 0 | 0.0% |
   | `export/` | 1 | 1 | 0 | 0 | — |
   | **Total** | **92** | **2** | **82** | **8** | **8.7%** |

2. Confirmar que `introspect/` e `math/layout/` (n=3, n=2) exibem o aviso de amostra
   pequena; `layout/` (n=20) não exibe.
3. Confirmar ordenação por γ absoluto: `layout/` (4) antes de `introspect/` (2) e
   `math/layout/` (1).
4. Fixture nova: `tests/fixtures/n16_summary_small_sample.rs` — módulo sintético com 3
   anotações `N16`, confirma que o aviso de amostra pequena aparece.
5. Auto-validação: `crystalline-lint --checks v16 --format n16-summary .` no próprio
   repositório do `tekt-linter` — sem falha, mesmo que a tabela venha vazia (repo do
   linter pode não ter anotações N16).
6. Documentação: README, secção de formatos de saída, entrada para `n16-summary` com
   exemplo da tabela e explicação da regra de amostra pequena.

## Critérios de aceitação

1. Tabela gerada por comando bate 100% com a tabela reconstruída manualmente (todas as 9
   linhas + total).
2. Aviso de amostra pequena presente exactamente nos módulos com `total < min_sample_size`
   (2 módulos no conjunto de referência: `introspect/`, `math/layout/`).
3. Ordenação por γ absoluto verificada.
4. `min_sample_size` configurável e testado com valor alterado (ex.: `min_sample_size = 10`
   deve acrescentar `03_infra/` e `layout/` à lista de avisos).
5. Fixture verde, auto-validação verde, README actualizado.
6. Com 1–5 verdes: passo fechado, sem promoção de ADR (não é decisão de arquitetura nova,
   é extensão de relatório sobre mecanismo já aprovado no ADR-0017).
