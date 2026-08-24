# Auditoria adversarial final — P0090 / partição de resultados do walker

**Data:** 2026-08-24  
**Baseline:** `40c374d572dcb0c674b807ebe498f1fb12c1b650`  
**Materialização auditada:** `77927bf`  
**Veredito:** `READY WITH RESIDUAL AUDIT`

## Escopo e integridade causal

O Assessment 0019 referencia insumos L0 por SHA-256 e todos os hashes conferem com os
bytes atuais: protocolo P0090 `eb0d4ae5...`, contrato `file-provider` `1574ce78...`,
`linter-core` `2e5da1cf...` e ADR-0004 `33380a0b...`. O gate congelado tem SHA-256
`a64c4a50...` e permanece separado do reparo oficial.

O delta causal é coerente com Tekt: ADR-0004 e os prompts autorizam a partição pura em
L1; `01_core/contracts/file_provider.rs` publica a função; L4 importa e chama a função
nos três pontos de coleta e não conserva loop, classificação ou cópia concorrente.
Não há acesso de L1 a filesystem, ambiente, relógio, rede ou processo.

## Confronto do contrato

- A implementação usa dois `Vec::new()` e um único `for` sobre o iterador tomado por
  valor. Em Rust, esse percurso equivale às chamadas normativas de `next`: uma por item
  e uma para o primeiro `None`, sem chamada pós-EOF.
- Não há `Clone`, replay, segunda coleta, reconstrução de campos, normalização ou
  consulta a `size_hint`.
- `Ok` e `Err` são movidos por `push` para subsequências distintas, preservando ordem,
  cardinalidade, duplicatas e bytes/campos observáveis.
- Um `Err` não produz retorno antecipado. Itens posteriores continuam sendo consumidos.
- O pipeline posterior e sua projeção fail-fast permanecem inalterados; somente a
  localização da partição mudou de helper privado L4 para contrato público L1.

## Gates e regressão

Verificações read-only executadas:

- `cargo test --test walker_result_partition_assessment`: 2/2 PASS;
- `cargo test --all-targets`: 628 unitários, 83 fixtures e todos os gates de integração
  PASS, incluindo os assessments 0001–0018;
- `cargo run --quiet -- . --fix-hashes --dry-run`: `Nothing to fix`;
- `git diff --check 40c374d..77927bf`: limpo.

O iterador hostil do gate entra em pânico se `size_hint` for consultado ou se `next`
for chamado após EOF, e registra exatamente `itens + 1` chamadas. Também cobre vazio,
apenas sucessos, apenas erros, alternância, duplicatas, itens posteriores a erros e
conteúdo hostil/Unicode sem normalização.

## Auditoria do delta

Há 11 arquivos Rust alterados. Destes, 9 são estritamente header-only, atualizando
`@prompt-hash` de `e042f8ff` para `4f04c0c9`: `01_core/contracts/mod.rs`,
`01_core/rules/mod.rs`, `02_shell/mod.rs`, `02_shell/n16_summary.rs`,
`03_infra/config.rs`, `03_infra/elixir_parser.rs`, `03_infra/go_parser.rs`,
`03_infra/java_parser.rs` e `03_infra/mod.rs`.

Os outros 2 arquivos Rust têm o delta funcional autorizado: a nova função em
`01_core/contracts/file_provider.rs` e a remoção/importação do helper em
`04_wiring/main.rs`. Portanto, uma alegação literal de “11 arquivos header-only” seria
imprecisa; a formulação correta é “11 arquivos Rust, 9 header-only e 2 funcionais”.

## Residual audit

1. O gate demonstra a semântica pública da partição, mas não prova formalmente ausência
   de efeitos futuros em L1; isso continua coberto por revisão arquitetural e V4.
2. O teste black-box instrumenta um iterador concreto hostil, não todo iterador Rust
   possível; o argumento geral depende também da inspeção direta do laço mínimo.
3. Corrigir, em qualquer fechamento que use essa contagem, a frase “11 arquivos
   header-only” para a contagem exata acima. É residual documental e não um RED.

Não foram encontrados `RED`, `SPEC-GAP` ou `GATE-DEFECT` remanescentes que bloqueiem o
fechamento. P0090 pode ser encerrado como `READY WITH RESIDUAL AUDIT`, antes de qualquer
merge.
