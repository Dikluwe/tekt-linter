# Assessment 0027/B1 — delta estrutural pós-P0096

**Papel:** B1, produção/testes/histórico em somente leitura  
**Baseline de integração P0096:** `3a5ffbec3968230f8fda29dff329c476fa73be39`  
**Baseline pós-merge P0097:** `7e358cff39ba24d5bba26de2fa0a3ba86ff7b379`  
**Resultado:** PASS factual com lacunas residuais preservadas  
**Autoridade derivada da produção ou dos testes:** não  
**Pareceres A/B2 consultados:** não

## Método e fronteira

Foram confrontados o inventário estrutural de P0096, a árvore de produção, os testes e o
histórico Git entre a integração P0096 e o baseline pós-P0097. A busca foi feita nos dois
sentidos: produtor → IR/efeito → consumidor e consumidor → contrato/produtor. Testes
existentes foram tratados como evidência de execução, não como autoridade nem como gate
independente por presunção.

O delta executável pós-P0096 contém somente:

- `03_infra/rs_parser.rs`: 22 linhas adicionadas e 13 removidas;
- `tests/rust_source_constant_identity_assessment.rs`: 113 linhas novas;
- `tests/rust_source_constant_context_assessment.rs`: 139 linhas novas.

Não houve delta executável em L1, L2 ou L4. Também não houve mudança de produção ou teste
em S1, S2, S3, S4 ou S6. Logo, o histórico não sustenta fechamento indireto dessas seams.

## Delta de S1–S6

### S1 — extractor/escritor de snapshot

**Produtor/efeito atual:** `03_infra/refinement_extractor.rs` permanece responsável, em
L3, por carregar contratos TOML, extrair observáveis, serializar JSON e escrever/renomear
snapshot. `04_wiring/main.rs` continua coordenando Snapshot, Seal e RefineRevisions.

**Consumidores:** os comandos revisionais em L4 e a extração a partir de conteúdo Git
continuam sendo os consumidores observáveis. A decisão pura de comparação permanece em
`01_core/entities/refinement.rs`.

**Delta:** nenhum arquivo da cadeia mudou desde P0096. Os riscos estruturais inventariados
— DTO/fronteira de entrada, distinção de falhas, cotas e semântica de publicação atômica
— não receberam nova confrontação independente. `PASS` apenas para a constatação de
ausência de mudança; nenhum fechamento adicional é inferível.

### S2 — refinamento Git/subprocesso

**Produtor/efeito atual:** `03_infra/git_refinement.rs` permanece como L3 concreto para
resolução de repositório/objetos, `ls-tree`, `cat-file`, subprocessos, timeout e kill.
Seal e RefineRevisions continuam coordenados em L4.

**Consumidores:** conteúdo extraído alimenta snapshots/fatos e finalmente
`compare_refinement` em L1; os testes `git_refinement_assessment` e `refinement_cli`
continuam exercitando Git e CLI reais.

**Delta:** nenhum. Não apareceu port injetável de subprocesso nem gate hostil simulado;
testes com Git real continuam evidência histórica/acoplada. Tratar esses testes como prova
independente seria `GATE-DEFECT`.

### S3 — manifesto, recibo e selo

**Produtor/efeito atual:** `03_infra/refinement_seal.rs` continua implementando hashes,
recibos, serialização e publicação do selo em L3; o bloco Seal em `04_wiring/main.rs`
continua compondo política e exit em L4.

**Consumidores:** CLI de materialização segregada e fluxo revisional permanecem as
confrontações fim a fim existentes.

**Delta:** nenhum. O fechamento histórico não foi reaberto por alteração estrutural, mas
os resíduos já registrados — limites/regularidade de leitura, fsync de diretório,
preservação explícita de modo e composição L4 — também não foram eliminados. A presença
de `file.sync_all()` continua significando sync do arquivo, não do diretório.

### S4 — pipeline principal

**Coordenador/efeitos atuais:** `04_wiring/main.rs` ainda concentra montagem dos adapters,
descoberta, `run_pipeline`, `run_checks`, Rayon, apresentação, mutações, reruns e decisão
de exit. L1 continua proprietário das regras/decisões, L2 dos planos e formatação, L3 dos
efeitos concretos e L4 da composição.

**Consumidores:** CLI e exit status são a saída observável; regras recebem `ParsedFile` e
índice produzidos pela composição.

**Delta:** nenhum. Não surgiu gate direto da composição completa nem decomposição nova
das precedências e reruns. Os gates de componentes e CLIs transitivos não provam, por si,
a política global de L4.

### S5 — parsers concretos para IR comum

**Produtores atuais:** os nove adapters L3 permanecem Rust, TypeScript, Python, C, C++,
Zig, Go, Java e Elixir. `ParserSet`/`LanguageParser` em L1 define o port e a decisão total
de roteamento; `MultiParser` em L4 injeta os nove adapters; `ParsedFile` é o IR consumido
pelas regras L1.

**Delta comprovado em S5/Rust/números:** `03_infra/rs_parser.rs` passou a limitar a
projeção `FunctionNumberLiteral`/`NegativeLiteral` a corpos de função, suprimir patterns,
ranges e ancestrais `macro_invocation`, preservar snippet sem `trim`, usar coluna
numérica 1-based e manter ordem/multiplicidade. Os dois novos gates chamam o adapter Rust
diretamente por `LanguageParser` e filtram somente essas duas variantes. Eles confrontam:

- identidade, sinal, sufixo, coluna UTF-8 em bytes, preorder, repetição e multiplicidade;
- exclusão de formas não numéricas, contextos fora de função, macro, range e pattern;
- estabilidade diante de whitespace/comentários e falha sintática sem IR parcial.

