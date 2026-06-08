# Prompt: oráculo diferencial linter × lente (verificar arquitetura, para fora)

> Numere em sequência (provável 0058) e salve em `00_nucleo/prompts/` do **linter**.
> Cruza dois repos: o **linter** (clone do 0052) e a **lente** (`tekt-cargo-dsm`).
> Primeira peça da virada "para fora": verificar arquitetura real, não regressão interna.

> **Nota de nucleação (0058)**: passo-zero demonstrado — `montar_grafo_workspace`
> produziu 507 nós no workspace da lente (fork 0.27.0). Os testes `#[ignore]` são
> **35**, não 28. O harness vive em `oraculo/` (crate standalone no repo do linter).

## Por que este teste é diferente dos anteriores

A mutação acha as distinções que o código **já faz** mas nenhuma fixture testa. Ela
**não** acha a distinção que o código **deixa de fazer** — e é aí que mora a deriva
arquitetural real. A inversão L2→L4 da anamnese não foi achada mutando o linter; foi
achada rodando o linter num projeto real, numa forma (L1 multi-crate) que o modelo
dele não representava. O ponto cego veio do mundo.

O oráculo é uma **segunda computação independente** da estrutura. A lente resolve
dependências pelo grafo do compilador (via o fork `cargo-modules`); o linter resolve
imports por análise textual (`classify_import`). Caminhos independentes → **não
compartilham os mesmos pontos cegos**. Nas mesmas entradas, têm de concordar sobre
quem depende de quem. Onde discordam, um dos dois tem um ponto cego numa arquitetura
real. É o modo de falha da anamnese virado para fora.

Assimetria a usar na triagem: o lado da lente é resolvido pelo compilador (mais
perto da verdade); o do linter é heurística textual. Então o **prior** numa
discordância é "ponto cego do linter" — mas não certeza: a lente/fork tem seus
próprios casos de borda (macros, `cfg`, `uses_kind` ausente). Cada discordância é
triada, não presumida.

## Pré-condição (e a convergência com o débito da lente)

1. Clone do linter com o 0052; **0057 reconciliado** (o furo de contagem 66/11/59 —
   ver crítica; o oráculo não depende dele, mas não deixe pendência aberta a montante).
2. A lente disponível e **funcionando de verdade**: o fork `cargo-modules` instalado,
   e `montar_grafo_workspace(raiz)` produzindo um `Grafo` real num workspace real.
   **Isto toca o maior débito da lente** — os 28 testes `#[ignore]` (git/workspace/
   diff reais nunca exercitados). O oráculo não roda se o caminho real da lente não
   roda. Então o passo zero é confirmar que a lente produz grafo num workspace real;
   se não produzir, **pare e reporte** — esse débito vira pré-requisito, e é o fio
   que originou toda a anamnese.

## O observável comum (onde os dois são comparáveis)

Para cada import de um workspace Rust compilável:

- **Lado linter**: a resolução — `(módulo-origem, caminho-importado) → (crate-alvo,
  módulo-alvo, camada-alvo, ImportKind, é_Unknown)`.
- **Lado lente**: a aresta correspondente do `Grafo` — `(módulo-origem) →
  (módulo-alvo)`, com a `Relation`.

**Comparar primeiro no nível de aresta** (módulo-origem → módulo/crate-alvo), onde a
independência é real (compilador vs texto). A **camada** é secundária e usa a mesma
projeção `[layers]`/path nos dois lados — é componente de **modo-comum** (se o
`[layers]` estiver errado, os dois herdam o erro); então rotular camada serve de
contexto, não é o sinal forte. O sinal forte é a aresta resolvida, que é exatamente
o que o linter cego errou na anamnese (pôs a aresta cross-crate em `Unknown`).

## Tarefa

### A. Linter — modo de despejo de resolução (saída nova, fora do selo de veredito)

Adicionar uma saída que, para **todo** import do workspace, emite a resolução
completa — crate/módulo-alvo, camada, `ImportKind`, e **explicitamente os
`Unknown`** (os silenciosos; o SARIF de hoje só mostra violações e os esconde).
Formato parseável (JSON Lines, uma resolução por linha). É instrumentação, não
veredito — segue a disciplina do linter (nucleação, camadas, linhagem), mas fica
**fora do selo "completo para vereditos"** (não muda regra nenhuma).

### B. Harness do oráculo (cruza os dois repos)

