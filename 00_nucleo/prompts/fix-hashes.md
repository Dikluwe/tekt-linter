# Prompt: Fix Hashes Command (fix-hashes)

**Camada**: L2 + L3 (Shell + Infra)
**Criado em**: 2025-03-13
**Arquivos gerados**:
  - 02_shell/fix_hashes.rs
  - 03_infra/hash_writer.rs + test

---

## Contexto

Após geração inicial ou revisão de prompts em L0, arquivos de
implementação ficam com `@prompt-hash 00000000` ou com hash
desatualizado. V5 detecta essa divergência mas não corrige.

`--fix-hashes` é o comando que fecha esse ciclo: lê todos os
arquivos com drift detectado e reescreve o header com o hash
real do prompt correspondente.

É uma operação destrutiva — reescreve arquivos em disco.
Por isso vive em L2 (decisão de executar) + L3 (escrita em disco),
nunca em L1.

---

## Comportamento
```
crystalline-lint --fix-hashes [PATH]
```

1. Executa o pipeline normal de análise
2. Filtra violations com `rule_id == "V5"`
3. Para cada violation V5:
   - Lê o arquivo fonte
   - Calcula SHA256[0..8] do prompt referenciado
   - Substitui a linha `//! @prompt-hash <old>` por
     `//! @prompt-hash <new>`
   - Reescreve o arquivo
4. Reporta quantos arquivos foram corrigidos
5. Executa análise novamente para confirmar zero V5

---

## Flags relacionadas
```
--fix-hashes        executa correção automática
--dry-run           mostra o que seria corrigido sem reescrever
--fix-hashes --dry-run  combinação válida
```

`--dry-run` é obrigatório implementar junto — operação
destrutiva sem preview é um risco desnecessário.

---

## Responsabilidades por camada

**L2 — `fix_hashes.rs`**
- Recebe `Vec<Violation>` filtrado por V5
- Decide se executa ou apenas reporta (--dry-run)
- Delega reescrita para `HashWriter` de L3
- Reporta resultado para stdout

**L3 — `hash_writer.rs`**
- Recebe path do arquivo e hash novo
- Lê conteúdo atual
- Localiza linha `//! @prompt-hash`
- Substitui valor
- Reescreve arquivo atomicamente (write to temp + rename)

Escrita atômica é obrigatória — falha no meio não deve
deixar arquivo corrompido.

---

## Estrutura da saída
```
# --dry-run
Would fix 3 files:
  02_shell/cli.rs          00000000 → a3f8c2d1
  03_infra/walker.rs       00000000 → b9e4f7a2
  01_core/entities/layer.rs 00000000 → c1d2e3f4

# execução real
Fixed 3 files:
  02_shell/cli.rs          → a3f8c2d1
  03_infra/walker.rs       → b9e4f7a2
  01_core/entities/layer.rs → c1d2e3f4

Re-running analysis... ✅ 0 drift warnings remaining
```

---

## Restrições

- L3 (`hash_writer.rs`) usa escrita atômica — temp file + rename
- L1 não é modificado — fix-hashes é operação de L2/L3
- Se `--dry-run`, nenhum arquivo é tocado
- Se prompt referenciado não existe, arquivo é reportado mas
  não modificado — não inventa hash

---

## Critérios de Verificação
```
Dado arquivo com @prompt-hash 00000000
E prompt correspondente existe em 00_nucleo/
Quando --fix-hashes rodar
Então header é atualizado com SHA256[0..8] real
E re-análise retorna zero V5

Dado --fix-hashes --dry-run
Quando rodar
Então nenhum arquivo é modificado
E output mostra mudanças que seriam feitas

Dado arquivo com @prompt referenciando prompt inexistente
Quando --fix-hashes rodar
Então arquivo é reportado como não corrigível
E não é modificado

Dado falha de escrita no meio do processo
Quando --fix-hashes rodar
Então arquivo original permanece intacto (escrita atômica)

Dado projeto sem nenhum V5
Quando --fix-hashes rodar
Então output: "Nothing to fix"
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2025-03-13 | Criação inicial | fix_hashes.rs, hash_writer.rs |
```

---
