# Prompt: Registro de membros do workspace e classificação de import ciente de dependências
Hash do Código: e53ee416

**Camada**: L3 (infra — I/O sobre `Cargo.toml`)
**Materializa**: `03_infra/crate_registry.rs` + alterações em `03_infra/rs_parser.rs`
**Origem**: 0052 (conserto da opção 3, premissa confirmada em 0051)
**ADRs**: ADR-0009 (parsers por linguagem, `ImportKind` semântico), ADR-0005 (sem `Box::leak`)

---

## Contexto

O `resolve_layer` (`03_infra/rs_parser.rs`) só resolvia `crate::`/`super::`;
todo `use lente_*::` cross-crate virava `Layer::Unknown`. Consequência (confirmada
em 0051): o **V3** (`forbidden_import.rs`) é cego a direção **entre crates**
(`if target == Unknown { return false }`), e o **V14** dispara falso no `Kind`
(`use EnumLocal::*` lido como pacote externo).

A raiz é a suposição "Unknown = crate externo". Vale em um-crate-por-camada;
quebra no multi-crate. **O conserto é a montante**: ensinar o `resolve_layer` as
dependências reais, para o `target_layer` vir certo. **As regras L1 (V3, V14, V9)
NÃO mudam** — elas já agem sobre `target_layer`/`target_subdir`.

---

## O que materializar

### 1. `CrateRegistry` (L3) — `03_infra/crate_registry.rs`

Lê o workspace do projeto-alvo (I/O em `Cargo.toml`) e produz:

- **Membros**: para cada crate-membro — nome do pacote (normalizado
  `-`→`_`), diretório, camada (via `resolve_file_layer` sobre o diretório,
  reutilizando a mesma lógica `[layers]` do walker), e o conjunto de
  **dependências declaradas** (`[dependencies]` **e** `[dev-dependencies]`,
  normalizadas `-`→`_`).
- Enumeração: `[workspace].members` (com globs `crates/*`) quando há workspace;
  senão um `[package]` único = um membro na raiz.
- `member_layer(name) -> Option<Layer>` — lookup first-party por nome de pacote.
- `owner_of(file_path) -> Option<&MemberCrate>` — o membro cujo diretório é
  prefixo do ficheiro (mais longo vence). Dá o **contexto per-crate** de deps.
- `empty()` / `Default` — registro vazio ⇒ comportamento idêntico ao legado.

Sem `Box::leak`. Strings owned (`String`, `PathBuf`) — o registro é construído uma
vez e não participa do zero-copy do `SourceFile`.

### 2. Classificação ciente em `resolve_layer`/`resolve_subdir` (L3)

`resolve_layer` passa a receber o `registry` + o `owner` do ficheiro. A ordem
(per-crate — `owner` é o crate dono do ficheiro que faz o import):

1. `crate::`/`super::` → `module_layer(seg[1])` (inalterado).
2. `std`/`core`/`alloc` → `Unknown` (preservado; V14 isenta stdlib; V4 cuida de I/O).
3. 1º segmento == **nome do próprio crate dono** → intra-crate: `module_layer(seg[1])`
   (equivalente a `crate::`; cobre o self-import por nome `crystalline_lint::…`).
4. 1º segmento ∈ **outro membro** (registry) → camada do membro (first-party
   cross-crate; **V3 volta a enxergar a direção**).
5. 1º segmento ∈ **deps externas declaradas do owner** → `Unknown` (externo real;
   V14 aplica no L1, como hoje).
6. `owner` ausente → `Unknown` (preserva o legado para ficheiros sem dono).
7. owner presente, segmento não é membro nem dep nem stdlib → **item local**
   (ex.: `use EnumLocal::*`): **não classificar — não emitir Import**. O falso
   positivo do `Kind` some sem tocar o V14.

`resolve_subdir` (V9): quando o `target_layer` resolvido é L1 (intra OU membro
first-party L1), retorna `seg[1]` (o subdir após o qualificador de crate). Assim o
V9 passa a checar portas de L1 **entre crates**.

### 3. Emissão condicional (L3, `collect_imports`)

Item local (passo 7) ⇒ **não** empurra `Import`. Demais casos empurram com o
`target_layer`/`target_subdir` resolvidos.

---

## Restrições

- **Regras L1 (V3, V14, V9) NÃO mudam.** Se a tentação for editar uma regra L1,
  PARAR e reportar — o diagnóstico estaria incompleto.
- **Sem regressão**: `crate::`/`super::`, `std`, e o **linter passando em si
  mesmo** (0 violações). Registro vazio ⇒ comportamento legado bit-a-bit.
- `dev-dependencies` contam (imports first-party em teste).
- Distinguir membro real (pacote com `Cargo.toml`) por **nome do pacote**, não do
  diretório.
- Sem `Box::leak` (ADR-0005). `Mutex` só se precisar de `Sync` — o registro é
  imutável após construção, então `&self` partilhado basta no pipeline rayon.

---

## Critérios de Verificação

```
Dado um arquivo L3 com `use lente_<l4>::X` (membro L4)
Quando classificado com o registro
Então target_layer = L4 → V3 dispara (caso 1)

Dado um arquivo L1 com `use lente_core::X` (membro L1)
Então target_layer = L1 → V3 calado e V14 calado (caso 2)

Dado um arquivo L1 com `use serde::X` (dep externa declarada)
Então target_layer = Unknown → V14 dispara (caso 3)

Dado um arquivo L1 com `use EnumLocal::*` (não-dependência)
Então nenhum Import emitido → V14 calado (caso 4 — falso positivo some)

Dado um arquivo L1 com `use std::collections::HashMap`
Então target_layer = Unknown isento no V14; I/O segue no V4 (caso 5)

Dado `use crate::shell::X` (shell→L2) num L1
Então module_layer → L2 → V3 dispara (caso 6 — o controle do 0051)

Dado um arquivo L2 com `use lente_core::internal::X` (membro L1, subdir não-porta)
Então target_subdir = "internal" → V9 dispara (cross-crate)

Dado o registro vazio
Então a classificação é idêntica ao legado (todos os testes existentes verdes)

Dado crystalline-lint . no próprio linter
Então 0 violações
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-06-06 | Materialização do 0052. Registro membro→camada + deps (L3) e classificação de import ciente de dependências a montante no `resolve_layer`/`resolve_subdir`; emissão condicional para o item local (caso `Kind`). Decisões: contexto **per-crate** (owner via `owner_of`, deps+dev-deps por membro) e **V9 incluído** (subdir cross-crate). Regras L1 (V3/V14/V9) inalteradas. Registro vazio ⇒ legado bit-a-bit. | `03_infra/crate_registry.rs` (novo), `03_infra/rs_parser.rs` (resolve_layer/resolve_subdir/collect_imports/parse), `03_infra/mod.rs`, `04_wiring/main.rs` |
