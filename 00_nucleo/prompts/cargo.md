# Prompt: Manifest Dependencies (cargo)

**Camada**: L0 (Root / Build)
**Criado em**: 2026-03-14 (ADR-0005)
**Arquivos gerados**:
  - Cargo.toml

---

## Contexto

`Cargo.toml` é o contrato de dependências do `crystalline-lint`.
A introdução de `rayon` pelo ADR-0004 sem prompt correspondente
demonstrou que decisões de dependência têm impacto arquitetural
direto e precisam ser governadas por L0 como qualquer outro
componente.

A partir deste prompt, qualquer nova dependência requer:
1. Entrada na tabela de Dependências Homologadas com justificativa
2. Aprovação humana do prompt antes de modificar `Cargo.toml`
3. Revisão do Histórico de Revisões deste prompt

---

## Dependências Homologadas

### Runtime

| Crate | Versão | Camada | Justificativa |
|-------|--------|--------|---------------|
| `tree-sitter` | `~0.22` | L3 | Motor de parsing AST agnóstico — base do ADR-0001 |
| `tree-sitter-rust` | `~0.21` | L3 | Grammar Rust para tree-sitter — v1 da estratégia multi-linguagem |
| `walkdir` | `~2` | L3 | Descoberta eficiente de arquivos com filtro de diretórios |
| `sha2` | `~0.10` | L3 | SHA256 para drift detection (V5) e fix-hashes |
| `hex` | `~0.4` | L3 | Encoding hexadecimal do SHA256[0..8] |
| `serde` | `~1` features: `derive` | L2, L3 | Serialização de structs — features mínimas |
| `serde_json` | `~1` | L2, L3 | Formatação SARIF e leitura de snapshots V6 |
| `toml` | `~0.8` | L3 | Leitura de `crystalline.toml` |
| `clap` | `~4` features: `derive` | L2 | CLI — parsing de flags e validação de combinações |
| `colored` | `~2` | L2 | Output colorido no formato text |
| `rayon` | `~1` | L4 | Paralelismo de dados — ADR-0004. Restrito a `main.rs` |

### Dev (apenas testes)

| Crate | Versão | Justificativa |
|-------|--------|---------------|
| `tempfile` | `~3` | Fixtures de disco em testes de L3 |

---

## Restrições

**Zero dependências em L1:**
Nenhuma crate desta tabela pode ser importada em `01_core/`.
L1 usa apenas `std` e tipos definidos internamente. O compilador
Rust enforce isso via ausência de `use` de crates externas em L1.

**Features mínimas:**
Crates com feature flags (`serde`, `clap`) devem declarar apenas
as features estritamente necessárias. Features adicionais requerem
revisão deste prompt.

**Rayon restrito a L4:**
`rayon` é importado exclusivamente em `04_wiring/main.rs`.
Nenhum arquivo de L1, L2 ou L3 pode importar `rayon` diretamente.
O paralelismo é uma decisão de orquestração — não de domínio
ou infraestrutura.

**Lockfile comitado:**
`Cargo.lock` é comitado e rastreado. Mudanças em `Cargo.lock`
sem mudança correspondente neste prompt são suspeitas e devem
ser investigadas antes de merge.

**Proibições explícitas:**
As seguintes categorias de crates são proibidas sem ADR específico:

| Categoria | Motivo |
|-----------|--------|
| `tokio`, `async-std` | Async desnecessário para CLI batch — rayon é suficiente |
| `reqwest`, `hyper` | L1 deve ser puro — rede só via trait em L3 |
| `diesel`, `sqlx` | Sem persistência no linter |
| `proc-macro` crates em L1 | Macros não podem cruzar a fronteira de pureza |

---

## Formato do Cargo.toml
```toml
[package]
name = "crystalline-lint"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "Crystalline Architecture Linter"
repository = "https://github.com/Dikluwe/tekt-linter"
license = "MIT"

[lib]
path = "lib.rs"

[[bin]]
name = "crystalline-lint"
path = "04_wiring/main.rs"

[dependencies]
tree-sitter       = "~0.22"
tree-sitter-rust  = "~0.21"
walkdir           = "~2"
sha2              = "~0.10"
hex               = "~0.4"
serde             = { version = "~1", features = ["derive"] }
serde_json        = "~1"
toml              = "~0.8"
clap              = { version = "~4", features = ["derive"] }
colored           = "~2"
rayon             = "~1"

[dev-dependencies]
tempfile = "~3"
```

---

## Critérios de Verificação
```
Dado o Cargo.toml atual
Quando comparado com a tabela de Dependências Homologadas
Então não existem crates não listadas

Dado qualquer arquivo em 01_core/
Quando inspecionado por imports externos
Então nenhum usa crate fora de std — compilador enforce

Dado qualquer arquivo em 01_core/, 02_shell/ ou 03_infra/
Quando inspecionado por imports de rayon
Então nenhum contém use rayon — restrito a 04_wiring/main.rs

Dado serde declarado no Cargo.toml
Quando features forem verificadas
Então contém apenas "derive" — sem features adicionais

Dado nova crate sendo adicionada ao projeto
Quando avaliada contra este prompt
Então requer entrada na tabela + revisão do histórico
antes de modificar Cargo.toml
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial — nucleação retroativa do Cargo.toml. rayon adicionado via ADR-0004 documentado aqui | Cargo.toml |
```

---

Ordem de materialização para o ADR-0005:
```
1. violation-types.md (revisado)  → parsed_file.rs, violation.rs
2. cargo.md (novo)                → Cargo.toml
3. linter-core.md (já aprovado)   → main.rs sem Box::leak()
```

O comando para o Claude Code:
```
Leia 00_nucleo/prompts/violation-types.md (revisado ADR-0005),
00_nucleo/prompts/cargo.md (novo) e 00_nucleo/prompts/linter-core.md.
Materialize na ordem:
1. violation.rs — Location com Cow<'a, Path>
2. parsed_file.rs — confirmar que Token.symbol é Cow<'a, str>
3. Cargo.toml — conforme formato declarado em cargo.md
4. main.rs — reescrever conversores sem Box::leak(),
   usando Cow::Owned(path) em source_error_to_violation
   e parse_error_to_violation
Após cada arquivo, atualizar @prompt-hash no header.
