# Prompt: PromptSnapshotReader (prompt-snapshot-reader)
Hash do Código: 7e7e5206

**Camada**: L1 (Core — Contracts) + L3 (Infra — Implementação)
**Criado em**: 2025-03-13
**Arquivos gerados**:
  - 01_core/contracts/prompt_snapshot_reader.rs
  - 03_infra/prompt_snapshot_reader.rs + test

---

## Contexto

V6 (PromptStale) compara a interface pública atual do código com o
snapshot registrado no prompt de origem. L1 não pode ler o prompt
diretamente — este contrato define a fronteira: L3 lê e desserializa,
L1 consome via `ParsedFile.prompt_snapshot`.

O snapshot vive numa seção especial do prompt em L0:
```markdown
## Interface Snapshot
<!-- GENERATED — não edite manualmente -->
<!-- crystalline-snapshot: {"functions":[...],"types":[...],"reexports":[]} -->
```

Esta seção é gerada automaticamente pelo linter no momento da
materialização e atualizada sempre que o prompt é revisado.
É a única seção de um prompt que pode ser modificada por ferramenta
— todo o resto é autoria humana.

---

## Instrução — L1 (contrato)
```rust
pub trait PromptSnapshotReader {
    /// Lê e desserializa o snapshot de interface do arquivo de prompt.
    /// Retorna None se:
    /// - o arquivo não existe
    /// - não contém seção ## Interface Snapshot
    /// - o JSON do snapshot é inválido
    fn read_snapshot(&self, prompt_path: &str) -> Option<PublicInterface>;

    /// Serializa uma PublicInterface para o formato de snapshot.
    /// Usado pelo comando --update-snapshot para gravar no prompt.
    fn serialize_snapshot(&self, interface: &PublicInterface) -> String;
}
```

`PublicInterface` vem de `crate::entities::parsed_file`.

---

## Instrução — L3 (implementação)
```rust
pub struct FsPromptSnapshotReader {
    pub nucleo_root: PathBuf,
}

impl PromptSnapshotReader for FsPromptSnapshotReader {
    fn read_snapshot(&self, prompt_path: &str) -> Option<PublicInterface> {
        let full_path = self.nucleo_root.join(prompt_path);
        let content = std::fs::read_to_string(&full_path).ok()?;
        extract_snapshot_json(&content)
            .and_then(|json| serde_json::from_str(&json).ok())
    }

    fn serialize_snapshot(&self, interface: &PublicInterface) -> String {
        let json = serde_json::to_string(interface)
            .unwrap_or_default();
        format!(
            "## Interface Snapshot\n\
             <!-- GENERATED — não edite manualmente -->\n\
             <!-- crystalline-snapshot: {} -->",
            json
        )
    }
}

/// Extrai o conteúdo JSON da linha de snapshot no prompt.
fn extract_snapshot_json(content: &str) -> Option<String> {
    content.lines()
        .find(|line| line.contains("crystalline-snapshot:"))
        .and_then(|line| {
            let start = line.find('{')? ;
            let end = line.rfind('}')? + 1;
            Some(line[start..end].to_string())
        })
}
```

`serde_json` é usado em L3 — nunca exposto a L1.
`PublicInterface` precisa derivar `Serialize` e `Deserialize`
em sua definição em L1, usando `serde` sem I/O.

---

## Formato do snapshot no prompt
```markdown
## Interface Snapshot
<!-- GENERATED — não edite manualmente -->
<!-- crystalline-snapshot: {"functions":[{"name":"check","params":["&ParsedFile"],"return_type":"Vec<Violation>"}],"types":[],"reexports":[]} -->
```

Regras de formato:
- Uma única linha de comentário HTML contendo JSON compacto
- Sempre na última seção do prompt, antes do Histórico de Revisões
- JSON é o único formato aceito — sem TOML, sem YAML

---

## Comando --update-snapshot

Quando V6 dispara, o desenvolvedor resolve com:
```bash
crystalline-lint --update-snapshot .
```

O linter lê a interface pública atual de cada arquivo com V6,
serializa via `PromptSnapshotReader::serialize_snapshot()`,
e reescreve a seção `## Interface Snapshot` do prompt correspondente.
Atualiza também `@updated` no header do prompt.

Requer prompt `fix-hashes.md` (revisão) para registrar
`--update-snapshot` como novo comando de mutação em L2/L3.

---

## Restrições

- L1: trait pura — zero serde_json, zero std::fs
- L3: implementa com std::fs + serde_json, absorve erros com Option
- `PublicInterface` deriva `serde::Serialize + serde::Deserialize`
  na sua definição em L1 — serde como dependência de dados, não I/O
- Snapshot malformado retorna None silenciosamente — V6 não dispara
  sem baseline válida

---

## Critérios de Verificação
```
Dado prompt com seção Interface Snapshot contendo JSON válido
Quando read_snapshot() for chamado
Então retorna Some(PublicInterface) desserializada corretamente

Dado prompt sem seção Interface Snapshot
Quando read_snapshot() for chamado
Então retorna None sem panic

Dado prompt com JSON de snapshot malformado
Quando read_snapshot() for chamado
Então retorna None sem panic

Dado PublicInterface com uma função "check"
Quando serialize_snapshot() for chamado
Então retorna string contendo "crystalline-snapshot:" e o JSON

Dado mock de PromptSnapshotReader retornando snapshot fixo
Quando V6::check() usar esse mock
Então regra opera sem nenhum acesso a disco
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | contracts/prompt_snapshot_reader.rs, 03_infra/prompt_snapshot_reader.rs |
