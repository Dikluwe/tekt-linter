# Recibo de mutação P0117

Data: 2026-08-26  
Baseline congelada: `4457695`  
Escopo: classificação tri-state de acesso a prompts e preservação no planejamento de
`fix-hashes`.

## Confinamento físico

```sh
cargo mutants --in-place --baseline skip \
  --file 03_infra/prompt_io.rs --timeout 90 -- \
  --lib --test prompt_io_hash_assessment --test v1_absolute_path_assessment
```

Resultado observado: 17 mutantes gerados, 17 mortos, zero sobreviventes.

## Contrato, cache e projeção V0/V1

```sh
cargo mutants --in-place --baseline skip \
  --file 01_core/contracts/prompt_reader.rs \
  --file 01_core/rules/prompt_header.rs \
  --file 01_core/rules/infrastructure_error.rs \
  --file 03_infra/prompt_reader.rs --timeout 90 -- \
  --lib --test v1_absolute_path_assessment \
  --test lineage_header_classifiers_assessment \
  --test infrastructure_error_projection_assessment \
  --test prompt_io_hash_assessment
```

Resultado final, após o ataque `Present → remoção → Missing`: 23 gerados, 10 mutantes
válidos mortos, 13 inviáveis em compilação e zero sobreviventes válidos.

Os testes exercitam as duas direções de mudança do filesystem depois da primeira resolução:
`Missing → criado` permanece `Missing`, e `Present → removido` permanece `Present` com o
hash congelado.

## Planejamento de fix-hashes

```sh
cargo mutants --in-place --baseline skip \
  --file 02_shell/fix_hashes.rs --timeout 90 -- \
  --lib --test fix_hashes_planning_assessment \
  --test fix_hashes_execution_assessment
```

Resultado observado: 35 gerados, 11 mortos, 4 inviáveis e 20 sobreviventes. As mutações
de `FixHashesUseCase::plan` foram mortas; o teste
`plan_preserves_prompt_infrastructure_failure` observa a nova obrigação. Os sobreviventes
listados pela ferramenta pertencem a `FixBatchError`, `FixBatchResult`, `plan_bijective`,
`validate_bijective`, `describe_unavailable` e `format_results`. Por isso este recibo não
atribui score total nem zero sobreviventes ao arquivo `fix_hashes.rs`.

## Limite da evidência

Este é um recibo reproduzível dos comandos e resultados observados, não prova de isolamento
histórico entre autoria e verificação. O filesystem foi compartilhado pelos participantes.
