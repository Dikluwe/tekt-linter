# Assessment 0032-A — inventário de ownership de prompts

> **Passo:** P0104 — propriedade biunívoca entre prompts e código  
> **Escopo:** protocolo A; somente leitura de produção, IR e configuração  
> **Data:** 2026-08-25  
> **Resultado:** `RED` material e horizonte finito; `SPEC-GAP` no saneamento semântico

## Pinagem causal

Os sete insumos locais hash-pinned pelo P0104 foram recalculados antes do inventário e
coincidem byte a byte com o passo:

| Unidade | SHA-256 observado | Estado |
|---|---|---|
| contrato V15 | `81ba0f080eac8c2db78f27f04f206ff746eecdd358fdb55b146523192704f053` | `PASS` |
| produção V15 | `123f2ab3da2c130ae47d624b731327ec857d8b752399c1b744a8d48d7d86a400` | `PASS` |
| contrato fix-hashes | `d6cc361ed70301c002717b6e80a6c166a0ba1f149084c0f3000c373ba5d1daf9` | `PASS` |
| produção fix-hashes | `26252ef696b1026168568e992a10b41a25b9848bd94e1ab0fa01403288ea3278` | `PASS` |
| arquitetura Tekt | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` | `PASS` |
| índice atual | `9bf8d5e772761347c52f628d9a0cde57d1a4dbd931dcb5e66968e6558e62aa91` | `PASS` |
| wiring atual | `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` | `PASS` |

O Typst Crystalline não foi lido nem modificado por A.

## Método e correspondência com a IR produtiva

O domínio foi obtido com a mesma seleção produtiva configurada pelo `FileWalker`:
diretórios L1–L4, extensões reconhecidas, exclusões de `crystalline.toml`, sem seguir
symlinks. Todos os 76 arquivos do domínio atual são Rust. Para cada arquivo foi aplicado
o mesmo limite lexical de `rs_parser::extract_header`: apenas o bloco inicial contíguo
de linhas `//!`; o valor canônico é `ParsedFile.prompt_header.prompt_path`.

Não foi possível consultar essa visão diretamente pelo binário: não existe comando de
dump da IR de lineage. A reprodução é fiel no estado observado (76 headers simples),
mas essa ausência de observabilidade deve ser coberta pelo gate B2, que exercita o
consumidor real. Como controle independente, `cargo run --quiet -- . --checks v7
--fail-on error` retornou `No violations found`, confirmando zero órfãos após as
exceções produtivas do `FsPromptWalker`.

## Contagens

| Medida | Quantidade |
|---|---:|
| arquivos/consumers produtivos L1–L4 | 76 |
| prompts referenciados distintos | 45 |
| prompts com exatamente um consumer | 32 |
| prompts compartilhados | 13 |
| consumers sob prompts compartilhados | 44 |
| consumers locais ambíguos (2+ `@prompt` no mesmo arquivo) | 0 |
| prompts órfãos após `orphan_exceptions` | 0 |
| novos prompts proprietários mínimos para bijeção | 31 |

O mínimo de 31 é `Σ(cardinalidade - 1)` sobre os 13 compartilhamentos. Portanto o
horizonte estrutural é finito: 44 pares hoje ambíguos precisam terminar em 44 prompts
distintos; os 32 pares já únicos não exigem individualização. Esse número não autoriza
duplicação mecânica de texto L0.

## Mapa integral `prompt → consumers`

### Prompts compartilhados (`RED` para a nova V15 global)

