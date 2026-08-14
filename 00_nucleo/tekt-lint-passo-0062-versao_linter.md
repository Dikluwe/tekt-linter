# Laudo 0062 — release `v0.2.0` do `crystalline-lint` (primeiro corte versionado)

**Onde roda**: o clone canônico (com o 0052 e os laudos 0054–0061). Ponto de parada.
**Criado em**: 2026-06-09
**Estado**: `IMPLEMENTADO`
**Prompt**: [`00_nucleo/prompts/versao-linter.md`](prompts/versao-linter.md)
**Natureza**: processo de release — não materializa Rust. Toca `Cargo.toml`
(versão), cria `CHANGELOG.md`, corta tag anotada. Os portões são **re-rodados**, não
herdados dos laudos.

---

## Passo 1 — divergência resolvida: **caso superconjunto**

Diffadas as duas linhas **na fonte** (git), não na memória do 0058:

- `git merge-base master origin/master` = `9fe884c` (o commit do 0059).
- `git rev-list --left-right --count master...origin/master` = **2 / 0**: o `master`
  local está 2 commits à frente (0060, 0061), e `origin/master` está 0 à frente.
- Logo `origin/master` é **ancestral directo** do `master` local e a história de
  `origin/master` **já contém**: `43e11a3` (Hash Locking / Dupla Paridade), os
  commits multi-linguagem (Zig `26edb80`, C/C++ `29160e2`, mais Python/TS), e
  `d6a0612` (0052) construído **em cima** deles.

**Conclusão**: a divergência "duas linhas" que o 0058 registrou **já estava
reconciliada** nesta linha — o 0052 assenta sobre a base multi-linguagem + Hash
Locking. Este clone é um **superconjunto estrito** (multi-linguagem + Hash Locking +
0052 + 0054–0061), confirmado por: parsers C/C++/Zig/Python/TS presentes e ligados no
despacho (`main.rs`), `Hash do Código` em 8 prompts, e o diff `origin/master→master`
só **adiciona** (0060/0061), não remove capacidade.

→ **Release limpo, sem ressalva só-Rust.** Não há regressão silenciosa: cortar esta
versão não perde nada que o estado anterior tinha.

## Passo 2 — versão: `0.1.0 → 0.2.0`

MINOR, pré-1.0. Desde o `0.1.0` houve mudança de comportamento notável (classificação
ciente de deps — passou a ver alias, dep renomeada e caminho fora do `use`) e recurso
novo (`check_test_imports`). **Não `1.0.0`**: seria overclaim — a completude **contra
a linguagem** não foi provada (a lista de cegos saiu de arquiteturas reais; a trilha
de descoberta segue aberta) e há residuais nomeados.

## Passo 3 — os 4 portões, RE-RODADOS no clone (binário `0.2.0` fresco)

| Portão | Resultado |
|--------|-----------|
| **Self-lint = 0** | `crystalline-lint .` → ✓ No violations found |
| **Suíte verde** | 492 unit + 58 fixtures, 0 falhas, 0 ignorados |
| **Selo de mutação** | 502 mutantes no caminho lint→veredito de Rust: 335 mortos, 55 inviáveis, **112 sobreviventes — 0 que mudam veredito** (triagem abaixo) |
| **Oráculo (default) na lente** | **0 cego-linter, 0 só-linter**, 21 acordo |

### Triagem do selo de mutação (o portão exige 0 *veredito-mudante*, não 0 sobrevivente)

Evidência decisiva: **nenhum sobrevivente na lógica de decisão das regras**
(`01_core/rules/*`) — toda mutação de `is_forbidden`, dos guards, das comparações de
`target_layer`, `package_name`/`is_allowed` foi **morta**. Os 112 sobreviventes caem
todos em categorias já reconciliadas (0054/0055/0056/0057/0060/0061), nenhuma muda o
multiset de IDs de regra:

- **Posição** (linha/coluna): `find_first_error_pos` (19), `row + 1` em
  collect_imports/collect_path_refs/collect_tokens/extract_declarations (~11).
