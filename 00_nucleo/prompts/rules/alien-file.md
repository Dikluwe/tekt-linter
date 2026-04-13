# Prompt: Rule V8 - Alien File (alien-file)
Hash do Código: 65009e66

**Camada**: L1 (Core — Rules)
**Regra**: V8
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 01_core/rules/alien_file.rs + test

---

## Contexto

O walker atual retorna `Layer::Unknown` silenciosamente para
arquivos fora de diretórios mapeados. Isso cria terra de ninguém
— código que existe no repositório mas escapa completamente de
todas as verificações V1–V7.

V8 converte esse silêncio em violação Fatal. A arquitetura é
hermética — não existe arquivo de código fora da topologia.

---

## Distinção crítica: excluído vs desconhecido
```
target/debug/build.rs    → excluído explicitamente → silêncio (correto)
src/utils/helper.rs      → desconhecido → V8 Fatal
scripts/gen.rs           → desconhecido → V8 Fatal
```

L3 (FileWalker) já distingue os dois casos — arquivos em
diretórios excluídos não aparecem no iterator. Arquivos com
`Layer::Unknown` fora de excluídos chegam ao pipeline e
disparam V8.

---

## Especificação

V8 opera sobre `ProjectIndex.alien_files` — lista de paths
com `Layer::Unknown` construída pelo walker:
```rust
pub fn check_aliens<'a>(index: &ProjectIndex<'a>) -> Vec<Violation<'a>> {
    index.alien_files.iter()
        .map(|path| Violation {
            rule_id: "V8".to_string(),
            level: ViolationLevel::Fatal,
            message: format!(
                "Arquivo fora da topologia: '{}' não pertence a \
                 nenhuma camada mapeada em crystalline.toml. \
                 Mapear o diretório ou mover o arquivo.",
                path.display()
            ),
            location: Location {
                path: Cow::Owned(path.to_path_buf()),
                line: 0,
                column: 0,
            },
        })
        .collect()
}
```

---

## Restrições (L1 Pura)

- Opera sobre `ProjectIndex.alien_files`, não `ParsedFile`
- Nível Fatal — não configurável, mesmo comportamento de V0
- Zero I/O — lista construída por L3

---

## Critérios de Verificação
```
Dado arquivo src/utils/helper.rs com Layer::Unknown
E src/ não mapeado em [layers] nem em [excluded]
Quando check_aliens() for chamado
Então retorna Violation V8 Fatal

Dado arquivo target/debug/build.rs
Quando walker iterar
Então arquivo não aparece em alien_files
— diretório excluído, não chega ao V8

Dado arquivo 01_core/domain/auth.rs com Layer::L1
Quando check_aliens() for chamado
Então não retorna V8 — layer conhecido

Dado alien_files vazio
Quando check_aliens() for chamado
Então retorna vec![]
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | alien_file.rs |