| Prompt | Consumers |
|---|---|
| `00_nucleo/prompts/linter-core.md` | `01_core/contracts/mod.rs`; `01_core/rules/mod.rs`; `02_shell/mod.rs`; `02_shell/n16_summary.rs`; `03_infra/config.rs`; `03_infra/elixir_parser.rs`; `03_infra/go_parser.rs`; `03_infra/java_parser.rs`; `03_infra/mod.rs`; `04_wiring/main.rs` |
| `00_nucleo/prompts/refinement-validator.md` | `01_core/entities/refinement.rs`; `02_shell/refinement.rs`; `03_infra/git_refinement.rs`; `03_infra/refinement_extractor.rs`; `03_infra/refinement_snapshot.rs` |
| `00_nucleo/prompts/rules/wildcard-saturation.md` | `01_core/rules/compound_guard.rs`; `01_core/rules/deep_pattern_nesting.rs`; `01_core/rules/or_pattern_alternatives.rs`; `01_core/rules/range_pattern.rs`; `01_core/rules/wildcard_saturation.rs` |
| `00_nucleo/prompts/fix-hashes.md` | `02_shell/fix_hashes.rs`; `02_shell/update_snapshot.rs`; `03_infra/hash_writer.rs`; `03_infra/snapshot_writer.rs` |
| `00_nucleo/prompts/violation-types.md` | `01_core/entities/layer.rs`; `01_core/entities/mod.rs`; `01_core/entities/parsed_file.rs`; `01_core/entities/violation.rs` |
| `00_nucleo/prompts/contracts/citation-freshness.md` | `01_core/contracts/citation_freshness.rs`; `03_infra/citation_freshness.rs` |
| `00_nucleo/prompts/contracts/prompt-reader.md` | `01_core/contracts/prompt_reader.rs`; `03_infra/prompt_reader.rs` |
| `00_nucleo/prompts/contracts/prompt-snapshot-reader.md` | `01_core/contracts/prompt_snapshot_reader.rs`; `03_infra/prompt_snapshot_reader.rs` |
| `00_nucleo/prompts/file-walker.md` | `03_infra/prompt_io.rs`; `03_infra/walker.rs` |
| `00_nucleo/prompts/rules/external-type-in-contract.md` | `01_core/entities/l1_allowed_external.rs`; `01_core/rules/external_type_in_contract.rs` |
| `00_nucleo/prompts/sarif-formatter.md` | `02_shell/cli.rs`; `02_shell/path_encoding.rs` |
| `00_nucleo/prompts/segregated-materialization.md` | `01_core/entities/refinement_seal.rs`; `03_infra/refinement_seal.rs` |
| `00_nucleo/prompts/unsourced-constant.md` | `01_core/rules/provenance_inventory.rs`; `01_core/rules/unsourced_constant.rs` |

### Prompts com ownership único

| Prompt | Consumer |
|---|---|
| `00_nucleo/prompts/contracts/file-provider.md` | `01_core/contracts/file_provider.rs` |
| `00_nucleo/prompts/contracts/language-parser.md` | `01_core/contracts/language_parser.rs` |
| `00_nucleo/prompts/contracts/parse-error.md` | `01_core/contracts/parse_error.rs` |
| `00_nucleo/prompts/contracts/prompt-provider.md` | `01_core/contracts/prompt_provider.rs` |
| `00_nucleo/prompts/contracts/rule-traits.md` | `01_core/entities/rule_traits.rs` |
| `00_nucleo/prompts/crate-registry.md` | `03_infra/crate_registry.rs` |
| `00_nucleo/prompts/parsers/c.md` | `03_infra/c_parser.rs` |
| `00_nucleo/prompts/parsers/cpp.md` | `03_infra/cpp_parser.rs` |
| `00_nucleo/prompts/parsers/python.md` | `03_infra/py_parser.rs` |
| `00_nucleo/prompts/parsers/rust.md` | `03_infra/rs_parser.rs` |
| `00_nucleo/prompts/parsers/typescript.md` | `03_infra/ts_parser.rs` |
| `00_nucleo/prompts/parsers/zig.md` | `03_infra/zig_parser.rs` |
| `00_nucleo/prompts/project-index.md` | `01_core/entities/project_index.rs` |
| `00_nucleo/prompts/prompt-walker.md` | `03_infra/prompt_walker.rs` |
| `00_nucleo/prompts/rules/alien-file.md` | `01_core/rules/alien_file.rs` |
| `00_nucleo/prompts/rules/context-erasure.md` | `01_core/rules/context_erasure.rs` |
| `00_nucleo/prompts/rules/dangling-contract.md` | `01_core/rules/dangling_contract.rs` |
| `00_nucleo/prompts/rules/decision-ownership.md` | `01_core/rules/decision_ownership.rs` |
| `00_nucleo/prompts/rules/forbidden-import.md` | `01_core/rules/forbidden_import.rs` |
| `00_nucleo/prompts/rules/impure-core.md` | `01_core/rules/impure_core.rs` |
| `00_nucleo/prompts/rules/infrastructure-error.md` | `01_core/rules/infrastructure_error.rs` |
| `00_nucleo/prompts/rules/multi-prompt-header.md` | `01_core/rules/multi_prompt_header.rs` |
| `00_nucleo/prompts/rules/mutable-state-core.md` | `01_core/rules/mutable_state_core.rs` |
| `00_nucleo/prompts/rules/orphan-prompt.md` | `01_core/rules/orphan_prompt.rs` |
| `00_nucleo/prompts/rules/prompt-drift.md` | `01_core/rules/prompt_drift.rs` |
| `00_nucleo/prompts/rules/prompt-header.md` | `01_core/rules/prompt_header.rs` |
| `00_nucleo/prompts/rules/prompt-stale.md` | `01_core/rules/prompt_stale.rs` |
| `00_nucleo/prompts/rules/pub-leak.md` | `01_core/rules/pub_leak.rs` |
| `00_nucleo/prompts/rules/quarantine-leak.md` | `01_core/rules/quarantine_leak.rs` |
| `00_nucleo/prompts/rules/semantic-field-loss.md` | `01_core/rules/semantic_field_loss.rs` |
| `00_nucleo/prompts/rules/test-file.md` | `01_core/rules/test_file.rs` |
| `00_nucleo/prompts/rules/wiring-logic-leak.md` | `01_core/rules/wiring_logic_leak.rs` |

