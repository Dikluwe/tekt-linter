# Prompt: Rule V7 - Orphan Prompt (orphan-prompt)
Hash do Código: 2b08e766

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
pub fn check_orphans<'a>(
    index: &ProjectIndex<'a>,
    all_prompts: &AllPrompts<'a>,
    level: ViolationLevel,
) -> Vec<Violation<'a>> {
    all_prompts.entries.iter()
        .filter(|entry| !index.referenced_prompts.contains(entry.relative_path))
        .map(|entry| Violation {
            rule_id: "V7".to_string(),
            level: level.clone(),
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
- Zero I/O — `all_prompts` e `referenced_prompts` chegam prontos
- `Level::Warning` por padrão — configurável via `[rules]` no `crystalline.toml`
- O nível é resolvido em L4 (`main.rs`) via `config.level_for("V7", ViolationLevel::Warning)`
  e injectado como parâmetro — L1 não lê config directamente

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

## Fundamentação Teórica

1. **Detecção de Ausências Arquiteturais (Reflexion Models):**
   * **Passos et al. (2010)** (*Static Architecture-Conformance Checking: An Illustrative Overview*): No modelo de conformidade arquitetural estática, discrepâncias onde elementos definidos no modelo pretendido de alto nível não possuem correspondência no código implementado são classificadas como *Ausências* (*Absences*). A regra V7 materializa essa checagem agregada ao comparar o inventário de prompts em $L_0$ contra o conjunto de referências extraídas pelo `ProjectIndex`.
2. **Completude de Grafo de Rastreabilidade (Trace Graph Completeness):**
   * **Erata et al. (2017, 2024)** (*A Tool for Automated Reasoning about Traces Based on Configurable Formal Semantics*): A consistência formal de uma base de requisitos exige a inexistência de especificações desancoradas downstream. Prompts sem materialização em $L_1\dots L_4$ representam ou contratos esquecidos ou resíduos de código removido, justificando a emissão de `Warning` para forçar a materialização ou a remoção do artefato de especificação órfão.

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-14 | Criação inicial (ADR-0006) | orphan_prompt.rs |
| 2026-03-23 | ADR-0014: assinatura com `level: ViolationLevel`; nível hardcoded eliminado; nível resolvido em L4 via `config.level_for` | orphan_prompt.rs |
