# Prompt: Rust Parser Implementation (rs-parser)

**Camada**: L4 (Wiring — artefato gerido)
**Criado em**: 2026-03-14 (ADR-0005)
**Revisado em**: 2026-03-18 (ADR-0009: tree-sitter-typescript, tree-sitter-python)
**Arquivos gerados**:
  - Cargo.toml

---

## Contexto

`Cargo.toml` pertence a L4 — é o ponto de composição de todas as
dependências externas do sistema. Qualquer mudança em dependências
requer revisão deste prompt antes de modificar o arquivo.

Este prompt declara todas as dependências, suas versões e a
justificativa arquitetural para cada uma.

---

## Dependências

```toml
[dependencies]
# ── L3 — Infra ────────────────────────────────────────────────────────────────

# Parsers tree-sitter por linguagem (ADR-0001, ADR-0009)
# Cada grammar é isolada — adicionar uma nova linguagem requer apenas
# adicionar a crate correspondente aqui e criar parsers/<lang>.md
tree-sitter       = "0.22"   # engine principal — agnóstico de linguagem
tree-sitter-rust  = "0.21"   # grammar Rust (parsers/rust.md)
tree-sitter-typescript = "0.21"  # grammar TypeScript (parsers/typescript.md)
tree-sitter-python = "0.21"  # grammar Python (parsers/python.md)

# Filesystem e descoberta
walkdir = "2"                # varredura de directórios (FileWalker)

# Hashing
sha2 = "0.10"               # SHA256[0..8] para @prompt-hash (PromptReader)
hex  = "0.4"                # codificação hex do hash

# Serialização
serde      = { version = "1", features = ["derive"] }  # derive Serialize/Deserialize
serde_json = "1"            # JSON para SARIF e Interface Snapshot
toml       = "0.8"          # parsing de crystalline.toml

# ── L2 — Shell ────────────────────────────────────────────────────────────────
clap    = { version = "4", features = ["derive"] }  # CLI (sarif-formatter.md)
colored = "2"               # output colorido no terminal

# ── L4 — Wiring ───────────────────────────────────────────────────────────────
rayon = "1"                 # paralelismo Map-Reduce (ADR-0004)

[dev-dependencies]
tempfile = "3"              # ficheiros temporários em testes de L3
```

---

## Justificativas

| Crate | Camada | Motivo |
|-------|--------|--------|
| `tree-sitter` | L3 | Engine de parse incremental multi-linguagem (ADR-0001) |
| `tree-sitter-rust` | L3 | Grammar Rust — `RustParser` (parsers/rust.md) |
| `tree-sitter-typescript` | L3 | Grammar TypeScript — `TsParser` (parsers/typescript.md) |
| `tree-sitter-python` | L3 | Grammar Python — `PyParser` (parsers/python.md) |
| `walkdir` | L3 | Varredura recursiva de directórios sem reimplementação |
| `sha2` | L3 | SHA256 para `@prompt-hash` — determinístico e verificável |
| `hex` | L3 | Codificação hex dos primeiros 8 bytes do hash |
| `serde` + `serde_json` | L3 | Serialização de Interface Snapshot (V6) e output SARIF |
| `toml` | L3 | Parsing de `crystalline.toml` sem dependência de serde_json |
| `clap` | L2 | CLI declarativa com derive — evita boilerplate de argparse |
| `colored` | L2 | Output legível no terminal sem ANSI manual |
| `rayon` | L4 | Map-Reduce paralelo — confinado a L4 (ADR-0004) |
| `tempfile` | dev | Testes de L3 que escrevem para disco sem poluir o sistema |

## Restrições

- `rayon` é dependência de L4 exclusivamente — nunca exposto a L1, L2 ou L3
- `serde_json` é dependência de L3 — nunca exposto a L1
- `tree-sitter` e as grammars são dependências de L3 — nunca expostas a L1
- Adicionar suporte a uma nova linguagem requer apenas adicionar
  `tree-sitter-<lang>` aqui e criar `parsers/<lang>.md` —
  zero alterações nos prompts universais

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação (ADR-0005): nucleação de Cargo.toml como artefato gerido | Cargo.toml |
| 2026-03-16 | ADR-0004: rayon adicionado como dependência de L4 | Cargo.toml |
| 2026-03-18 | ADR-0009: tree-sitter-typescript adicionado (TsParser); tree-sitter-python adicionado (PyParser); tabela de justificativas e nota de extensibilidade adicionadas | Cargo.toml |
