# Relatório P0071 — fechamento local do refinamento entre revisões

**Data:** 2026-08-24  
**Branch:** `codex/refinement-validator`  
**Base da execução:** `58ce52b740dc3598b079ecec7be725422b9e7597`  
**Master preservado:** `4c9583d8860236788ceaeb05d795d58453d0fc69`

## Resultado

O gate local passou. A execução parou antes de merge e instalação, como exigido pelo
P0071. A B2 agora possui fixtures negativas para as principais fronteiras locais e
mantém a mesma semântica do fluxo `snapshot + refine`.

## Matriz executada

| Caso | Resultado |
|---|---|
| revisão preservada | `PRESERVED`, exit 0 |
| revisão regressiva em fixture | `VIOLATED`, exit 1, com testemunha |
| observável ausente | `UNKNOWN(missing-observable)`, exit 2 |
| ref inexistente ou `--help` | erro fechado, exit 2 |
| diretório não Git | erro fechado, exit 2 |
| working tree sujo | status e bytes preservados nos três vereditos |
| B2 versus exportação + B1 + A | stdout e exit code idênticos |
| symlink observável | `UNKNOWN(unsupported-parser)`, sem seguir alvo |
| gitlink/submódulo | `UNKNOWN(unsupported-parser)`, sem inicialização |
| hooks e filtros sentinela | marcadores não executados |
| processo bloqueado | morto pelo timeout testável |
| blob acima de 4 MiB | `UNKNOWN(budget-exhausted)`, exit 2 |
| soma acima de 32 MiB | `UNKNOWN(budget-exhausted)`, exit 2 |
| 513 paths | rejeição por budget, exit 2 |

O teste de budget revelou que a implementação inicial convertia excesso de blob em
erro global. Foi corrigida para preservar a semântica ternária: contrato válido com
evidência grande demais produz `Unknown(BudgetExhausted)`.

## Casos reais

```text
18a9b6e → 0f4e5df: PRESERVED, exit 0
f8a0dae → 0f4e5df: UNKNOWN(missing-observable), exit 2
```

Ambos imprimiram os OIDs completos e informaram que o working tree foi ignorado.

## Validação final

- 581 testes unitários passaram;
- 83 fixtures gerais passaram;
- 10 testes CLI passaram;
- `target/debug/crystalline-lint .` terminou com exit 0;
- `git diff --check` passou;
- índice e stash permaneceram vazios;
- `Cargo.toml` e `Cargo.lock` não mudaram;
- nenhuma biblioteca Git foi adicionada.

O único warning de compilação observado é o `print_tree` morto preexistente em
`03_infra/ts_parser.rs`; o P0071 não o criou.

## Evidência ainda ausente

Não foram testados neste host:

- Windows e macOS;
- repositório SHA-256;
- alternates;
- shallow clone;
- partial clone com objeto prometido ausente;
- ausência de rede observada por sandbox/servidor sentinela dedicado;
- path realmente não UTF-8.

Foi adicionada ao CI uma matriz `ubuntu-latest`, `macos-latest` e `windows-latest` que
executa `cargo test --test refinement_cli`. Ela prepara a coleta da evidência de
portabilidade, mas só conta como prova depois de executada pelo provedor remoto.

LFS foi coberto indiretamente pela leitura de blobs crus e pela não execução de
filtros, mas ainda não por instalação real de `git-lfs`. Portanto não se declara essa
integração como provada.

## Estado Git medido

No início: índice e stash vazios; HEAD `58ce52b`; branch dedicado; master em `4c9583d`.
O diff rastreado inicial tinha hash `9990f286…` porque o índice do núcleo já referia o
P0071 ainda não rastreado. Após a execução, mudanças permanecem limitadas ao passo,
índice do núcleo, relatório, adapter e fixtures; nenhum comando B2 alterou HEAD, branch, índice
ou stash do repositório analisado.

## Gate

Gate local verde, mas portabilidade externa não demonstrada. Conforme a parada do
P0071, não foram feitos merge no `master`, build/instalação release nem substituição do
binário do sistema. O próximo ato exige decisão humana sobre executar CI externo ou
aceitar as lacunas documentadas antes da integração.

## Integração local autorizada

Após nova autorização humana, o branch foi integrado por fast-forward no `master`, que
passou a apontar para `9d5b8ef22e1363d915c31a50542e358c132043d9`. A suíte e o
auto-lint foram repetidos no estado integrado com as mesmas contagens e sem falhas.

O release integrado foi construído e passou o smoke test `18a9b6e → 0f4e5df` com
`PRESERVED`. Hash SHA-256 do release: `60a4b99a…`. O binário instalado continua sendo
o anterior, hash `8ff2246c…`; nenhuma substituição externa foi feita. A matriz CI está
configurada, mas ainda depende de publicação/execução remota.
