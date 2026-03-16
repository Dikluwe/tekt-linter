# Prompt: Rule V7 - Orphan Prompt (orphan-prompt)

**Camada**: L1 (Core — Rules)
**Regra**: V7
**Criado em**: 2026-03-14 (ADR-0006)
**Arquivos gerados**:
  - 01_core/rules/orphan_prompt.rs + test

---

## Contexto

V1 verifica que todo arquivo de código aponta para um prompt
existente. V7 verifica a direção oposta: todo prompt em
`00_nucleo/prompts/` deve ter pelo menos um arquivo em L1–L4
com `@prompt` header apontando para ele.

Prompts sem materialização são sementes estéreis — indicam
contratos propostos mas nunca implementados, ou código removido
sem limpeza do L0 correspondente.

---

## Especificação

V7 não opera sobre `ParsedFile` individual. Opera sobre
`ProjectIndex` — estrutura global construída por L3 após varrer
todo o projeto:
```rust
pub fn check_orphans<'a>(index: &ProjectIndex<'a>) -> Vec<Violation<'a>> {
    index.all_prompts.iter()
        .filter(|prompt| !index.referenced_prompts.contains(*prompt))
        .map(|prompt| Violation {
            rule_id: "V7".to_string(),
            level: ViolationLevel::Warning,
            message: format!(
                "Prompt órfão: '{}' não é referenciado por nenhum \
                 arquivo em L1–L4. Materializar ou remover.",
                prompt
            ),
            location: Location {
                path: Cow::Owned(PathBuf::from(prompt.to_string())),
                line: 0,
                column: 0,
            },
        })
        .collect()
}
```

---

## Exceções

Prompts que existem legitimamente sem materialização Rust são
declarados em `crystalline.toml`:
```toml
[orphan_exceptions]
"prompts/template.md"  = "template — não materializa diretamente"
"prompts/readme.md"    = "gera README.md, não arquivo Rust"
"prompts/cargo.md"     = "gera Cargo.toml, não arquivo Rust"
```

L3 exclui essas entradas de `all_prompts` antes de construir
o `ProjectIndex`. V7 nunca as vê.

---

## Restrições (L1 Pura)

- Opera sobre `ProjectIndex`, não `ParsedFile`
- Zero I/O — `all_prompts` e `referenced_prompts` chegam
  prontos via `ProjectIndex`
- `Level::Warning` por padrão — configurável para `Error`
  via `crystalline.toml`

---

## Critérios de Verificação
```
Dado prompt "prompts/novo-contrato.md" em all_prompts
E nenhum arquivo com @prompt apontando para ele
Quando check_orphans() for chamado
Então retorna Violation V7 com path do prompt órfão

Dado prompt "prompts/auth.md" em all_prompts
E arquivo 01_core/rules/auth.rs com @prompt apontando para ele
Quando check_orphans() for chamado
Então não retorna V7 para esse prompt

Dado prompt "prompts/readme.md" em orphan_exceptions
Quando check_orphans() for chamado
Então não retorna V7 — exceção declarada

Dado all_prompts == referenced_prompts
Quando check_orphans() for chamado
Então retorna vec![]
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | orphan_prompt.rs |
