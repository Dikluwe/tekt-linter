# Assessment 0031 — contenção forte do Git

**Estado:** FECHADO `BLOCKED` — C não executado; F05 aberto
**Data:** 2026-08-25
**Passo:** P0102
**Baseline funcional:** `c681c7a8fd419c48683553f88b6a3bf391f2032b`
**Envelope operacional:** `af8250e9a538fd50554b3e545efcb3ce421ba208`

## Insumos hash-pinned conferidos

| Unidade | SHA-256 |
|---|---|
| P0102 | `77407b9b97633292c149bc36f0aa2b0f23c138d5e0d9609d03999ed812b730a5` |
| Assessment P0101 | `c58082915d0d02576c3664d9c1e9757dc43448d173f92c7ffe7d442a478f35fc` |
| relatório P0101 | `798d67554ddb559447c2473c7f6fb5b98ce9cbc852320e31b544db3d346ab36d` |
| contrato Git | `9ab972915e8f21e6c0fc323686d507fb2cb4b590de6d987b454e05642f167818` |
| arquitetura Tekt | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| protocolo segregado | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| produção L3 | `42bab723efa948b3025a70154d2087493d7104fa9186ba02fc5347e6a4614d65` |
| wiring L4 | `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` |
| gate lifecycle P0101 | `c946032e31c083d051705f4bfe3ff66c8d03d5894822a99cb47ab8fe7af615f0` |
| gate objetos P0101 | `8d612adac31fc168b2904b4ef32c82f34573a6e753780edf9e9a8a35e4a33925` |
| gate Windows P0101 | `7c1541991e8b303767c3d5c0e1b8c1f89599cb4e0cd97775713bc8feed59fc35` |

Todos os hashes conferem. O único delta após o baseline funcional é o envelope P0102.

## Infraestrutura observada

- targets Rust instalados: `x86_64-unknown-linux-gnu` e `x86_64-unknown-linux-musl`;
- `wine` e `wine64` ausentes;
- instalação, download, emulação e acesso a host Windows não foram autorizados.

B3 pode ser materializado, mas não pode receber `PASS` nesta execução. R2 continuará
`BLOCKED` salvo mudança verificável de infraestrutura durante P0102.

## REDs de entrada

| ID | Estado inicial |
|---|---|
| R1 | `PASS`, regressão proibida |
| R2 | `RED / BLOCKED` |
| R3 | `PASS` Unix, regressão proibida |
| R4 | `RED`; B2 P0101 possui `GATE-DEFECT` |
| R5 | `RED` Unix e `BLOCKED` Windows; B1 P0101 possui `GATE-DEFECT` |

## Segregação

A é somente leitura. B1, B2 e B3 começam apenas após o mapa A congelado. Cada gate usa
arquivo e fixture exclusivos e não lê produção. B4 apenas reexecuta gates históricos.
Produção reabre somente após RED reproduzível, ou após `SPEC-GAP` ser resolvido em passo
separado. D é somente leitura e não promove teste pulado ou cleanup da fixture a prova.

Sem merge, push ou fechamento de F05 antes das cinco linhas `PASS`.

## Resultado A

Mapa congelado em `09425ec`; SHA-256
`0f1c2cecd490d7c86ffa8566681ccf18fdbd5417ab8a1e4d4feefecd5947f326`.

- SG-1 `SPEC-GAP`: contenção Unix depois de `setsid` exige decisão de mecanismo,
  plataforma, privilégio e cleanup.
- SG-2 `SPEC-GAP`: contenção forte do object database exige decidir entre staging,
  sandbox ou backend controlado, com política de I/O/budget/portabilidade.
- Job Object é normativo, mas runtime Windows continua indisponível.
- deadline de readers é implementável, mas não fecha isoladamente R5.

A autorizou somente B1–B4. Produção permanece proibida nesta execução.

## Gates P0102 congelados

| Gate | SHA-256 | Resultado inicial | Classificação |
|---|---|---|---|
| B1 `tests/git_refinement_transient_object_race_assessment.rs` | `628d6286e119a0e562169c4067758b747ad0f347de8f00d283148e0152606ae4` | 3 controles PASS; 3/3 confrontos RED | R4 `RED` |
| B2 `tests/git_refinement_session_escape_assessment.rs` | `261c2d76d71fe6d2164d169848c8caf62191b553434f9f8d75d7e37084f316fb` | 0/4 PASS | R5 `RED` |
| B3 `tests/git_refinement_windows_job_v2_assessment.rs` | `f2c0c89068721ddfb86ec4f62411e49a4885eb1b3fe56fcf1b7e6c8f2909ea50` | Linux 0 testes; Windows compile-RED por seam ausente | R2 `RED / BLOCKED` |
| B4 gates P0101 rota/protocolo/stream | hashes P0101 preservados | 3/3 + 7/7 + 4/4 | R1/R3 continuam `PASS` |

B1 sincroniza troca e restauração antes do retorno em loose object, fanout e par
pack/idx. Marcadores provam a corrida; nos três casos o adapter retorna `Ok` contendo
exatamente a sentinela externa, em vez de `ContainmentFailure`.

B2 confronta `setsid` com pipes abertos/fechados, escape durante timeout e cadeia com
intermediário encerrado. Três casos excedem watchdog externo de 15 segundos; o caso que
fecha pipes retorna `ProcessFailure` em vez de `ContainmentFailure`. A fixture registra
identidade `(PID, starttime)` antes do cleanup e nenhum processo próprio restou vivo.

B3 não constitui prova runtime: no Linux descobre zero testes. Em Windows, a compilação
permanece intencionalmente RED até existir seam privada para injetar falhas separadas de
criação, configuração e atribuição do Job. O gate ainda precisará ganhar os seis cenários
runtime após a seam; não pode ser promovido diretamente a PASS.

B1 não apresentou `GATE-DEFECT` no confronto D. B2 preserva RED causal, mas possui dois
defeitos parciais: depois de `recv_timeout`, cleanup e `worker.join()` não têm deadline
próprio; e a cadeia publica somente um PID, sem inventariar todos os membros. B3 é
`GATE-DEFECT` como gate runtime: `cfg(windows) + compile_error!` prova seam ausente, mas
não materializa os seis cenários Windows exigidos. Skip/compile-RED não substitui runtime.

SG-1 e SG-2 bloqueiam C conforme o protocolo; nenhuma produção foi alterada parcialmente
para contorná-los.

## Confronto D e fechamento

D confrontou `f2ebbfb` e confirmou:

- hashes L0 e gates conferem;
- delta desde `c681c7a` contém somente documentação e gates; L1/L2/L3/L4 e manifests
  produtivos permanecem byte a byte iguais;
- B1 reproduz 3 controles PASS + 3 REDs com sentinela externa publicada;
- B2 reproduz 0/4, com três watchdogs e um `ProcessFailure` indevido;
- B3 executa zero testes no Linux;
- B4 preserva rota 3/3, protocolo 7/7 e stream 4/4;
- nenhum processo próprio da fixture permaneceu vivo após a execução.

Matriz final: R1 `PASS`; R2 `RED/BLOCKED`; R3 `PASS` Unix; R4 `RED`; R5 `RED` Unix e
`BLOCKED` Windows. A regressão produtiva dirigida passou: 630 testes de biblioteca,
Git histórico 6/6, objetos P0101 7/7, timeout 4/4 e CLI 10/10. V5/V6/V7/V12 estão
limpos e o reparador V5 responde `Nothing to fix`. A suíte workspace integral permanece
deliberadamente RED pelos novos gates congelados.

Resultado: **P0102/F05 `BLOCKED`**. Sem merge ou push.
