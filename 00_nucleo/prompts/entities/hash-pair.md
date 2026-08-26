# Prompt: valores transacionais de par hash
Hash do Código: 00000000

Owner exclusivo: `01_core/entities/hash_pair.rs`.

Modelar `BijectivePair` e `PairSnapshot` como valores puros L1 compartilhados pelo
planejamento L2, pela persistência L3 e pela composição L4. Preservar integralmente paths,
hashes e bytes. Não executar I/O, decidir política de rollback nem importar camadas
superiores.

## Critério observável

Existe uma única definição de cada valor em L1; testes de transporte preservam clone,
igualdade, Unicode e bytes hostis; L2 e L3 dependem desses valores sem depender entre si.
