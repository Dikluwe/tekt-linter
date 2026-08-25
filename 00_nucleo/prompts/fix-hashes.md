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

## Solução do Paradoxo do Hash (Dupla Paridade)

Para evitar o ciclo infinito onde a alteração do Hash do Prompt no Código altera o Hash do Código, e vice-versa, o linter **ignora a linha de metainformação durante o cálculo**.

### Algoritmo de 5 passos:
Hash do Código: 2c13736a
2. Calcula o **Hash A** usando apenas o texto limpo do `.md`.
3. O linter lê o ficheiro `.rs` (ou outra linguagem) inteiro para a memória e apaga temporariamente a linha `//! @prompt-hash [valor]`.
4. Calcula o **Hash B** usando apenas o texto limpo do ficheiro de código.
5. O linter injeta o **Hash A** no ficheiro de código e injeta o **Hash B** no ficheiro `.md`.

Este método garante que a dupla paridade funciona de forma cruzada sem interferir no resultado do cálculo do ficheiro oposto.

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

## Contrato L2 — planejamento e execução de snapshots

L2 possui o caso de uso e o port. L3 implementa serialização canônica e escrita; L4
somente instancia e injeta o adapter.

```rust
pub trait SnapshotRewriter {
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String;
    fn write_snapshot(
        &self,
        prompt_path: &str,
        snapshot: &str,
    ) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotUnreadable {
    MissingParsedFile,
    MissingPromptHeader,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotEntry {
    Ready {
        source_path: PathBuf,
        prompt_path: String,
        snapshot: String,
    },
    Unreadable {
        source_path: PathBuf,
        reason: SnapshotUnreadable,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotResult {
    DryRun {
        source_path: PathBuf,
        prompt_path: String,
        snapshot: String,
    },
    Written {
        source_path: PathBuf,
        prompt_path: String,
    },
    WriteFailed {
        source_path: PathBuf,
        prompt_path: String,
        reason: String,
    },
    Unreadable {
        source_path: PathBuf,
        reason: SnapshotUnreadable,
    },
}

pub fn plan<'a>(
    violations: &[Violation<'a>],
    parsed_files: &[ParsedFile<'a>],
    rewriter: &dyn SnapshotRewriter,
) -> Vec<SnapshotEntry>;

pub fn execute(
    entries: &[SnapshotEntry],
    rewriter: &dyn SnapshotRewriter,
    dry_run: bool,
) -> Vec<SnapshotResult>;
```

`plan` produz exatamente uma entrada para cada ocorrência cujo `rule_id` seja exatamente
`"V6"`, preservando ordem e duplicatas. Para associar o arquivo, usa o primeiro
`ParsedFile` na ordem recebida cujo `path` seja integralmente igual ao path da violação;
não normaliza, canonicaliza, compara prefixo ou basename.

Sem arquivo associado, produz `Unreadable::MissingParsedFile`. Sem header, produz
`Unreadable::MissingPromptHeader`. Esses estados não chamam serialização. `Ready`
preserva source path e prompt path do mesmo arquivo e chama `serialize_snapshot` uma vez
com sua `public_interface`.

`execute` produz exatamente um resultado por entrada, na mesma ordem. `Unreadable`
permanece `Unreadable`, sem chamada ao port. Em dry-run, cada `Ready` vira `DryRun` com
path e snapshot inalterados e nenhuma escrita. Em execução real, cada `Ready` chama
`write_snapshot` exatamente uma vez; `Ok` vira `Written` e `Err(reason)` vira
`WriteFailed` com a razão exata. Falhas não interrompem itens posteriores.

L2 não lê filesystem, ambiente, relógio, rede ou processo. Somente a implementação L3 do
port produz efeito externo.

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
  e captura falhas em `SnapshotEntry::Unreadable`
- `execute()` preserva cardinalidade inclusive para entradas não acionáveis
- dry-run possui resultado distinto de escrita realizada
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
