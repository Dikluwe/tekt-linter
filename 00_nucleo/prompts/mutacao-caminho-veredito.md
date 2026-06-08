# Prompt: estender a mutação ao caminho do veredito (fechar a higiene)

> Numere em sequência (provável 0057) e salve em `00_nucleo/prompts/`.
> Alvo: o repo do **linter** (clone com o conserto do 0052).
> Continuação de 0054–0056. Último passo da rede de regressão (para dentro); depois
> dele o selo "completo para vereditos" vale sem ressalva.

## Contexto

A mutação de 0054–0056 rodou sobre `01_core/rules/*.rs`, `03_infra/rs_parser.rs` e
`03_infra/crate_registry.rs`. Mas o veredito que o harness afirma — o multiset de
`ruleId` no SARIF — também é produzido por config, descoberta de arquivos, leitura
de prompt, despacho e emissão. Esses arquivos **nunca estiveram sob mutação**:
completude-para-veredito sobre eles é **não-medida**. As fixtures os exercitam em
caixa-preta (o verde delas já depende deles), mas isso não é o mesmo que provar que
todo mutante que muda veredito é morto. Este prompt fecha exatamente essa lacuna.

## Pré-condição

Clone do 0052; corpo 0054–0056 verde (≥38 fixtures), self-lint = 0,
`01_core/rules` + `rs_parser` + `crate_registry` com 0 sobreviventes que mudam
veredito (os 38 restantes do `rs_parser` já triados: 8 inerte + 30 fora-do-oráculo).

## Escopo — confirmar na fonte, não na minha lista

Pôr sob mutação **todo arquivo no caminho lint→veredito**. Candidatos (lidos de um
clone; **confirme cada papel rastreando o que alimenta uma regra** neste clone, e
inclua o que eu tiver perdido):

- `03_infra/config.rs` — lê `crystalline.toml`: `excluded`, `allow_adapter_structs`,
  `l1_ports`, `layers`, `members`, `module_layers`. Decide *qual* regra dispara e
  *em quais* arquivos.
- `03_infra/walker.rs`, `03_infra/prompt_walker.rs` — *quais* arquivos são lintados.
- `03_infra/prompt_reader.rs`, `03_infra/prompt_snapshot_reader.rs` (e qualquer util
  de hash que o V5 **leia** — não o `hash_writer`, que é do caminho de *fix*) —
  alimentam V1/V5/V6/V7.
- `04_wiring/main.rs` — despacha as regras e coleta/dedup as violações.
- `02_shell/cli.rs` — emite o SARIF que o harness lê.

**Fora do escopo (mantém o selo finito e honesto):** os parsers de outras
linguagens (`c/cpp/py/ts/zig` — o corpo é Rust) e o caminho de *fix/update*
(`hash_writer`, `snapshot_writer`, `fix_hashes`, `update_snapshot`). O selo vale
para **lint de Rust**, que é o que o corpo cobre — diga isso no laudo.

## Tarefa

1. Rodar a mutação no escopo confirmado:
   `cargo mutants -j 4 --file 03_infra/config.rs --file 03_infra/walker.rs --file 03_infra/prompt_walker.rs --file 03_infra/prompt_reader.rs --file 03_infra/prompt_snapshot_reader.rs --file 04_wiring/main.rs --file 02_shell/cli.rs`
   (ajuste a lista ao que o rastreio confirmar). Ler os sobreviventes de
   `mutants.out/missed.txt` — fonte autoritativa, não a contagem de cabeça.
2. Classificar **cada** sobrevivente pela taxonomia do 0056, exata:
   - **Muda veredito** (qual regra dispara / contagem de IDs) → **matar com fixture
     bite-proof**, re-rodar, confirmar morto.
   - **Fora-do-oráculo** → muda só posição reportada. Itemizar (linha/função).
   - **Inerte** → saída que nenhuma regra lê, ou código morto. Itemizar com prova.
   Soma das três = total da ferramenta, sem buraco.
3. Os "muda-veredito" esperados aqui são, na maioria, **botões de config e bordas
   de walker que nenhuma fixture varia** — e cada um vira uma fixture nova:
   - `allow_adapter_structs = false` → `struct` em L4 passa a disparar V12 (par com
     o default `true` já existente). Mata o mutante do default.
   - `l1_ports` ausente vs presente → muda V9. Fixture que varia a porta.
   - `[excluded]`: um dir com uma violação real, **excluído** → 0 violações; o mesmo
     conteúdo **não-excluído** → a violação aparece. Mata os mutantes de
     walker/exclusão.
   - recursão do walker: uma violação num **subdiretório aninhado** tem de ser
     achada → mata o mutante que para a recursão.
   - despacho (`main.rs`) e emissão (`cli.rs`): os pares por-regra do 0054 já matam
     "regra não despachada"; para dedup/ordem, a fixture do par `[V3, V10]` ajuda —
     some sobrevivente que reste com fixture de **múltiplas violações no mesmo
     arquivo** (afirma o multiset, não só presença).
4. Iterar até **0 sobreviventes que mudam veredito** no escopo.

## Critérios de Verificação

- [ ] Pré-condição confirmada; escopo confirmado por rastreio (não pela minha lista).
- [ ] `missed.txt` lido; total = `caught + missed + unviable`.
- [ ] Cada sobrevivente em exatamente uma natureza (veredito-morto / fora-do-oráculo
      / inerte); soma exata.
- [ ] Fixtures novas para os botões de config e bordas de walker (≥ as cinco acima),
      cada uma bite-proof, harness afirmando IDs + contagem.
- [ ] 0 sobreviventes que mudam veredito em config/walker/prompt-IO/despacho/SARIF.
- [ ] Laudo registra o que ficou **fora** (multi-linguagem, fix/update) e que o selo
      é **"lint de Rust"**.
- [ ] Atualizar 0056: o selo "completo para vereditos" passa de
      *{regras + classificação}* para *todo o caminho lint→veredito de Rust*, **sem
      ressalva**, com ponteiro para este laudo.
- [ ] Self-lint = 0; suíte verde fora do `blanket_impl` pré-existente; nada mascarado.

## Fora de escopo — a virada para fora (prompts seguintes)

Fechada a higiene, o propósito puxa para fora (verificar arquitetura real):
1. **Oráculo diferencial contra a lente** — a lente computa a estrutura de
   dependências de um workspace; o linter classifica imports em camadas. Nas mesmas
   entradas têm de concordar; discordância = ponto cego de um dos dois numa
   arquitetura real. É o teste mais forte porque é uma segunda computação
   independente, que não compartilha os nossos pontos cegos.
2. **Corpus de projetos reais variados** — single/multi-crate, workspace com glob,
   camada em irmãos, código gerado, macros, re-exports, layout de `tests/`.
3. **Contador de `Layer::Unknown`** em alvo real — pico = categoria faltante.

E, à parte: a decisão de **merge com o `master` público** (multi-linguagem + Hash
Locking ⊕ conserto do 0052).

## Disciplina (do repo)

Escopo confirmado da fonte; sobreviventes da `missed.txt`, um a um; muda-veredito
morto com prova-de-mordida; nada de rótulo agregado; laudo ao fim e correção do 0056
in loco com nota no histórico.
