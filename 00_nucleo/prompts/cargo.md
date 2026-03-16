# Prompt: Manifest Dependencies (cargo)

**Camada**: L0 (Root / Build)
**Criado em**: 2026-03-14 (ADR-0005)
**Revisado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - Cargo.toml

---

## Contexto

`Cargo.toml` é o contrato de dependências do `crystalline-lint`.
A introdução de `rayon` pelo ADR-0004 sem prompt correspondente
demonstrou que decisões de dependência têm impacto arquitetural
direto e precisam ser governadas por L0.

A partir deste prompt, qualquer nova dependência requer:
1. Entrada na tabela de Dependências Homologadas com justificativa
2. Aprovação humana do prompt antes de modificar `Cargo.toml`
3. Revisão do Histórico de Revisões deste prompt

---

## Dependências Homologadas

### Runtime

| Crate | Versão | Camada | Justificativa |
|-------|--------|--------|---------------|
| `tree-sitter` | `~0.22` | L3 | Motor de parsing AST agnóstico — ADR-0001 |
| `tree-sitter-rust` | `~0.21` | L3 | Grammar Rust — v1 da estratégia multi-linguagem |
| `walkdir` | `~2` | L3 | Descoberta eficiente de arquivos com filtro de diretórios |
| `sha2` | `~0.10` | L3 | SHA256 para V5 e --fix-hashes |
| `hex` | `~0.4` | L3 | Encoding hexadecimal do SHA256[0..8] |
| `serde` | `~1` features: `derive` | L2, L3 | Serialização de structs — features mínimas |
| `serde_json` | `~1` | L2, L3 | Formatação SARIF e leitura de snapshots V6 |
| `toml` | `~0.8` | L3 | Leitura de `crystalline.toml` incluindo [excluded], [l1_ports], [orphan_exceptions] |
| `clap` | `~4` features: `derive` | L2 | CLI — parsing de flags V0–V9 e subcomandos |
| `colored` | `~2` | L2 | Output colorido no formato text |
| `rayon` | `~1` | L4 | Map-Reduce paralelo — ADR-0004, ADR-0006 |

### Dev (apenas testes)

| Crate | Versão | Justificativa |
|-------|--------|---------------|
| `tempfile` | `~3` | Fixtures de disco em testes de L3 |

---

## Restrições

**Zero dependências em L1:**
Nenhuma crate desta tabela pode ser importada em `01_core/`.
L1 usa apenas `std`. O compilador Rust enforce isso via ausência
de `use` de crates externas em L1.

**Features mínimas:**
Crates com feature flags (`serde`, `clap`) devem declarar apenas
as features estritamente necessárias. Features adicionais requerem
revisão deste prompt.

**Rayon restrito a L4:**
`rayon` importado exclusivamente em `04_wiring/main.rs`.
Nenhum arquivo de L1, L2 ou L3 importa `rayon` diretamente.
O paralelismo é uma decisão de orquestração — não de domínio
ou infraestrutura.

**Lockfile comitado:**
`Cargo.lock` é comitado e rastreado. Mudanças em `Cargo.lock`
sem mudança correspondente neste prompt são suspeitas e devem
ser investigadas antes de merge.

**Proibições explícitas:**

| Categoria | Motivo |
|-----------|--------|
| `tokio`, `async-std` | Async desnecessário para CLI batch — rayon é suficiente |
| `reqwest`, `hyper` | Rede apenas via trait em L3 |
| `diesel`, `sqlx` | Sem persistência no linter |
| `proc-macro` crates em L1 | Macros não cruzam fronteira de pureza |

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
Então nenhum usa crate fora de std
— compilador Rust enforce sem necessidade de regra adicional

Dado qualquer arquivo fora de 04_wiring/main.rs
Quando inspecionado por imports de rayon
Então nenhum contém use rayon — restrito a L4

Dado serde declarado no Cargo.toml
Quando features forem verificadas
Então contém apenas "derive" — sem features adicionais

Dado Cargo.lock modificado sem mudança correspondente neste prompt
Quando revisado em PR
Então deve ser investigado antes de merge

Dado nova crate sendo adicionada ao projeto
Quando avaliada contra este prompt
Então requer entrada na tabela + revisão do histórico
antes de modificar Cargo.toml
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0005) — nucleação retroativa do Cargo.toml. rayon adicionado via ADR-0004 documentado aqui | Cargo.toml |
| 2026-03-14 | ADR-0006: sem novas dependências — rayon já cobre Map-Reduce. toml justificativa atualizada para incluir [excluded], [l1_ports], [orphan_exceptions] | Cargo.toml |
