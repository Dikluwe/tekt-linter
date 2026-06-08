# Prompt: reconciliar os 38 sobreviventes e separar inerte de fora-do-oráculo

> Numere em sequência (provável 0056) e salve em `00_nucleo/prompts/`.
> Alvo: o repo do **linter** (clone com o conserto do 0052).
> Continuação direta do laudo 0055. Trabalho majoritariamente de **verificação e
> documentação**; só vira código se a reconciliação achar um sobrevivente que muda
> veredito (aí, fixture).

## Contexto — duas coisas a fechar no 0055

O 0055 reportou **38 sobreviventes** no `rs_parser.rs` (vindos da ferramenta,
confiáveis) e marcou "0 não-documentados". Mas há dois problemas a resolver antes
de tratar isso como fechado:

1. **A itemização não soma 38.** A tabela final lista 17 + 5 + 5 + 4 + 2 + 1 + 2 =
   **36**. Faltam **2 sobreviventes** individualizados. Então o critério "0
   não-documentados" não está demonstrado pela tabela — ou foram 17 mortos e não
   15, ou há 2 sem classificar.
2. **"Equivalente" está cobrindo duas naturezas diferentes.** Dos 38, só **8**
   são equivalentes no sentido forte — mutá-los não muda comportamento observável
   nenhum:
   - `parse_layer_tag` (5): produz `PromptHeader.layer`, que nenhuma regra lê.
   - `collect_imports:253` (1): só altera `ImportKind`, que nenhuma regra lê.
   - `collect_type_param_names:786/788` (2): arms de grammar antiga, código morto
     sob `tree-sitter-rust 0.23`.

   Os outros **28** — `find_first_error_pos` (17) e a aritmética de linha:coluna
   (11) — **mudam comportamento observável**: a posição reportada de uma violação
   e a posição de um erro de sintaxe. O harness não os pega porque afirma **IDs +
   contagem, não posição**. Isso não os torna inertes; torna-os **fora do oráculo**
   — comportamento real que o corpo, por desenho, escolheu não testar. Chamá-los de
   "equivalentes" diz mais do que se provou.

   É a mesma lição recorrendo dentro do corpo: "0 sobreviventes" é verdadeiro
   contra o oráculo de **veredito**, que é cego à posição. O ganho real é: o corpo
   é **completo para vereditos**. Não é completo para a saída inteira.

> **Nota de nucleação (0056)**: a reconciliação contra o `missed.txt`
> autoritativo achou os 2 que faltavam — são mutantes de `find_first_error_pos`
> na forma *substituir função inteira* (`928:5 -> (1,0)` e `-> (1,1)`), que o
> `grep` por sufixo `in <fn>` do 0055 não casou. Logo `find_first_error_pos`
> são **19, não 17**, e fora-do-oráculo são **30, não 28** (19 + 11 de
> linha:coluna). Inerte = 8. Veredito-mudante = 0. Soma = 38.

## Pré-condição

Clone do 0052; estado do 0055 (29+8 fixtures, self-lint = 0, suíte verde fora do
`blanket_impl`).

## Tarefa

### 1. Lista crua e reconciliação um-a-um

Re-rodar a mutação no escopo e ler a lista autoritativa de sobreviventes do
diretório de saída — não confiar na tabela do laudo:

```
cargo mutants -j 4 --file '03_infra/rs_parser.rs'
cat mutants.out/missed.txt      # os sobreviventes, um por linha (= "MISSED")
# conferir os totais:
wc -l mutants.out/caught.txt mutants.out/missed.txt mutants.out/unviable.txt
```

Bater `missed.txt` contra a tabela do 0055. Para **cada** sobrevivente (os 38, ou
o número que a ferramenta der agora), uma de três decisões, explícita:

- **Muda veredito** (qual regra dispara / contagem de IDs) → não é equivalente nem
  fora-do-oráculo: **matar com fixture** (bite-proof; afirmar IDs+contagem),
  re-rodar, confirmar morto.
- **Fora do oráculo**: muda só posição (linha:coluna de violação) ou posição de
  erro de sintaxe V0/`PARSE`. Registrar com linha, função e o que muda.
- **Inerte**: saída que nenhuma regra lê, ou código inalcançável sob a grammar
  pinada. Registrar com a prova (o `grep` que mostra que ninguém lê, ou a nota da
  grammar).

Os **2 que faltam** têm de cair numa dessas três, nomeados. Ao fim, a soma das
três categorias = o total da ferramenta, exatamente.

### 2. Reenquadrar os laudos 0054 e 0055

- Substituir o rótulo único "equivalente (38)" por **duas categorias**: **inerte
  (8)** e **fora-do-oráculo (28)** — mais os 2 reconciliados onde caírem.
- Trocar a afirmação "0 sobreviventes não-documentados" pela afirmação que se
  prova: **"0 sobreviventes que mudam veredito; cada sobrevivente restante é
  inerte (saída não-lida/código morto) ou fora-do-oráculo (posição), itemizado"**.
- Registrar que **posição de violação e posição de erro de sintaxe (V0/`PARSE`)
  são um oráculo à parte**, hoje não testado, a decidir se vale um prompt próprio.
  Não decidir agora; só nomear.

## Critérios de Verificação

- [ ] `missed.txt` lido e anexado/transcrito; total da ferramenta confere com
      `caught + missed + unviable`.
- [ ] Cada sobrevivente classificado em exatamente uma de: muda-veredito (morto) /
      fora-do-oráculo / inerte — soma = total da ferramenta, **sem buraco de 2**.
- [ ] Qualquer "muda-veredito" achado foi morto por fixture bite-proof e re-rodado.
- [ ] 0054 e 0055 reescritos: rótulos inerte vs fora-do-oráculo; a afirmação de
      completude trocada por "completo para vereditos".
- [ ] Oráculo de posição/`PARSE` nomeado como trilha à parte (sem decidir).
- [ ] Self-lint = 0; suíte verde fora do `blanket_impl` pré-existente.
- [ ] Nada mascarado.

## Fora de escopo (trilhas seguintes)

- Construir o **oráculo de posição** (testar linha:coluna), se decidido depois.
- Os três **detectores contra a linguagem**: contador de `Layer::Unknown` em alvo
  real; oráculo diferencial contra a lente (`tekt-cargo-dsm`); corpus de projetos
  reais variados.
- A **decisão de merge** com o `master` público (multi-linguagem + Hash Locking ⊕
  conserto do 0052).

## Disciplina (do repo)

Verificação da fonte, um a um; nada de rótulo agregado que esconde diferença de
natureza; se nascer código, prova-de-mordida; laudo ao fim, e correção dos laudos
anteriores in loco com nota no histórico de revisões.
