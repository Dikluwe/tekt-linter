# Prompt: cortar a versão do `crystalline-lint` (primeiro release)

> Numere em sequência (provável 0062) e salve em `00_nucleo/prompts/` do **linter**.
> Alvo: o clone canônico (com o 0052 e os laudos 0054–0061). Ponto de parada do linter.

## Contexto

O linter teve um avanço grande (0052–0061) e vamos parar de mexer nele por enquanto.
Esta é a **primeira versão de verdade**: o repo está em `0.1.0`, **sem CHANGELOG e
sem tags**. Então este prompt estabelece o release — número, portões, changelog,
escopo honesto, tag — não só um bump.

## Passo 1 — Resolver a divergência ANTES de escolher o escopo

Há duas linhas (registrado desde o 0058): o `master` público tem parsers C/C++/Zig/
Python e "Hash Locking" e **não** tem o conserto do 0052; este clone tem o 0052 e
todo o trabalho 0054–0061. **Diff as duas linhas** e determine, na fonte, o que este
clone tem:

- **Se este clone for superconjunto** (tem o multi-linguagem também) → release limpo.
- **Se este clone NÃO tiver o multi-linguagem** → cortar a versão dele **perderia**
  capacidade que existe no `0.1.0` público. Isso **não** pode virar regressão
  silenciosa. Escolha honesta: cortar a versão como a **linha de lint de Rust**, e as
  notas de release dizem, explícito, que multi-linguagem **não está incluído** e o
  merge está **pendente** — não fingir que esta versão supera tudo. O merge segue
  decisão à parte (fora deste prompt).

Registre no laudo qual caso se aplica.

## Passo 2 — Número da versão

Ler a versão atual no `Cargo.toml` do clone canônico e subir a partir dela.
**Recomendação: `0.2.0`** (MINOR, pré-1.0). Justificativa: desde o `0.1.0` houve
mudança de comportamento notável (classificação ciente de deps; passou a ver alias,
dep renomeada e referência de caminho fora do `use`) e recurso novo
(`check_test_imports`).

**Não `1.0.0`** — seria overclaim. O 1.0 sugere completude/estabilidade, e três coisas
estão abertas: a completude **contra a linguagem** não foi provada (a lista de cegos
saiu de uma arquitetura real; a trilha de descoberta está aberta), as duas linhas não
foram mescladas, e há residuais nomeados. A versão tem de afirmar só o que se provou.

## Passo 3 — Portões de release (RE-RODAR, não confiar nos laudos)

Um release re-verifica; não herda o verde dos laudos. Rodar agora, no clone:

- [ ] **Self-lint = 0**: `crystalline-lint .`.
- [ ] **Suíte verde**: unit + fixtures (era 492 + 58 no 0061) fora do `blanket_impl`
      pré-existente.
- [ ] **Selo de mutação**: 0 sobreviventes que **mudam veredito** em todo o caminho
      lint→veredito de Rust (regras + `classify_import`/`crate_registry` + config +
      walker + prompt-IO + despacho + SARIF). Re-rodar o escopo dos 0054/0057/0061.
- [ ] **Oráculo (modo default)** na lente: **0 cego-linter, 0 só-linter**.

Se **qualquer** portão falhar: **parar e reportar**. Não se tagueia um estado que
falha um portão.

## Passo 4 — `CHANGELOG.md` (o primeiro), em linguagem de usuário

Sintetizar 0052–0061 (não jargão de laudo). Sugestão de seções para `0.2.0`:

- **Added**: classificação de import ciente de dependências (gravidade cross-crate
  agora vista); detecção de referência cross-crate por `use` com alias, por
  dependência renomeada, e por caminho qualificado fora do `use` (expressão, tipo,
  atributo); opção `check_test_imports` (default `false`).
- **Changed**: V3/V9/V14 passam a **excluir `#[cfg(test)]` por padrão** (gravidade é
  sobre o grafo de produção).
- **Qualidade interna** (sem impacto de API): corpo de fixtures bite-proof por regra;
  completude por mutação para **vereditos de lint de Rust** em todo o caminho do
  veredito; oráculo diferencial contra a lente (`tekt-cargo-dsm`).
- **Limitações conhecidas** (não esconder): precisão de sub-caminho para referências
  dentro de atributo/macro `token_tree` (afeta o subdir do V9 a partir de atributo);
  caminhos dentro de corpos de macro não-estruturados; **posição e severidade** não
  estão sob o oráculo de veredito; **multi-linguagem e Hash Locking** são linha
  separada, não mesclada.

### A declaração de escopo (o ponto anti-overclaim)

As notas têm de afirmar **só o que se provou**, nestes termos: o `0.2.0` é **completo
para vereditos de lint de Rust** (selado por mutação) e **concorda com o oráculo
independente da lente nas arquiteturas testadas**. **Não** afirma completude contra
todas as formas que o Rust permite — a lista de cegos saiu de uma arquitetura real e
a trilha de descoberta (corpus de projetos variados) está aberta. Dizer mais que isso
repete o modo de falha da anamnese numa nota de release.

## Passo 5 — Tag

Tag **anotada** `v0.2.0` (ou o número escolhido) no commit do release, **depois** dos
portões passarem. Mensagem da tag aponta para o `CHANGELOG.md` e a declaração de escopo.

## Critérios de Verificação

- [ ] Divergência diffada; caso (superconjunto / só-Rust) registrado; se só-Rust, as
      notas dizem que multi-linguagem não entra e o merge é pendente.
- [ ] Versão subida (recomendado `0.2.0`), com a justificativa do MINOR e do não-1.0.
- [ ] Os 4 portões **re-rodados** e verdes (ou parada reportada).
- [ ] `CHANGELOG.md` criado em linguagem de usuário, com Added/Changed/Qualidade/
      Limitações e a **declaração de escopo** (completo-para-vereditos-de-Rust +
      concorda-com-a-lente; não completude-contra-a-linguagem).
- [ ] Tag anotada criada após os portões.
- [ ] Laudo de release ao fim (versão, resultado dos portões, caso da divergência,
      tag); nada overclaimed; nada mascarado.

## Fora de escopo (decisões/trilhas à parte)

- **Publicar em crates.io** — decisão separada, e prematura com a divergência aberta e
  a completude não provada. Este prompt corta um release **versionado e taggeado**, não
  publica externamente.
- **Merge com o `master` público** (multi-linguagem ⊕ 0052).
- **Trilha de descoberta** — corpus de projetos reais variados, para responder se a
  lista de cegos é completa. É o trabalho que retoma quando voltarmos ao linter.
- Oráculo de **posição/severidade**.

## Disciplina

Portões re-rodados (não herdados); escopo afirmado só até onde se provou; divergência
sem regressão silenciosa; release laudado; nada overclaimed.