Essa é uma cadeia direta fonte Rust → parser L3 → `ParsedFile.constants`. O delta não
alterou `SourceConstant`, `ParsedFile`, V21 ou V22. A busca reversa confirma que os dois
consumidores diretos continuam:

- `01_core/rules/unsourced_constant.rs` (V21);
- `01_core/rules/provenance_inventory.rs` (V22).

Os gates novos não usam esses classificadores como oráculo. Os gates sintéticos dos
consumidores continuam provando decisões L1 sobre IR fornecido, e não extração Rust.

**Limite factual do fechamento:** somente a projeção numérica Rust recebeu esse novo gate.
Não foram confrontados por esses gates `citation`, `is_test_origin`,
`function_return_type`, `is_in_binary_scaling`, `context_var`, `geometric_sink`,
`is_in_data_table`, nem as variantes não numéricas de `ConstantKind`. A associação/janela
de citações permanece no mesmo coletor, mas fora do recorte. Também não houve mudança ou
gate novo nos oito outros parsers. Promover qualquer desses elementos a fechado por
associação seria `RED` de cobertura.

**Possível omissão gramatical residual:** a supressão de macro é reconhecida por ancestral
com kind textual `macro_invocation`; os gates cobrem `emit!(5)`/`value!(47)`, não uma
matriz de variantes gramaticais de macro. Isto limita a generalização do resultado, mas
não constitui por si nova seam fora de S5.

### S6 — preflight/precedência CLI ampliada

**Produtor/coordenação atuais:** `02_shell/cli.rs` continua proprietário em L2 de parsing
estruturado, `validate_args`, `EnabledChecks`, apresentação e `should_fail`;
`04_wiring/main.rs` continua aplicando ordem de subcomandos, preflight, scans, fixes,
updates, `emit_resolution`, quiet e exit em L4.

**Consumidores:** a CLI e seu exit status permanecem a fronteira observável. Unitários e
gates de apresentação confrontam decisões isoladas; CLIs exercitam combinações
transitivas.

**Delta:** nenhum. A matriz global de precedência clap/config/subcomando/dry-run/quiet/
emit-resolution não ganhou gate dedicado. Inferir sua cobertura integral de testes
isolados ou CLIs parciais seria `GATE-DEFECT`.

## Busca reversa e travessias Tekt

| Fluxo | Proprietários observados | Situação estrutural pós-P0097 |
|---|---|---|
| contrato/fonte de refinement → fatos → veredito | L3 extractor/Git → L1 `compare_refinement` → L4 CLI | sem delta; S1/S2 permanecem distintas pelo efeito de origem |
| selo/recibo → publicação | L1 entidades → L3 seal → L4 Seal | sem delta; S3 não reabre nem amplia fechamento |
| arquivo → parser → IR → regra → diagnóstico/exit | L3 parser → L1 IR/regra → L2 apresentação → L4 pipeline | delta apenas na projeção numérica do parser Rust; S4 não herda fechamento |
| argumentos/config → checks/ação → saída | L2 CLI → L4 precedência → L3 efeitos → L2 saída | sem delta; S6 continua transversal |

Não foi encontrada nova seam estrutural fora de S1–S6 no delta. O novo helper
`has_ancestor_kind` é detalhe interno do mesmo produtor L3 e os dois arquivos de teste são
gates da sub-seam S5/Rust/números, não novos componentes de produção. Não houve migração
de decisão para camada incompatível: a interpretação de AST permaneceu em L3, regras em
L1, apresentação em L2 e coordenação em L4.

## Evidência hash-pinned deste parecer

| Unidade lida | SHA-256 |
|---|---|
| `03_infra/rs_parser.rs` | `d9da19406f2efb91a8a01169a95e813217f4715bec21690127c38f4973b17ecc` |
| gate identidade/contexto numérico | `cc596d876bcfbcacbf3688bd9a1aa1b875928bcb53f77abe1544af5100fd3dcb` |
| gate exclusões/erro sintático | `dc50ebb0b3913a108c09a0b5e2dc81d6918ac622d7ab8a4159340a1c818237dd` |
| contrato `SourceConstant` | `242ced79ac737b9950f2bf98630b05b5a25830afc7751619e1591d2b6c58702c` |
| IR `ParsedFile` | `b7680a1fecf8b801e62f0d4fd9baa15f8d07e7d0a8c85069f9be9b3d1d835898` |
| consumidor V21 | `96e95623a0ae337c419c408fcd4c0b6a4c7a55410a977eb78e54db189ef3398a` |
| consumidor V22 | `2f54fb50b3b1c97037262a65a7f345712e2ae80ab07871487e5a522380797a6a` |
| extractor snapshot/refinement | `1c4fd25935565af3622d92dde8a809693e05374b6598560d8cd45954620225ba` |
| Git/subprocesso | `e81bddea5ded6d3239ad462a6d9294248986f36416b14b539b1366748464163c` |
| seal/recibo | `d470698f260a45a44b68e4d8c21e886be63c28a6f047f9b1fecb43d28c907513` |
| pipeline/composição L4 | `ad290f1d89543300840a29deb710e6088781df7ae23135cc3c65647fb91b7f12` |
| CLI/preflight L2 | `28738882ad62db457323549817b647fd4de99ffa1e188001be4c748cfd5f2c02` |

## Conclusão B1

`PASS` factual: S1–S6 continuam cobrindo o universo estrutural observado, sem seam nova
comprovada pelo delta. P0097 reduziu exclusivamente S5/Rust/projeção numérica e criou
gates diretos proporcionais a esse recorte. S1–S4, S6, oito parsers, variantes não
numéricas, citações e demais campos estruturais não mudaram e não podem herdar o mesmo
fechamento. Este parecer não atribui destino final, prioridade ou candidato P0099.
