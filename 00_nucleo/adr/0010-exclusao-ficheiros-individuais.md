# ⚖️ ADR-0010: Exclusão de Ficheiros Individuais e Tratamento de `lib.rs`

**Status**: `IMPLEMENTADO`
**Data**: 2026-03-20

---

## Contexto

O mecanismo `[excluded]` do `crystalline.toml` foi concebido para
**directórios** — `target`, `.git`, `node_modules`. A implementação
em `is_ignored` compara componentes de path:

```rust
fn is_ignored(path: &Path, excluded: &HashSet<String>) -> bool {
    path.components().any(|c| {
        let name = c.as_os_str().to_str().unwrap_or("");
        excluded.contains(name)
    })
}
```

Isso funciona correctamente para directórios porque o nome do
directório aparece como componente em todos os paths que o contêm.
Mas não é adequado para ficheiros individuais: a entrada
`lib_root = "lib.rs"` excluiria **qualquer ficheiro chamado
`lib.rs`** em qualquer subdirectório do projecto, não apenas o da
raiz.

### O problema com `lib.rs`

`lib.rs` é o ponto de reexport da crate — declarado em
`Cargo.toml` como `[lib] path = "lib.rs"`. Está na raiz do
projecto, fora de qualquer directório mapeado em `[layers]`.

Quando a entrada `lib_root = "lib.rs"` foi removida de
`[excluded]` para diagnóstico, duas violações dispararam:

- **V8 Fatal** — `lib.rs` está fora de todos os directórios
  mapeados em `[layers]`, logo `resolve_file_layer` retorna
  `Layer::Unknown`, que o walker propaga para `LocalIndex`
  e V8 detecta como alien file.

- **V5 Warning** — o header de `lib.rs` declara
  `@layer L1`, mas o walker resolve-o como `Layer::Unknown`.
  Esta contradição entre o layer declarado no header e o layer
  resolvido pelo walker é silenciosa — V5 não a detecta porque
  V5 compara hashes, não layers.

A entrada foi reposta com comentário a marcar o workaround:

```toml
[excluded]
lib_root = "lib.rs"  # workaround — ver ADR-0010
```

### Por que `lib.rs` não pertence a nenhuma camada existente

`lib.rs` não contém lógica de negócio, contratos, regras, CLI,
infra, nem wiring. É exclusivamente um ficheiro de reexport que
expõe os módulos da crate ao binário em `04_wiring/main.rs` e
a consumers externos. O seu conteúdo é:

```rust
pub mod entities;   // L1
pub mod contracts;  // L1
pub mod rules;      // L1
pub mod infra;      // L3
pub mod shell;      // L2
```

Mapeá-lo para L1, L2, L3 ou L4 seria semanticamente incorrecto —
não pertence a nenhuma camada, pertence à crate como um todo.
Declarar `@layer L1` no seu header (estado actual) é uma
aproximação incorrecta que introduz uma contradição silenciosa
com o walker.

---

## Decisão

### 1. Introduzir `[excluded_files]` no `crystalline.toml`

Adicionar uma secção separada para exclusão de ficheiros
individuais por path relativo à raiz do projecto:

```toml
[excluded_files]
# Ficheiros excluídos individualmente por path relativo.
# Distinto de [excluded] que opera sobre directórios.
# Usar apenas para ficheiros que existem fora da topologia de
# camadas por razões estruturais da linguagem ou do tooling.
crate_root = "lib.rs"
```

O valor é o path relativo à raiz do projecto. A chave é apenas
documentação — o walker usa o valor.

### 2. Actualizar `is_ignored` para verificar `[excluded_files]`

```rust
fn is_ignored(
    path: &Path,
    root: &Path,
    excluded_dirs: &HashSet<String>,
    excluded_files: &HashSet<String>,
) -> bool {
    // Verificação de directório (comportamento existente)
    if path.components().any(|c| {
        let name = c.as_os_str().to_str().unwrap_or("");
        excluded_dirs.contains(name)
    }) {
        return true;
    }
    // Verificação de ficheiro individual por path relativo
    if let Ok(relative) = path.strip_prefix(root) {
        if let Some(rel_str) = relative.to_str() {
            // Normalizar separadores para comparação cross-platform
            let normalized = rel_str.replace('\\', "/");
            if excluded_files.contains(&normalized) {
                return true;
            }
        }
    }
    false
}
```

`excluded_dirs` constrói-se de `config.excluded` (comportamento
actual). `excluded_files` constrói-se de `config.excluded_files`
(novo campo).

