# Prompt: Rule V15 — Multi Prompt Header (multi-prompt-header)
Hash do Código: 50ec428f

**Camada**: L1 (Core — Rules)
**Regra**: V15
**Criado em**: 2026-07-23
**Arquivos gerados**:
  - 01_core/rules/multi_prompt_header.rs + test

---

## Contexto

A regra de linhagem do projecto é **um ficheiro, um prompt**: cada
ficheiro `.rs` em L1–L4 tem exactamente uma linha `//! @prompt` no
bloco de doc-header. O linter nunca foi desenhado para ficheiros com
2+ linhas `@prompt`: `extract_header` fica com o último valor e o
`--fix-hashes` comporta-se de forma indefinida (pode escrever o hash
correcto no header errado).

V15 torna esse estado um **erro bloqueante de lint** em vez de
silêncio ou correcção ambígua: um ficheiro `.rs` em L1–L4 com 2+
linhas `//! @prompt` no bloco de doc-header gera V15 Error, com
mensagem que lista os prompts encontrados e reafirma a regra
(um ficheiro, um prompt).

---

## Especificação

V15 opera sobre `ParsedFile.prompt_refs` por arquivo, na fase Map.
Aplica-se apenas a arquivos com `layer` em {L1, L2, L3, L4}.

### Novo campo em `ParsedFile` — `prompt_refs`

```rust
/// Para V15: todos os valores `@prompt` encontrados no bloco de
/// doc-header (`//!`), em ordem de aparecimento.
/// Populado apenas pelo RustParser — outros parsers usam `vec![]`.
/// `len() <= 1` é o estado legítimo; `len() >= 2` é V15.
pub prompt_refs: Vec<&'a str>,
```

`prompt_refs` inclui também o valor que alimenta
`prompt_header.prompt_path` — ou seja, um ficheiro normal tem
exactamente 1 entrada; um ficheiro sem header tem 0.

### Nova trait — `HasPromptRefs<'a>`

```rust
/// Para V15 — verifica unicidade da linhagem @prompt.
pub trait HasPromptRefs<'a> {
    fn layer(&self) -> &Layer;
    fn prompt_refs(&self) -> &[&'a str];
    fn path(&self) -> &'a Path;
}
```

Implementada por `ParsedFile` em `entities/parsed_file.rs`, na mesma
zona das outras impls de trait de regra.

### Verificação

```rust
pub fn check<'a, T: HasPromptRefs<'a>>(file: &T) -> Vec<Violation<'a>> {
    let refs = file.prompt_refs();
    if refs.len() < 2
        || !matches!(file.layer(), Layer::L1 | Layer::L2 | Layer::L3 | Layer::L4)
    {
        return vec![];
    }

    vec![Violation {
        rule_id: "V15".to_string(),
        level: ViolationLevel::Error,
        message: format!(
            "Arquivo com {} headers @prompt ({}). \
             Regra: um ficheiro, um prompt — dividir o ficheiro ou \
             remover as linhagens extra. --fix-hashes é indefinido \
             com multi-@prompt.",
            refs.len(),
            refs.join(", "),
        ),
        location: Location { path: Cow::Borrowed(file.path()), line: 1, column: 0 },
    }]
}
```

- Nível: **Error** (mesma categoria bloqueante de V1/V3/V4). Sem
  excepções configuráveis: não há caso legítimo de multi-@prompt.
- Localização: linha 1 — o bloco de doc-header vive no topo.
- Uma única violação por ficheiro, independentemente de quantos
  `@prompt` extra existam.

---

## Extracção em L3 (RustParser)

`extract_header` passa a devolver também o vector de refs:

```rust
fn extract_header<'a>(source: &'a str) -> (Option<PromptHeader<'a>>, Vec<&'a str>)
```

A varredura é a mesma de hoje (linhas `//!` do topo, `break` na
primeira linha não-`//!`); cada `content.strip_prefix("@prompt ")`
faz **push** no vector de refs, além de alimentar `prompt_path`
(comportamento de último-valor preservado para V1/V5 — é V15 quem
agora bloqueia o caso ambíguo).