Um crate/programa pequeno que, dado um workspace Rust:
1. roda o linter com `--emit` da resolução (caixa-preta, binário) → arestas-linter;
2. chama a lente como **biblioteca** (`lente_wiring::montar_grafo_workspace`) e lê
   `GrafoWorkspace.grafo` (`Aresta`/`Relation` públicos) → arestas-lente;
3. **projeta** os dois ao observável comum e remove diferenças legítimas de escopo
   (arestas intra-módulo, `std`/externas que um lado modela e o outro não — removidas
   **simetricamente**; arestas item-level da lente colapsadas a módulo/import);
4. **alinha** por chave normalizada (módulo-origem relativo ao crate + alvo) — este é
   o ponto difícil; normalize path↔caminho-de-módulo dos dois lados antes de casar;
5. **diff**, classificando cada discordância:
   - **linter `Unknown`, lente resolve concreto** → candidato a ponto cego do linter
     (o caso da anamnese). Sinal alto.
   - **alvos diferentes** (T_linter ≠ T_lente) → um dos dois erra.
   - **aresta num lado só** → macro/`cfg`/gerado/re-export/glob — uma forma que o
     modelo de um dos dois não representa. Triar (é o "ouro": candidato a forma nova).

### C. Prova-de-mordida do oráculo

O oráculo só vale se **morde**. Construir uma entrada onde os dois são **conhecidamente
divergentes** — p.ex. forçar no linter o classificador legado/cego (pré-0052) sobre um
import cross-crate, ou um caso sintético que a resolução textual não segue — e
confirmar que o oráculo **reporta** a discordância. Se reportar vazio numa entrada
sabidamente divergente, o oráculo é cego e não serve. (Mesma disciplina das fixtures,
aplicada ao oráculo.)

### D. Primeira corrida real + triagem

Rodar em ≥2 workspaces Rust **compiláveis e multi-crate**: o **próprio repo da lente**
(`tekt-cargo-dsm`) e ≥1 outro. Triar **cada** discordância em: ponto cego do linter /
bug da lente ou do fork (`uses_kind`, macro, `cfg`) / artefato de projeção. **Não
consertar** o tool subjacente aqui — cada ponto cego de linter achado vira um prompt
próprio (pode exigir capacidade de regra/classificação nova, que é trabalho à parte).

## Critérios de Verificação

- [ ] Pré-condição: lente produz grafo real num workspace real (senão, parada
      reportada com o débito dos 28 `#[ignore]` nomeado como pré-requisito).
- [ ] Linter emite resolução de todo import, **incluindo `Unknown`**, em formato
      parseável; é saída nova, fora do selo de veredito.
- [ ] Harness roda os dois, projeta ao observável comum (remoção simétrica de escopo),
      alinha e diffa — reprodutível por um comando.
- [ ] Comparação primária no nível de **aresta**; camada como contexto (modo-comum
      anotado).
- [ ] Oráculo **morde**: numa entrada conhecidamente divergente, reporta a
      discordância (demonstrado).
- [ ] Corrida em ≥2 workspaces (a lente + ≥1); **toda** discordância triada em uma de
      três naturezas, com a evidência.
- [ ] Pontos cegos achados ficam **listados como achados**, não consertados aqui;
      cada um aponta para um prompt seguinte.
- [ ] Laudo ao fim; self-lint do linter = 0; nada mascarado.

## Fora de escopo (prompts seguintes)

- **Consertar** cada ponto cego de linter que o oráculo achar (um prompt por forma).
- **Corpus de projetos reais variados** — escalar este oráculo a muitos workspaces
  (single/multi-crate, glob, camada em irmãos, gerado, macros, re-exports, `tests/`).
  Este oráculo é o motor; o corpus é a escala.
- **Contador de `Layer::Unknown`** em alvo real (detector mais barato, mesmo modo de
  falha por outro ângulo: pico de `Unknown` = categoria faltante).
- **Oráculo de posição** (linha:coluna / `PARSE`) e **severidade** — trilha à parte.
- **Merge com o `master` público** (multi-linguagem ⊕ conserto do 0052).

## Disciplina

A saída de resolução do linter segue nucleação/camadas/linhagem do repo; o oráculo
é prova-de-mordida (tem de morder antes de valer); discordâncias triadas da fonte,
uma a uma; nada mascarado; laudo ao fim.
