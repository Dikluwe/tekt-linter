# Prompt: PromptReader (prompt-reader)

**Camada**: L1 (Core — Contracts) + L3 (Infra — Implementação)
**Criado em**: 2025-03-13
**Arquivos gerados**:
  - 01_core/contracts/prompt_reader.rs
  - 03_infra/prompt_reader.rs + test

---

## Contexto

A regra V5 (drift detection) precisa comparar o hash declarado no
header do arquivo com o hash atual do prompt em `00_nucleo/`.
L1 não pode ler disco — então declara o contrato, L3 implementa.

Este é o único contrato que tem um prompt único para L1 e L3 juntos,
porque são inseparáveis conceitualmente: o contrato só existe por
causa da implementação que o motivou.

## Instrução — L1 (contrato)
```rust
pub trait PromptReader {
    /// Retorna SHA256[0..8] do arquivo de prompt em 00_nucleo/
    /// Retorna None se o arquivo não existe
    fn read_hash(&self, prompt_path: &str) -> Option<String>;

    /// Retorna true se o arquivo existe em 00_nucleo/
    fn exists(&self, prompt_path: &str) -> bool;
}
```

## Instrução — L3 (implementação)
```rust
pub struct FsPromptReader {
    pub nucleo_root: PathBuf,
}

impl PromptReader for FsPromptReader {
    fn read_hash(&self, prompt_path: &str) -> Option<String> {
        let full_path = self.nucleo_root.join(prompt_path);
        let content = std::fs::read(&full_path).ok()?;
        let hash = sha256(&content);
        Some(hash[..8].to_string())
    }

    fn exists(&self, prompt_path: &str) -> bool {
        self.nucleo_root.join(prompt_path).exists()
    }
}
```

`sha256` usa a crate `sha2` em L3 — nunca exposta a L1.

## Restrições

- L1: trait pura, sem sha2, sem std::fs
- L3: implementa com std::fs + sha2, absorve io::Error com Option
- V1 usa `exists()`, V5 usa `read_hash()` — ambas via injeção em L4

## Critérios de Verificação
```
Dado FsPromptReader com nucleo_root apontando para fixture em disco
Quando read_hash("prompts/auth.md") for chamado
Então retorna Some(String) com 8 caracteres hex

Dado prompt_path que não existe
Quando read_hash() for chamado
Então retorna None sem panic

Dado prompt_path que existe
Quando exists() for chamado
Então retorna true

Dado mock de PromptReader que retorna hash fixo
Quando V5::check() usar esse mock
Então regra opera sem nenhum acesso a disco
```

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | contracts/prompt_reader.rs, 03_infra/prompt_reader.rs |
```

---

Com esses quatro prompts, Gap 1 está resolvido e a estrutura de contratos está completa. A estrutura atualizada de L1 fica:
```
01_core/
├── entities/
│   ├── parsed_file.rs
│   ├── violation.rs
│   └── layer.rs
├── contracts/
│   ├── file_provider.rs
│   ├── language_parser.rs
│   ├── parse_error.rs
│   └── prompt_reader.rs
└── rules/
    ├── prompt_header.rs
    ├── test_file.rs
    ├── forbidden_import.rs
    ├── impure_core.rs
    └── prompt_drift.rs