## Matriz dos parsers

| Linguagem | Extensões do walker | `prompt_header` canônico | `prompt_refs` para V15 local | Presente no domínio atual |
|---|---|---|---|---:|
| Rust | `.rs` | sim | todos os headers do bloco | 76 |
| TypeScript | `.ts`, `.tsx` | sim | sempre vazio | 0 |
| Python | `.py` | sim | sempre vazio | 0 |
| C | `.c`, `.h` | sim | sempre vazio | 0 |
| C++ | `.cpp`, `.hpp`, `.cc`, `.cxx`, `.hxx` | sim | sempre vazio | 0 |
| Zig | `.zig` | sim | sempre vazio | 0 |
| Go | `.go` | sim | somente o valor canônico | 0 |
| Java | `.java` | sim | somente o valor canônico | 0 |
| Elixir | `.ex`, `.exs` | sim | somente o valor canônico | 0 |

Para a V15 global, os nove parsers podem alimentar ownership por meio do
`prompt_header` já existente. Para preservar a V15 local (“um código, um prompt”), B2
precisa revelar a assimetria: TypeScript/Python/C/C++/Zig não publicam `prompt_refs`, e
Go/Java/Elixir não conseguem representar 2+ headers. Isso é `RED` de implementação,
não exceção normativa.

## Map/Reduce atual e seam mínima

O Map atual produz por arquivo
`(Vec<Violation>, Option<ParsedFile>, LocalIndex)`. `LocalIndex::from_parsed` reduz o
header canônico a apenas `Option<&str>`. O Reduce insere isso em
`ProjectIndex.referenced_prompts: HashSet<&str>`, apagando deliberadamente identidade,
path do consumer e multiplicidade. Esse conjunto basta para V7, mas não pode provar
injeção.

A seam mínima é ampliar a contribuição L1 pura com o par lógico
`(prompt_path, source_path)` e reduzi-lo deterministicamente em uma visão de ownership
que preserve todos os consumers distintos. L4 executa a nova verificação global após o
Reduce, como já faz com as demais regras globais. L1 recebe somente strings/paths
lógicos extraídos; não abre filesystem, não canonicaliza e não escolhe owner. V7 pode
continuar consumindo seu conjunto, evitando acoplar a regra existente à nova decisão.

## Classificação dos 13 compartilhamentos

| Classe observada | Prompts | Leitura causal |
|---|---:|---|
| composição/fachada de subsistema | `linter-core.md` | documento comum cobre módulos, adapters e wiring de camadas diferentes |
| fluxo vertical port/use-case/adapter | três `contracts/*`, `refinement-validator.md`, `segregated-materialization.md` | um L0 governa papéis arquiteturais distintos |
| família de operação | `fix-hashes.md`, `file-walker.md`, `sarif-formatter.md` | um L0 agrupa planejamento/apresentação e/ou adapters auxiliares |
| família de regra/entidades | `wildcard-saturation.md`, `external-type-in-contract.md`, `unsourced-constant.md`, `violation-types.md` | um L0 agrupa regra, suporte e tipos relacionados |

Todos são `RED` pela decisão humana 1:1. A taxonomia explica a origem; não legitima
exceções. A individualização exige decidir, por consumer, qual intenção é realmente
proprietária e qual conteúdo é apenas contexto comum. O P0104 proíbe criar uma relação
compartilhada e proíbe copiar L0 mecanicamente; portanto essa decisão semântica é
`SPEC-GAP` para D. A implementação da detecção global não está bloqueada por ela, mas o
self-lint biunívoco e o fechamento `READY` estão.

## Veredito A

- **RED confirmado:** 13 prompts governam 44 códigos; a V15 atual não os denuncia.
- **RED de parser/gate:** a V15 local não tem cobertura semântica uniforme nos nove
  parsers, embora todos publiquem o header canônico necessário à redução global.
- **Seam autorizável:** contribuição `(prompt, consumer)` em L1/IR, Reduce integral em
  L4 e decisão pura global em L1; nenhuma leitura de filesystem na regra.
- **SPEC-GAP:** conteúdo proprietário a separar nos 13 documentos compartilhados.
- **Horizonte finito:** no mínimo 31 novos prompts e 44 reassociações semanticamente
  revisadas; zero órfãos e zero consumers localmente ambíguos no baseline.

