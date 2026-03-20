# Prompt: Fix Hashes & Update Snapshot Commands (fix-hashes)

**Camada**: L2 + L3 (Shell + Infra)
**Criado em**: 2025-03-13
**Revisado em**: 2026-03-20 (plan() reporta falhas de leitura em vez de silenciar)
**Arquivos gerados**:
  - 02_shell/fix_hashes.rs
  - 02_shell/update_snapshot.rs
  - 03_infra/hash_writer.rs + test
  - 03_infra/snapshot_writer.rs + test

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
   - Tenta ler `@prompt` path e `@prompt-hash` actual do header
   - Se leitura falha → registar como não-corrigível com razão
   - Se leitura ok → calcular SHA256[0..8] do prompt referenciado
   - Reescreve `//! @prompt-hash <old>` por `//! @prompt-hash <new>`
   - Reescreve arquivo atomicamente (write to temp + rename)
4. Reporta ficheiros corrigidos, não-corrigíveis e razões
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

## Estrutura de dados — `FixEntry`

```rust
pub struct FixEntry {
    pub source_path: PathBuf,
    /// Hash actualmente escrito no header do ficheiro.
    /// Vazio se unreadable_reason está preenchido.
    pub old_hash: String,
    /// Real hash do ficheiro de prompt L0.
    /// None se o ficheiro de prompt não existe (não corrigível).
    pub new_hash: Option<String>,
    /// None se o header foi lido com sucesso.
    /// Some(reason) se read_header falhou — entrada não corrigível
    /// com razão explícita. Nunca descartar silenciosamente.
    pub unreadable_reason: Option<String>,
}
```

---

## Contrato L2 — `plan()` em `fix_hashes`

`plan()` não descarta entradas silenciosamente. Se `read_header`
retorna `None` para um ficheiro (header malformado, ficheiro
modificado entre análise e execução, permissões), a entrada é
incluída com `unreadable_reason` preenchido.

```rust
pub fn plan(violations: &[Violation<'_>], rewriter: &dyn HashRewriter) -> Vec<FixEntry> {
    violations
        .iter()
        .filter(|v| v.rule_id == "V5")
        .map(|v| {
            match rewriter.read_header(&v.location.path) {
                None => FixEntry {
                    source_path: v.location.path.to_path_buf(),
                    old_hash: String::new(),
                    new_hash: None,
                    unreadable_reason: Some(format!(
                        "não foi possível ler o header de '{}'",
                        v.location.path.display()
                    )),
                },
                Some((prompt_path, old_hash)) => FixEntry {
                    source_path: v.location.path.to_path_buf(),
                    old_hash,
                    new_hash: rewriter.compute_hash(&prompt_path),
                    unreadable_reason: None,
                },
            }
        })
        .collect()
}
```

O mesmo princípio aplica-se a `update_snapshot::plan` — falhas
de leitura são reportadas, não descartadas.

---

## Contrato L2 — `SnapshotRewriter` (novo)
```rust
pub trait SnapshotWriter {
    fn read_interface(&self, source_path: &Path) -> Option<PublicInterface>;
    fn serialize(&self, interface: &PublicInterface) -> String;
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

Cannot fix 1 file (header unreadable):
  01_core/entities/layer.rs  — não foi possível ler o header

# --fix-hashes
Fixed 2 files:
  02_shell/cli.rs           → a3f8c2d1
  03_infra/walker.rs        → b9e4f7a2

Cannot fix 1 file (header unreadable):
  01_core/entities/layer.rs  — não foi possível ler o header

Re-running analysis... ✅ 0 drift warnings remaining
```

---

## Restrições

- L3 usa escrita atômica em ambos os comandos — temp file + rename
- L1 não é modificado por nenhum dos comandos
- Se `--dry-run`, nenhum arquivo é tocado
- `plan()` nunca descarta entradas com `filter_map` — usa `map`
  e captura falhas em `unreadable_reason`
- `--fix-hashes` e `--update-snapshot` não podem rodar juntos

---

## Critérios de Verificação
```
Dado arquivo com @prompt-hash 00000000
E prompt correspondente existe em 00_nucleo/
Quando --fix-hashes rodar
Então header é atualizado com SHA256[0..8] real
E re-análise retorna zero V5

Dado violation V5 para ficheiro cujo header não pode ser lido
Quando plan() for chamado
Então entries.len() == 1
E entries[0].unreadable_reason == Some(...)
— falha reportada, não descartada silenciosamente

Dado violation V5 para ficheiro com header válido
Quando plan() for chamado com MockRewriter que retorna None de read_header
Então entries.len() == 1
E entries[0].unreadable_reason é Some com mensagem explicativa

Dado --fix-hashes --dry-run
Quando rodar
Então nenhum arquivo é modificado
E output mostra mudanças que seriam feitas
E output mostra entradas não-corrigíveis com razão

Dado falha de escrita no meio do processo
Quando qualquer comando de mutação rodar
Então arquivo original permanece intacto (escrita atômica)

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
| 2025-03-13 | V6: adicionado --update-snapshot, SnapshotWriter, update_snapshot.rs, snapshot_writer.rs | fix_hashes.md |
| 2026-03-20 | plan() corrigido: filter_map → map com unreadable_reason; falhas de read_header reportadas em vez de silenciadas; FixEntry ganha campo unreadable_reason; critérios adicionados | fix_hashes.rs, update_snapshot.rs |