A ordem de verificação existente no `else if` (`@prompt-hash ` antes
de `@prompt `) é preservada — conservadorismo, embora
`"@prompt-hash x"` não case com o prefixo `"@prompt "` (hífen ≠
espaço).

`@prompt` mencionado fora do bloco de doc-header (comentário `//`
normal, string, ou `//!` após código) **não** conta — o `break` na
primeira linha não-`//!` garante isso.

Parsers de outras linguagens (TS, Python, C, C++, Zig) constroem
`ParsedFile` com `prompt_refs: vec![]` — V15 é regra de linhagem de
código Rust.

---

## Wiring (L4 / L2)

- `EnabledChecks` ganha `pub v15: bool`; `from_cli` faz
  `v15: has("v15")`; o default de `--checks` passa a incluir `v15`.
- `run_checks` em `04_wiring/main.rs`:
  `if enabled.v15 { violations.extend(multi_prompt_header::check(file)); }`
- Os dois literais `EnabledChecks { ... }` de re-run em `main.rs`
  (--fix-hashes e --update-snapshot) ganham `v15: false`.
- Lista SARIF em `02_shell/cli.rs` ganha
  `sarif_rule("V15", "MultiPromptHeader", "Multiple @prompt headers in one file", "error")`.

---

## Restrições (L1 Pura)

- Opera sobre `ParsedFile.prompt_refs` — zero I/O
- Sem estado, sem configuração por projecto
- A regra **detecta e bloqueia** — nunca tenta corrigir (escolher um
  dos prompts seria decisão arquitectural, não mecânica)

---

## Critérios de Verificação

```
Dado arquivo L1 com duas linhas @prompt no doc-header
Quando V15::check() for chamado
Então retorna Violation { rule_id: "V15", level: Error, line: 1 }
— e a mensagem lista os dois prompts

Dado arquivo L1 com um único @prompt
Quando V15::check() for chamado
Então retorna vec![]

Dado arquivo L1 sem nenhum @prompt
Quando V15::check() for chamado
Então retorna vec![]
— ausência de header é território de V1, não de V15

Dado arquivo com layer Lab ou Unknown e dois @prompt
Quando V15::check() for chamado
Então retorna vec![]
— V15 aplica-se apenas a L1–L4

Dado arquivo L2 com dois @prompt e fixture de projecto real
Quando o binário roda sobre a fixture v15_fail
Então o veredito é exactamente ["V15"]

Dado ficheiro Rust normal (um @prompt) na fixture v15_pass
Quando o binário roda
Então o veredito é [] — sem falso positivo

Dado ficheiro Rust cujo segundo "@prompt" aparece em comentário //
  normal depois de código (fora do doc-header)
Quando o binário roda sobre a fixture v15b_pass
Então o veredito é [] — só o bloco de doc-header conta
```

---

## Fundamentação Teórica

1. **Unicidade de Linhagem Causal em Grafos de Proveniência:**
   * **Buneman et al. (2001)** (*Why and Where: A Characterization of Data Provenance*): A rastreabilidade determinística de um artefato derivado exige que sua relação de derivação de origem (`prov:wasDerivedFrom`) seja unívoca no nível de granularidade do arquivo. A existência de múltiplas anotações `@prompt` não impede o cálculo — ela o torna silenciosamente incorreto, pois o mecanismo de extração retém apenas o último valor, e ferramentas de correção automática como `--fix-hashes` podem escrever o hash certo associado à referência errada, sem sinalizar erro.
2. **Consistência de Rastreabilidade e Decomposição Modular:**
   * **Erata et al. (2017, 2024)** (*A Tool for Automated Reasoning about Traces Based on Configurable Formal Semantics*): A verificação formal de conformidade pressupõe correspondência semântica clara entre contratos de especificação e unidades de código. A regra V15 impõe o princípio de "um arquivo, um prompt", impedindo que um módulo físico acumule múltiplas especificações concorrentes e forçando a decomposição modular do código.

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-07-23 | Criação inicial (passo P847 — endurecer lint contra multi-@prompt) | multi_prompt_header.rs, parsed_file.rs, rule_traits.rs, rs_parser.rs, cli.rs, main.rs, tests/fixtures.rs |
