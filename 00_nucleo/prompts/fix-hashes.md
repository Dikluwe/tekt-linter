# Prompt: Fix Hashes & Update Snapshot Commands (fix-hashes)

**Camada**: L2 + L3 (Shell + Infra)
**Criado em**: 2025-03-13
**Revisado em**: 2025-03-13
**Arquivos gerados**:
  - 02_shell/fix_hashes.rs
  - 02_shell/update_snapshot.rs        ← novo (V6)
  - 03_infra/hash_writer.rs + test
  - 03_infra/snapshot_writer.rs + test ← novo (V6)

---

## Contexto

Dois comandos de mutação que fecham ciclos de divergência detectados
pelo linter:

**`--fix-hashes`** — fecha o ciclo de V5 (PromptDrift).
Após revisão de prompt em L0, arquivos de implementação ficam com
`@prompt-hash` desatualizado. Este comando reescreve os headers
com o hash real do prompt correspondente.

**`--update-snapshot`** — fecha o ciclo de V6 (PromptStale).
Após modificação da interface pública de um arquivo, o snapshot
registrado no prompt fica desatualizado. Este comando serializa
a interface atual e reescreve a seção `## Interface Snapshot`
do prompt correspondente.

Ambas são operações destrutivas — reescrevem arquivos em disco.
Por isso vivem em L2 (decisão de executar) + L3 (escrita em disco).

---

## Comportamento de --fix-hashes
```
crystalline-lint --fix-hashes [--dry-run] [PATH]
```

1. Executa pipeline normal de análise
2. Filtra violations com `rule_id == "V5"`
3. Para cada violation V5:
   - Lê `@prompt` path do header do arquivo fonte
   - Calcula SHA256[0..8] do prompt referenciado
   - Substitui `//! @prompt-hash <old>` por `//! @prompt-hash <new>`
   - Reescreve arquivo atomicamente (write to temp + rename)
4. Reporta quantos arquivos foram corrigidos
5. Re-executa análise para confirmar zero V5

---

## Comportamento de --update-snapshot
```
crystalline-lint --update-snapshot [--dry-run] [PATH]
```

1. Executa pipeline normal de análise
2. Filtra violations com `rule_id == "V6"`
3. Para cada violation V6:
   - Lê `@prompt` path do header do arquivo fonte
   - Serializa `public_interface` atual como JSON
   - Localiza seção `## Interface Snapshot` no prompt
   - Substitui o conteúdo da seção com o novo JSON
   - Atualiza `@updated` no header do prompt para data atual
   - Reescreve prompt atomicamente (write to temp + rename)
4. Reporta quantos prompts foram atualizados
5. Re-executa análise para confirmar zero V6

---

## Contrato L2 — SnapshotWriter (novo)
```rust
/// L2-defined contract para leitura e escrita de snapshots em prompts.
/// L3 provê a implementação concreta.
/// L4 cria o adapter — L2 nunca importa L3 diretamente.
pub trait SnapshotWriter {
    /// Lê a interface pública atual do arquivo fonte.
    /// Retorna None se o arquivo não pode ser lido.
    fn read_interface(&self, source_path: &Path) -> Option<PublicInterface>;

    /// Serializa PublicInterface para JSON compacto.
    fn serialize(&self, interface: &PublicInterface) -> String;

    /// Reescreve atomicamente a seção Interface Snapshot no prompt.
    /// Cria a seção se não existir. Atualiza @updated no header.
    fn write_snapshot(
        &self,
        prompt_path: &str,
        interface: &PublicInterface,
    ) -> Result<(), String>;
}
```

---

## Estrutura da saída
```
# --fix-hashes --dry-run
Would fix 3 files:
  02_shell/cli.rs           00000000 → a3f8c2d1
  03_infra/walker.rs        00000000 → b9e4f7a2
  01_core/entities/layer.rs 00000000 → c1d2e3f4

# --fix-hashes
Fixed 3 files:
  02_shell/cli.rs           → a3f8c2d1
  03_infra/walker.rs        → b9e4f7a2
  01_core/entities/layer.rs → c1d2e3f4
Re-running analysis... ✅ 0 drift warnings remaining

# --update-snapshot --dry-run
Would update 2 prompts:
  00_nucleo/prompts/rules/forbidden-import.md
    +fn check_v2, -fn validate
  00_nucleo/prompts/contracts/file-provider.md
    +struct SourceFile.layer

# --update-snapshot
Updated 2 prompts:
  00_nucleo/prompts/rules/forbidden-import.md
  00_nucleo/prompts/contracts/file-provider.md
Re-running analysis... ✅ 0 stale warnings remaining
```

---

## Restrições

- L3 usa escrita atômica em ambos os comandos — temp file + rename
- L1 não é modificado por nenhum dos comandos
- Se `--dry-run`, nenhum arquivo é tocado
- Se prompt referenciado não existe, arquivo é reportado como
  não corrigível e não é modificado
- `--fix-hashes` e `--update-snapshot` não podem rodar juntos —
  cada um valida e reporta independentemente

---

## Critérios de Verificação
```
Dado arquivo com @prompt-hash 00000000
E prompt correspondente existe em 00_nucleo/
Quando --fix-hashes rodar
Então header é atualizado com SHA256[0..8] real
E re-análise retorna zero V5

Dado arquivo com interface pública alterada desde o snapshot
Quando --update-snapshot rodar
Então seção Interface Snapshot no prompt é atualizada
E @updated no header do prompt é atualizado para hoje
E re-análise retorna zero V6

Dado --fix-hashes --dry-run
Quando rodar
Então nenhum arquivo é modificado
E output mostra mudanças que seriam feitas

Dado --update-snapshot --dry-run
Quando rodar
Então nenhum arquivo é modificado
E output mostra delta de interface que seria registrado

Dado falha de escrita no meio do processo
Quando qualquer comando de mutação rodar
Então arquivo original permanece intacto (escrita atômica)

Dado prompt sem seção Interface Snapshot
Quando --update-snapshot rodar
Então seção é criada no final do prompt, antes do Histórico

Dado projeto sem nenhum V5
Quando --fix-hashes rodar
Então output: "Nothing to fix"

Dado projeto sem nenhum V6
Quando --update-snapshot rodar
Então output: "Nothing to update"
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | fix_hashes.rs, hash_writer.rs |
| 2025-03-13 | V6: adicionado --update-snapshot, SnapshotWriter, update_snapshot.rs, snapshot_writer.rs | fix_hashes.md, update_snapshot.rs, snapshot_writer.rs |
```

---
