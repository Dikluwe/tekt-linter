# Assessment 0031 — contenção forte do Git

**Estado:** PROTOCOLO CONGELADO — produção e gates proibidos até A
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
