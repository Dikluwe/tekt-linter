# Assessment 0025/A — cobertura histórica

**Papel:** A, leitura documental segregada
**Resultado:** PASS
**Produção/testes lidos:** não
**Recomendação de P0097:** proibida e não realizada

## Matriz

| Assessment | Seam | Cobertura | Resíduo/fronteira reservada |
|---|---|---|---|
| 0001 | Git imutável, revisões e blobs | CLOSED | — |
| 0002 | entidades, severidade e redução ProjectIndex | CLOSED | — |
| 0003 | ordenação, texto, SARIF e `should_fail` | CLOSED_WITH_RESIDUAL | path Windows condicional |
| 0004 | crate registry, ownership e V22 | CLOSED | — |
| 0005 | config e descoberta walker | CLOSED_WITH_RESIDUAL | EACCES real não reproduzido portavelmente |
| 0006 | prompt reader/walker/snapshot/hash/writers | CLOSED_WITH_RESIDUAL | frescura e TOCTOU externo |
| 0007 | V5/V6/V7 | CLOSED_WITH_RESIDUAL | `describe()` V6 não expõe todo desempate |
| 0008 | V2/V8/V10/V11 | CLOSED | — |
| 0009 | V1/V15 linhagem | CLOSED | — |
| 0010 | V3/V9 imports/evidência | CLOSED | resolução upstream fora da seam |
| 0011 | V12/V13 | CLOSED | — |
| 0012 | V4/V14 | CLOSED | — |
| 0013 | fechamento transversal 0001–0012 e materialização/refinement | CLOSED_WITH_RESIDUAL | selo/infra/wiring fechados com gate 16/16; rustfmt legado, Typst e matrizes condicionais |
| 0014 | V17–V20 L1 | CLOSED | — |
| 0015 | V23–V25 L1 | CLOSED | integração/extrator/config/agregação PARTIAL |
| 0016 | V16 L1 | CLOSED | parser `DecisionExpr` e wiring config PARTIAL |
| 0017 | V21 L1 + frescura L3 | CLOSED_WITH_RESIDUAL | associação parser, plataforma e política L4 PARTIAL |
| 0018 | projeção V0/PARSE | CLOSED_WITH_RESIDUAL | convenção global 0:0 e integração ampliada |
| 0019 | partição Ok/Err walker | CLOSED_WITH_RESIDUAL | iterator ilegal após `None` |
| 0020 | roteamento/composição MultiParser | CLOSED_WITH_RESIDUAL | não prova fidelidade dos parsers concretos |
| 0021 | `update-snapshot` | CLOSED_WITH_RESIDUAL | independência cognitiva não provada pelo Git |
| 0022 | `fix-hashes` | CLOSED_WITH_RESIDUAL | rollback composto, exit parcial e falha entre escritas |
| 0023 | summary N16 | CLOSED_WITH_RESIDUAL | sem gate CLI versionado; overflow sintético |
| 0024 | loader JSON/TOML de refinamento | CLOSED_WITH_RESIDUAL | sentinel, TOCTOU e fingerprint parcial |

## Seams não fechadas por associação

1. extração e integração semântica V23–V25: `PARTIAL`;
2. parser `DecisionExpr` e config/wiring V16: `PARTIAL`;
3. associação do parser e política L4 ampliada V21: `PARTIAL`;
4. política global de localização 0:0 e integração V0: `INDETERMINATE/PARTIAL`;
5. atomicidade composta/exit parcial de fix-hashes: residual, novo L0 se reaberta;
6. CLI versionado N16: residual;
7. garantias fortes TOCTOU/fingerprint: residual, novo L0 se ampliadas;
8. rustfmt global e artefatos Typst: histórico/fora do universo funcional Rust.

A materialização/refinement do selo, incluindo contrato, entidade, infraestrutura e
wiring, foi fechada pelo Assessment 0013 com `segregated_materialization_cli` 16/16. Sem
mudança funcional ou novo consumidor demonstrado, essa seam é `CLOSED_WITH_RESIDUAL`, não
`UNAUDITED`, e não pode ser candidata por simples renomeação da fronteira.

Não foi encontrada mudança posterior que contradiga um fechamento. Classificadores L1
fechados não podem ser reabertos ao inventariar integração upstream/downstream.

## Manifesto hash-pinned usado

```text
0001 5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17
0002 daac95d8d06faf81d9204ee0b824407ed8c853af7a6d70b4e9b03b908819501e
0003 1e551c6ca8c53a24ac74fc248cd1a930d040657ba267720595d166c8701235a6
0004 208efcb6d79bd64d0a50e14f03bcb9320cf07543a273a0eebd2698f40b2c55a1
0005 373da1016d75d4cea79ed9c3f69d9a28069e876b848ec7cbd654eca23c57423f
0006 df3b8bcf1f14f1989c978efe620a55a822512a8ccdbf6e5ea35d3d918d636567
0007 0a261e805d0b1359840aec431dfed163d1efd7f962141270a8dbd962062b1f48
0008 b1eb77b93633881d12f8da58ad82e6af992653914b752f005a4997303549956c
0009 865b6c70669e041aad98484594070ac242384501a0641efdcf103d48418068b8
0010 f7400a0d9784919a9329edd58f39bc88933049782e9069e7189d6bda8d88f9cb
0011 bf9cb415c93156d72fc77e0c2984be15a2727513864767dd1f351a6977eceab9
0012 2e55b7d0d408ea343ce0392c5630dde1739b4ca04a56e03353429018b9f3a34a
0013 3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9
0014 fb6bd7aa96a08b2988a29a4e2ed4c7821c0ed1ab8aa8b36e8917f064b2c1d6c1
0015 0f45e283dc741c627d9482ab1a3486df6905cfb4d02a75166a12de74594832d0
0016 d32a31f8774587711a34ba8ef1ea52a4a3afb028e3fe80aea82ea9386bc0577e
0017 fb3024c255789d409b73d2d8e5e138c753c8a01e9c986190ff727147081e584b
0018 1981996710a2cca18681b2948dde55b99095668d067a674d7b87e9a3b1854f34
0019 66e5636851df672b4c779634ead399f4e2eaf80bfe3055fd38460dd1ef93d953
0020 49ab7b126652eda0dc75e9e3e64ee9888fa479611c693b8acfc9122ea51e6297
0021 98a905f3ae1e14e4a38a54a6a56179202303eb63115a9234fd152239b990ad29
0022 eb4c880fe4abe43f8799bb2e6e3a7473d88381fedf1fe7e1b6d77b58605c7fbe
0023 29087e323f2fac23c12cbcd4b7feea309cbacb336b5f567b3fe43371d787594e
0024 6e32d9d8b798928c438bca3959b5de35bbd899a4dfed646860e82ffd4f51bb3c
```

Os relatórios P0072–P0095 pertinentes também foram recalculados; os hashes centrais
P0081, P0094 e P0095 coincidem com o Assessment 0025. Nenhum RED documental foi encontrado.
