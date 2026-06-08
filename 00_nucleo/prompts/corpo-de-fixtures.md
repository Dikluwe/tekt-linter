# Prompt: corpo de fixtures bite-proof + completude por mutação (solidificar o motor de regras)

> Numere e salve em `00_nucleo/prompts/` seguindo a convenção do repo do `tekt-linter`.
> Alvo: o repo do **linter**, não o `tekt-cargo-dsm`.

## Contexto

O critério atual de confiança no linter é "`crystalline-lint .` passa em si mesmo
= 0 violações". Isso só exercita a forma que o próprio linter tem: crate único,
multi-camada, todo import `crate::`/`super::`. A lógica cross-crate fica dormente
e não é testada por ele mesmo. Um verificador só exercita as formas que ele tem;
um "0 violações" só vale para as invariantes que o modelo dele consegue
representar — e foi exatamente uma forma não representada (L1 multi-crate) que
expôs o falso negativo do V3 e o falso positivo do V14.

Princípio (do laudo 0052): uma fixture que falha só prova algo se ela **morde** —
se falha sob a regra deliberadamente errada. No 0052 isso foi feito à mão para 6
casos, injetando o classificador legado. O teste de mutação (`cargo-mutants`)
generaliza a prova-de-mordida para cada ramo do código: um mutante que sobrevive
é um ramo que nenhuma fixture exercita, ou seja, uma categoria não coberta.

Este prompt constrói o corpo de fixtures para as 14 regras e prova a completude
dele contra o motor de regras por mutação. É **completude contra as regras**
(finita, verificável), não contra a linguagem (essa fica para os prompts
seguintes).

## Pré-condição — resolver antes de escrever qualquer fixture

1. **Confirmar que este clone é o canônico** (o que tem o conserto do 0052):
   `03_infra/crate_registry.rs` existe, e `forbidden_import` (V3) enxerga
   cross-crate — não retorna `None` para um alvo de outro crate. Rode os testes
   do 0052 se existirem. **Se `crate_registry.rs` não existir, PARE e reporte**:
   a versão sem o conserto classifica o alvo cross-crate como `Unknown` e erra
   V3/V9/V14, então o corpo dessas três sairia errado.
2. **Reportar (não resolver agora) a divergência com o `master` público.** O
   `master` em `github.com/Dikluwe/tekt-linter` tem parsers de C/C++/Zig/Python e
   o "Hash Locking", e **não** tem o conserto do 0052; este clone tem o conserto e
   (provavelmente) não tem o multi-linguagem. As duas linhas divergiram, nenhuma é
   superconjunto da outra. Registre isso no laudo como **decisão de merge
   pendente, à parte**. Não bloqueia este prompt: das 14 regras, 11 são idênticas
   nas duas linhas; só V3/V9/V14 dependem do `classify_import` (que este clone tem
   certo). Construa o corpo contra este clone.

## Objetivo

Um corpo de fixtures que exercite, **por regra (V1–V14)**, no mínimo uma forma que
**passa** e uma que **falha**, cada uma com o conjunto de violações esperado
fixado, e a prova de que o corpo é completo contra o motor de regras: rodar
`cargo-mutants` sobre o código das regras e levar os mutantes sobreviventes a zero
(ou documentar cada sobrevivente como mutante-equivalente, com justificativa).

## Mapa de regras — a característica que cada uma lê define o que varia entre a forma-que-passa e a forma-que-falha

(extraído da fonte: `01_core/rules/*.rs`)

| Regra | Lê | Forma que PASSA | Forma que FALHA |
|---|---|---|---|
| V1 prompt_header | cabeçalho presente; prompt referenciado existe em `00_nucleo/` | cabeçalho correto + prompt existe | cabeçalho ausente **ou** prompt inexistente |
| V2 test_file | camada==L1; tem cobertura de teste | módulo L1 com teste | módulo L1 sem teste |
| **V3 forbidden_import** | camada-origem × camada-alvo (matriz de permissão) | import para baixo (ex. L2→L3) ou mesma camada | **inversão (L2→L4), CROSS-CRATE** |
| V4 impure_core | camada==L1; token de I/O proibido | L1 puro | L1 com I/O (ex. `std::fs`) |
| V5 prompt_drift | hash declarado no cabeçalho × hash calculado | hashes batem | hashes divergem |
| V6 prompt_stale | prompt desatualizado vs código | atual | stale |
| V7 orphan_prompt | prompt referenciado por algum arquivo | prompt com código que o cita | prompt órfão |
| V8 alien_file | diretório do arquivo mapeia para camada | arquivo em camada mapeada | arquivo fora de qualquer camada |
| **V9 pub_leak** | import alcança subdir de membro L1; subdir é porta declarada (`[l1_ports]`) | import por porta declarada | **import por subdir não-porta, CROSS-CRATE** |
| V10 quarantine_leak | alvo é `lab` | não importa `lab` | importa `lab` |
| V11 dangling_contract | contrato tem implementador | contrato implementado | contrato sem implementador |
| V12 wiring_logic_leak | item no L4 é `enum` (struct é permitido) | struct adaptador no L4 | `enum` declarado no L4 |
| V13 mutable_state_core | camada==L1; estado mutável | L1 sem estado mutável | L1 com `static mut`/global mutável |
| **V14 external_type_in_contract** | camada==L1; tipo é externo | L1 só com tipos locais/`std` | **L1 com dep externa real (ex. `serde`); distinguir de import first-party L1, que NÃO falha — CROSS-CRATE** |

