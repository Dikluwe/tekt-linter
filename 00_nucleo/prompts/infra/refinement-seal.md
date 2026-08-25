# Prompt: I/O transacional do selo de refinamento
Hash do Código: a2863cc2

## Owner

`03_infra/refinement_seal.rs`, exclusivamente.

## Instrução

Ler manifesto/contrato/prompts confinados, verificar hashes exatos e publicar recibo
canônico somente após validação integral da política L1.

## Restrições

- rejeitar traversal, symlink, OID/hash inválido e mutação concorrente;
- falha não deixa arquivo parcial;
- publicação atômica não sobrescreve destino existente de modo silencioso.

## Critérios

Entrada válida produz bytes determinísticos; qualquer falha prévia mantém o destino
ausente/intacto e retorna erro observável.