- **ImportKind** (V3/V9/V14 ignoram `kind`; o import é emitido na mesma): 1.
- **Precisão de sub-caminho em `token_tree`** (parte B, residual nomeado do 0060): 6.
- **`parse_layer_tag`** (a camada do ficheiro é por path, não pelo header): 5.
- **Saída/formatação**: `cli.rs` (9 — `format_resolution`/`first_segment` do
  `--emit-resolution`; `layer_str`/`import_kind_str`/`sarif_rule` = texto/metadados do
  SARIF, não o `ruleId` dos resultados).
- **Severidade/arms de config**: `level_for` fatal/error (2 — severidade é
  fora-do-oráculo), `layer_for_module` `Some("L0")` (1 — nenhum módulo mapeia L0).
- **Equivalente**: `CrateRegistry::empty` ≡ `Default::default()` (1).
- **Orquestração/traversal** (0057-reconciliado): `main.rs` (34 — código de saída,
  ordenação, flags), `walker.rs` (18 — traversal), `prompt_reader.rs` (4).

O threading novo do `check_test_imports` em `main.rs` **é** selado — as fixtures
`vtest_*_on_fail`/`_default_pass` matam qualquer mutação de `unwrap_or(false)` (viraria
o default). → **Portão cumprido: 0 sobreviventes que mudam veredito.**

## Passo 4 — `CHANGELOG.md` (o primeiro)

Criado em linguagem de usuário, sintetizando 0052–0061: Added (classificação ciente de
deps; detecção de alias/rename/caminho-fora-do-`use`; `check_test_imports`), Changed
(V3/V9/V14 excluem `#[cfg(test)]` por padrão), Qualidade interna (fixtures bite-proof;
completude por mutação dos vereditos de Rust; oráculo diferencial), Limitações
conhecidas (precisão de sub-caminho em atributo/macro; corpos de macro; posição e
severidade fora do oráculo; modo `on` sem oráculo) e a **declaração de escopo**.

### Declaração de escopo (anti-overclaim)

O `0.2.0` é **completo para os vereditos de lint de Rust** (selado por mutação) e
**concorda com o oráculo independente da lente nas arquiteturas testadas**. **Não**
afirma completude contra todas as formas que o Rust permite — a lista de cegos saiu de
arquiteturas reais e a trilha de descoberta (corpus variado) está aberta.

## Passo 5 — tag

Tag **anotada** `v0.2.0` no commit do release, **após** os 4 portões passarem. A
mensagem aponta para o `CHANGELOG.md` e a declaração de escopo.

## Critérios de Verificação

- [x] Divergência diffada **na fonte**; caso **superconjunto** registrado (sem ressalva
      só-Rust — multi-linguagem e Hash Locking estão incluídos, não pendentes nesta linha).
- [x] Versão subida `0.1.0 → 0.2.0` (MINOR justificado; não-1.0 justificado).
- [x] Os 4 portões **re-rodados** e verdes (binário `0.2.0` fresco).
- [x] `CHANGELOG.md` criado em linguagem de usuário, com Added/Changed/Qualidade/
      Limitações e a declaração de escopo.
- [x] Tag anotada `v0.2.0` criada após os portões.
- [x] Laudo de release ao fim; nada overclaimed; nada mascarado.

## Fora de escopo (registrado)

- **Publicar em crates.io** — decisão separada; este corte é versionado e taggeado,
  não publica externamente.
- **Trilha de descoberta** (corpus de projetos variados) — responde se a lista de
  cegos é completa; retoma quando voltarmos ao linter.
- Oráculo de **posição/severidade**.

## Histórico de Revisões

- 2026-06-09 — Primeiro corte versionado: `0.2.0`. Divergência resolvida na fonte
  (superconjunto — `origin/master` é ancestral, multi-linguagem + Hash Locking + 0052
  já na linha). 4 portões re-rodados verdes (self-lint 0; 492+58; mutação 502/0
  veredito-mudante; oráculo 0/0). `CHANGELOG.md` criado; tag anotada `v0.2.0`.