As três em negrito (V3, V9, V14) leem a camada-alvo de um import: a forma-que-falha
delas exige um **workspace multi-crate** na fixture (não um arquivo solto), porque é
aí que a versão cega regredia. Para o V3, inclua o caso real já conhecido: a CLI L2
importando o wiring L4 (`use lente_wiring::AlvoBusca` etc.). Para o V14, a fixture
tem que provar a distinção que o 0052 trouxe: `use serde::…` (externo real) FALHA,
`use outro_crate_L1::…` (first-party mesma camada) PASSA.

## Tarefa

1. Criar `tests/fixtures/` (ou onde o repo já põe fixtures) com um subdiretório por
   caso: `vNN_pass/` e `vNN_fail/`. Cada caso é o **menor** workspace/arquivo que
   exercita a característica daquela linha do mapa, com seu `crystalline.toml`
   mínimo quando precisar de config (ex. `[l1_ports]` para V9).

   > **Restrições de montagem das fixtures multi-crate** (verificadas contra
   > `03_infra/crate_registry.rs` — `from_root`/`expand_member_pattern`; o registry
   > é construído de `cli.path`, então a fixture-alvo é resolvida sozinha):
   > - **Camada do membro vem do `[layers]` da própria fixture.** A camada de cada
   >   crate-membro é resolvida por `resolve_file_layer(dir, root, config)`, logo o
   >   `crystalline.toml` da fixture **tem de mapear os diretórios dos membros em
   >   `[layers]`** — senão resolvem para `Layer::Unknown` e a inversão L2→L4 do V3
   >   não dispara. (Para V9, isto é além do `[l1_ports]`.)
   > - **Glob de membros é só `*` final.** `expand_member_pattern` aceita path exato
   >   ou sufixo `*`/`/*` (ex. `crates/*`); **não** há `**` recursivo nem glob no
   >   meio do path. Use lista explícita ou `crates/*`.
   > - **Todo membro precisa de `[package].name`.** Membro sem nome é silenciosamente
   >   pulado. Workspace virtual na raiz (só `[workspace]`, sem `[package]`) é OK —
   >   os membros é que precisam de nome.
2. Escrever um harness de teste (`#[test]`) que roda o linter sobre cada fixture e
   **afirma o conjunto de violações esperado** (IDs e contagem), não só "passou".
   Esses testes são o que a mutação vai usar — então têm que ser específicos.
3. Instalar e rodar a mutação restrita ao motor de regras:
   `cargo mutants --file 01_core/rules/*.rs --file 03_infra/rs_parser.rs --file 03_infra/crate_registry.rs`
   (a resolução de camada-alvo vive em `classify_import` + `resolve_subdir` em
   `03_infra/rs_parser.rs` — não existe `resolve_layer` nomeado). Para cada **mutante
   sobrevivente**: ou adicione a fixture que o mata (a categoria que faltava), ou
   documente-o como equivalente com justificativa. Itere até **0 sobreviventes não
   documentados**.
4. Cobertura de ramos como conferência barata (`cargo-llvm-cov` ou equivalente):
   nenhum ramo do código de classificação/regra deve ficar sem execução pela
   suíte de fixtures.

## Critérios de Verificação

- [ ] Pré-condição confirmada (este é o clone com o 0052) **ou** parada reportada.
- [ ] Divergência com o `master` público registrada no laudo como merge pendente.
- [ ] Uma fixture-passa e uma fixture-falha por regra V1–V14, com veredito fixado.
- [ ] V3/V9/V14 com fixtures **multi-crate**; V3 inclui a inversão L2→L4 real; V14
      distingue externo real de first-party L1.
- [ ] Harness afirma IDs + contagem de violações, não só sucesso/fracasso.
- [ ] `cargo-mutants` sobre o motor de regras: **0 mutantes sobreviventes** não
      documentados; cada sobrevivente restante justificado como equivalente.
- [ ] Cobertura de ramos sem ramo morto no código de regra/classificação.
- [ ] O linter continua passando em si mesmo (`crystalline-lint .` = 0).
- [ ] Nada mascarado (sem recolocar whitelists para esconder caso).
- [ ] Laudo escrito na raiz de `00_nucleo/` com prefixo numérico (ex.
      `00NN-corpo_de_fixtures.md`), no formato dos laudos anteriores (`0051-…`, `0052-…`).

## Fora de escopo — prompts seguintes (não fazer agora)

- **Contador de "Desconhecida"**: instrumentar o linter para contar, num alvo real,
  quantos imports caem em `Layer::Unknown`. Um pico = categoria faltante (detecção
  contra a linguagem).
- **Oráculo diferencial**: cruzar a classificação de camadas do linter com a
  computação independente de dependências da própria lente (`tekt-cargo-dsm`);
  divergência = uma das duas trata mal uma categoria.
- **Corpus de projetos reais variados**: single-crate, multi-crate, workspace com
  glob, camada espalhada em irmãos, código gerado, macros, re-exports, layout de
  `tests/` — rodar o linter em todos e caçar veredito errado.

## Disciplina (do repo)

Prompt nucleado antes do código; cabeçalhos de linhagem nos arquivos novos; TDD com
prova-de-mordida (a mutação é a forma mecânica disso); suíte verde fora do débito
`blanket_impl` pré-existente; nada mascarado; laudo ao fim.