### 3. Corrigir o header de `lib.rs`

O header actual declara `@layer L1`, o que é incorrecto. Como
`lib.rs` é excluído explicitamente de `[excluded_files]`, V1 não
vai disparar para ele — mas o header deve reflectir a realidade.

Substituir `@layer L1` por `@layer L0`:

```rust
//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash <hash>
//! @layer L0
//! @updated 2026-03-20
```

**Justificativa:** L0 é o único estrato que não tem restrições de
import, não gera violações de I/O (V4), e não é verificado por V2.
`lib.rs` é documentação executável da estrutura da crate — mais
próximo de L0 do que de qualquer estrato de implementação. Não
é ideal, mas é a aproximação menos incorrecta dentro do sistema
de layers existente.

### 4. Remover o workaround do `crystalline.toml`

Após implementar `[excluded_files]`:

```toml
# Remover:
[excluded]
lib_root = "lib.rs"  # workaround — ver ADR-0010

# Adicionar:
[excluded_files]
crate_root = "lib.rs"
```

---

## Impacto na IR e nos ficheiros

### `CrystallineConfig` — novo campo

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CrystallineConfig {
    // ... campos existentes ...

    /// Ficheiros individuais excluídos por path relativo à raiz.
    /// Distinto de `excluded` que opera sobre nomes de directório.
    /// Exemplo: { "crate_root" = "lib.rs" }
    #[serde(default)]
    pub excluded_files: HashMap<String, String>,
}
```

### `FileWalker` — actualização de `is_ignored`

`FileWalker::new` passa a construir dois conjuntos separados:
- `excluded_dirs` — de `config.excluded` (valores)
- `excluded_files` — de `config.excluded_files` (valores)

A função `is_ignored` recebe ambos.

### `walker.rs` — assinatura de `is_ignored`

A função pública `resolve_file_layer` não muda — o mecanismo de
exclusão é anterior à resolução de layer no pipeline do walker.

---

## Prompts afectados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `file-walker.md` | `[excluded_files]`, `is_ignored` com dois conjuntos, critérios novos |
| `linter-core.md` | `[excluded_files]` no exemplo de `crystalline.toml`; nota sobre `lib.rs` |
| `cargo.md` | Não afectado |

---

## Consequências

### ✅ Positivas

- **Exclusão precisa**: `lib.rs` na raiz é excluído sem afectar
  ficheiros `lib.rs` em subdirectórios (que seriam alien files
  legítimos e devem disparar V8)
- **Semântica clara**: `[excluded]` para directórios,
  `[excluded_files]` para ficheiros — a distinção é
  explícita no toml e no código
- **Workaround eliminado**: a entrada com comentário
  `# workaround — ver ADR-0010` deixa de existir
- **Header de `lib.rs` correcto**: a contradição entre
  `@layer L1` e `Layer::Unknown` resolvida

### ❌ Negativas

- Pequena expansão de superfície de configuração no toml
- `is_ignored` fica mais complexo (dois conjuntos em vez de um)
- `FileWalker` precisa de acesso à raiz do projecto para
  `strip_prefix` — já tem via `self.root`, sem mudança de
  contrato

### ⚙️ Neutras

- Projectos existentes sem `[excluded_files]` continuam a
  funcionar — campo tem `#[serde(default)]`
- V8 para outros ficheiros fora da topologia não é afectado —
  `[excluded_files]` é uma lista explícita opt-in, não um
  mecanismo genérico

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Manter workaround `lib_root = "lib.rs"` em `[excluded]` | Zero esforço | Exclui qualquer `lib.rs` no projecto; semanticamente errado |
| Mapear `lib.rs` em `[layers]` como L4 | Sem novo campo no toml | `lib.rs` não é wiring; introduz falsos positivos V1/V2/V3 |
| Mover `lib.rs` para `04_wiring/lib.rs` | Semanticamente mais correcto | Muda estrutura de crate; requer actualização de `Cargo.toml` e todos os headers |
| Suporte a glob em `[excluded]` (ex: `*.rs` na raiz) | Flexível | Complexidade de implementação alta; semântica ambígua |

---

## Referências

- ADR-0006: Fechamento Topológico — introduziu V8 e a distinção
  excluído vs desconhecido
- `file-walker.md` — `is_ignored` e `resolve_file_layer`
- `linter-core.md` — estrutura de ficheiros e `crystalline.toml`
