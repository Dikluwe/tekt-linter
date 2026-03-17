# Prompt: Rule V11 - Dangling Contract (dangling-contract)

**Camada**: L1 (Core — Rules)
**Regra**: V11
**Criado em**: 2026-03-16 (ADR-0007)
**Arquivos gerados**:
  - 01_core/rules/dangling_contract.rs + test

---

## Contexto

A Arquitetura Cristalina funda-se na Inversão de Dependência: L1
declara traits (Ports) em `contracts/`, L3 e L2 as implementam
(Adapters), L4 injeta as implementações. O circuito é:

```
declaração (L1/contracts) → implementação (L2 ou L3) → injeção (L4)
```

Quando a IA perde contexto ou é interrompida no meio de uma
refatoração, o resultado comum é uma trait declarada em L1 sem
nenhum `impl` correspondente em L2 ou L3. O contrato existe, mas
o circuito está aberto — não há implementação concreta para injetar.

V1 detecta arquivos sem `@prompt` header. V11 detecta o complemento
estrutural: traits com `@prompt` correto, código sintaticamente
válido, mas sem o `impl` que fecha o circuito.

---

## Especificação

V11 não opera sobre `ParsedFile` individual — requer visão global.
Opera sobre `ProjectIndex` após a fase Reduce, junto com V7 e V8.

### Extração (L3 — RustParser)

Para cada arquivo em L1 com `subdir == "contracts"`:
- Extrair nós `trait_item` com modificador `pub`
- Registrar o nome da trait em `LocalIndex.declared_traits`

Para cada arquivo em L2 ou L3:
- Extrair nós `impl_item` que sejam `impl <TraitName> for <Type>`
  (não `impl <Type>` sem trait)
- Registrar o nome da trait em `LocalIndex.implemented_traits`

### Agregação (ProjectIndex)

```rust
pub struct ProjectIndex<'a> {
    // campos existentes...
    pub all_declared_traits: HashSet<&'a str>,
    pub all_implemented_traits: HashSet<&'a str>,
}
```

`merge_local` absorve os campos de cada `LocalIndex`.
A fusão é associativa e comutativa — segura para rayon.

### Verificação (L1)

```rust
pub fn check_dangling_contracts<'a>(
    index: &ProjectIndex<'a>,
) -> Vec<Violation<'a>> {
    index.all_declared_traits
        .iter()
        .filter(|t| !index.all_implemented_traits.contains(*t))
        .map(|trait_name| Violation {
            rule_id: "V11".to_string(),
            level: ViolationLevel::Error,
            message: format!(
                "Contrato sem implementação: trait '{}' declarada em \
                 L1/contracts/ não tem impl correspondente em L2 ou L3. \
                 O circuito está aberto — nenhuma instância pode ser injetada.",
                trait_name
            ),
            location: Location {
                path: Cow::Owned(PathBuf::from("01_core/contracts")),
                line: 0,
                column: 0,
            },
        })
        .collect()
}
```

---

## Limitação Declarada

A comparação é por **nome simples** da trait, não por caminho
completo. `FileProvider` em `file_provider.rs` e `FileProvider`
hipotético em `language_parser.rs` seriam tratados como a mesma
trait para fins de V11.

Na prática, nomes de traits em `contracts/` de um projeto
Cristalino são únicos por convenção — este é o trade-off aceito
para manter a implementação simples. Uma versão futura pode usar
FQN se colisões se tornarem problema real.

---

## Restrições (L1 Pura)

- Opera sobre `ProjectIndex` — zero I/O, zero acesso a disco
- Nível Error — bloqueia CI por padrão
- Não há exceções configuráveis: uma trait sem impl é sempre
  lixo estrutural, independentemente do contexto
- A localização da violação aponta para `01_core/contracts` com
  `line: 0` — V11 é uma violação global, não de arquivo específico.
  Uma versão futura pode rastrear o path exato da trait.

---

## Critérios de Verificação
```
Dado trait "FileProvider" em all_declared_traits
E "FileProvider" ausente em all_implemented_traits
Quando check_dangling_contracts() for chamado
Então retorna Violation V11 Error mencionando "FileProvider"

Dado trait "LanguageParser" em all_declared_traits
E "LanguageParser" presente em all_implemented_traits
Quando check_dangling_contracts() for chamado
Então não retorna V11 para "LanguageParser"

Dado all_declared_traits == all_implemented_traits
Quando check_dangling_contracts() for chamado
Então retorna vec![]

Dado duas traits declaradas, uma implementada e outra não
Quando check_dangling_contracts() for chamado
Então retorna exatamente uma violação V11

Dado all_declared_traits vazio
Quando check_dangling_contracts() for chamado
Então retorna vec![] — sem contratos, sem violações

Dado trait declarada em arquivo L1 fora de "contracts/"
(ex: em "rules/" ou "entities/")
Quando RustParser extrair declared_traits
Então a trait não entra em declared_traits
— V11 só cobre traits em subdir "contracts"
```

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-03-16 | Criação inicial (ADR-0007) | dangling_contract.rs |
| 2026-03-16 | Materialização: check_dangling_contracts() implementado sobre ProjectIndex, 8 testes cobrindo todos os critérios; módulo registado em rules/mod.rs | dangling_contract.rs |
