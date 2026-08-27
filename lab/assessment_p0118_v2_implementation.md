# Assessment P0118 — identidade de artefatos de teste em V2

Data: 2026-08-26  
Baseline funcional: `961030bb3520acfd74b53858cf55020bfebb8ffb`  
Passo selado: `bec57040c08009e300302a06f452b54fbab2e935`  
Estado: `APPROVE` após correção e reverificação independente

## Obrigação observada

- `Production` L1 sem cobertura produz exatamente uma V2 Error;
- `Production` L1 coberto produz zero V2;
- `Test` produz zero V2 independentemente do booleano de cobertura;
- conteúdo `assert`/helpers/`pytest.raises` não altera o papel;
- chamada/import de pytest em produção Python não prova cobertura;
- somente V2 usa a isenção por papel; V1/V5/V7/V15/V26 permanecem aplicáveis.

## Materialização

O walker classifica `SourceRole::Production | Test | Unknown` por linguagem e nome completo, antes
da leitura pelo parser. `SourceFile` transporta o papel, os nove parsers o copiam para
`ParsedFile` e `HasCoverage` o expõe à regra pura V2.

Nome ou stem não UTF-8 preserva `Unknown`; o walker não o converte em produção e emite
`SourceError::Unreadable` com razão explícita, que o pipeline projeta como V0 Fatal. Para C,
prefixo/sufixo de teste só vale com extensão `.c`; headers `.h` continuam produção.

A adjacência usa o mesmo domínio aceito pelo walker. Diretório, symlink e arquivo presente
em `excluded_files` não cobrem produção. Artefatos `Test` não consultam adjacência própria.

Python deixou de considerar chamadas `pytest`/`unittest` como cobertura de produção;
adjacência e isenção declaration-only permanecem.

## Oráculos independentes

O autor de oráculos, sem editar produção, criou:

- `tests/v2_source_role_contract_assessment.rs` — 5 casos;
- `tests/v2_test_artifact_classifier_assessment.rs` — 4 casos;
- `tests/v2_python_source_role_assessment.rs` — 3 casos;
- `tests/v2_walker_test_artifact_assessment.rs` — 7 casos;
- `tests/v2_concrete_parser_role_transport_assessment.rs` — 1 caso que executa os nove
  parsers concretos com papel sentinela.

Os oráculos produziram REDs materiais para `pytest.raises` em produção, arquivo excluído
contando como adjacente e alternativas físicas de linguagem. Todos estão verdes no estado
final.

## Mutação dirigida

Comando final:

```sh
cargo mutants --in-place --baseline skip \
  --file 01_core/rules/test_file.rs --file 03_infra/walker.rs \
  --re 'test_file.rs|classify_source_role|check_adjacent_test_with_exclusions|source_role == SourceRole::Production|SourceRole::Unknown' \
  --timeout 90 -- \
  --test v2_source_role_contract_assessment \
  --test v2_test_artifact_classifier_assessment \
  --test v2_walker_test_artifact_assessment \
  --test v2_python_source_role_assessment \
  --test v2_concrete_parser_role_transport_assessment
```

Resultado final corrigido: 47 mutantes gerados, 45 mutantes válidos mortos, 2 inviáveis em
compilação e zero sobreviventes válidos. Score do fragmento válido: `45/45 = 1.0`.

Uma campanha anterior foi interrompida enquanto oráculos eram ampliados e deixou um
marcador temporário do cargo-mutants. A busca mecânica detectou o resíduo, ele foi reparado
e a campanha final acima partiu de árvore sem marcadores.

## Gates

- `cargo fmt --all --check`: passou;
- `cargo test --all-targets`: passou, incluindo 651 unitários e todas as integrações;
- cinco assessments P0118: 20/20 passaram;
- ratchet P0109: 2/2 passou;
- `git diff --check`: passou;
- `--fix-hashes --dry-run`: `Nothing to fix`;
- auto-lint V1/V2/V5/V7/V15/V26: zero violações;
- smoke somente leitura na Tradução: exit 0 e zero V2, sem editar o consumidor.

## Prompts e limites

Os prompts proprietários de V2, `SourceFile`, `ParsedFile`, `HasCoverage`, walker e parser
Python foram alinhados e resselados pelo fluxo `--fix-hashes`. Programa e teste continuam
com ownership 1:1 independente; manifesto não adjacente e geração de `crystalline.toml`
ficam fora do P0118.

A execução usou papéis nominais separados, mas filesystem compartilhado. Portanto o
resultado é auditado e reproduzível sem atestação forte de isolamento histórico; não se
alega equivalência funcional geral do linter.

## Rejeição e correção

O primeiro veredito independente foi `REJECT`: nomes não UTF-8 caíam em `Production`,
headers C com nome de teste eram isentos e oito parsers careciam de ataque dinâmico ao
transporte. Os três pontos foram materialmente corrigidos e receberam oráculos de
regressão; esta rejeição permanece registrada em vez de ser apagada do histórico. A
reverificação independente aprovou o fragmento P0118 com os 20 oráculos, gates completos e
score de mutação válido `45/45`.
