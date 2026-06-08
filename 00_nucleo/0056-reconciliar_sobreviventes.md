# Laudo 0056 — reconciliar os 38 sobreviventes: inerte vs fora-do-oráculo

**Onde roda**: clone canônico do `tekt-linter` (com o conserto do 0052).
**Criado em**: 2026-06-08
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/reconciliar-sobreviventes.md`](prompts/reconciliar-sobreviventes.md)
**Continuação de**: laudo 0055. Trabalho de **verificação e documentação** — 0 código
novo (a reconciliação não achou nenhum sobrevivente que muda veredito).
**Camadas tocadas**: nenhuma. Só os laudos 0054/0055 (corrigidos in loco) e
`crystalline.toml` (exceção de órfão do prompt).

---

## Os dois furos do 0055 que este laudo fecha

1. **A itemização do 0055 somava 36, não 38.** A tabela listava
   `17 + 5 + 5 + 4 + 2 + 1 + 2 = 36`. Os 2 ausentes eram mutantes de
   `find_first_error_pos` na forma *substituir função inteira*
   (`928:5 -> (1,0)` e `-> (1,1)`), que o `grep` por sufixo `in <fn>` do 0055 não
   casou. Logo **`find_first_error_pos` são 19, não 17**. O "0 não-documentados"
   não estava demonstrado pela tabela.
2. **O rótulo "equivalente" misturava duas naturezas.** Chamar de "equivalente"
   um mutante que muda a **posição reportada** diz mais do que se prova: a posição
   é saída observável; o harness só não a pega porque afirma **IDs + contagem**.
   Isso é **fora-do-oráculo**, não equivalência.

## Reconciliação um-a-um (fonte: `mutants.out/missed.txt`)

`cargo mutants -j 4 --file '03_infra/rs_parser.rs'` →
`178 testados: 127 caught + 38 missed + 13 unviable` (totais conferem).

Cada um dos 38 cai em exatamente uma natureza. **Soma = 38, exata.**

### Muda veredito — 0

Nenhum. O motor de decisão (V1–V14) e a classificação ciente de deps
(`classify_import`/`resolve_subdir`) já tinham 0 no 0054; o 0055 fechou os 15 de
extração que afetavam veredito (V6 interface, V2 cobertura, V4 token-macro).

### Inerte — 8 (saída que nenhuma regra lê, ou código morto)

| Linha:função | Mutante | Prova de inércia |
|---|---|---|
| 213/215/216/217/218 `parse_layer_tag` | delete arms L0/L2/L3/L4/Lab | produz `PromptHeader.layer`; `grep` em `01_core/rules/` mostra que nenhuma regra o lê — a camada efetiva vem de `resolve_file_layer` (path) |
| 253 `collect_imports` | `&&`→`||` | só decide `ImportKind` (`Named` vs `Direct`); nenhuma regra lê `import.kind` (comentário em `forbidden_import.rs`: "V3 não usa ImportKind") |
| 786/788 `collect_type_param_names` | delete arms `type_identifier` / `constrained_type_parameter` | código morto sob `tree-sitter-rust 0.23` (só emite `type_parameter`); arms de compat de grammars antigas |

### Fora-do-oráculo — 30 (muda posição; o harness não testa posição)

| Linha:função | nº | O que muda |
|---|---|---|
| 928–946 `find_first_error_pos` | 19 | linha:coluna de um erro de sintaxe. Gated por `root.has_error()` (`rs_parser.rs:79`) — **não decide** se o `ParseError::SyntaxError` (V0/`PARSE`) é emitido, só a posição na mensagem |
| 247/266 `collect_imports` (`+`→`*`) | 2 | `line` do import reportado |
| 584/597 `collect_tokens` (`+`→`-`/`*`) | 4 | `line`/`column` do token (V4) |
| 813/822/833 `extract_declarations` (`+`→`-`/`*`) | 5 | `line` da declaração (V12) |

**8 inerte + 30 fora-do-oráculo + 0 veredito = 38.** ✓

## A lição, dita com precisão

"0 sobreviventes" é verdadeiro **contra o oráculo de veredito** — que regra dispara
e a contagem de IDs. O corpo de fixtures é **completo para vereditos**. **Não** é
completo para a saída inteira: a **posição** de uma violação e a posição de um erro
de sintaxe (V0/`PARSE`) são um **oráculo à parte**, hoje não exercitado pelo harness.

Isso é a mesma lição do 0054 recorrendo um nível abaixo: um verificador só prova o
que o seu oráculo consegue observar. O ganho permanece real e é exatamente esse —
nem mais (não é "saída inteira correta"), nem menos (é "todo veredito é mordido").

## Trilha à parte (nomeada, não decidida)

**Oráculo de posição.** Testar a linha:coluna reportada de violações e de erros de
sintaxe mataria os 30 fora-do-oráculo. É um contrato de teste de outra natureza
(mais frágil, acoplado à grammar). Fica **nomeado como trilha própria** — a decidir
se vale um prompt, sem decidir agora.

## Critérios de Verificação

- [x] `missed.txt` lido e transcrito; total da ferramenta confere
      (`127 + 38 + 13 = 178`).
- [x] Cada um dos 38 classificado em exatamente uma natureza; soma = 38, **sem
      buraco de 2** (os 2 eram `find_first_error_pos` substituição-de-função).
- [x] Nenhum "muda-veredito" achado → nenhuma fixture nova (0 código).
- [x] 0054 e 0055 reescritos: rótulos **inerte (8)** vs **fora-do-oráculo (30)**;
      "0 não-documentados" trocado por **"0 que mudam veredito; resto inerte ou
      fora-do-oráculo, itemizado"**.
- [x] Oráculo de posição/`PARSE` nomeado como trilha à parte (sem decidir).
- [x] Self-lint = 0; suíte verde.
- [x] Nada mascarado.

## Histórico de Revisões

- 2026-06-08 — Reconciliação dos 38 sobreviventes do `rs_parser` contra
  `missed.txt`. Corrigido o furo de 2 (`find_first_error_pos` = 19) e o rótulo
  agregado "equivalente" → **inerte (8)** + **fora-do-oráculo (30)** + veredito (0).
  Laudos 0054 e 0055 corrigidos in loco com ponteiro para cá.
