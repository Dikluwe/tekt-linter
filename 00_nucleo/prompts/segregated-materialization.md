# Prompt: política pura do selo de refinamento
Hash do Código: 309f7ad4

## Owner

`01_core/entities/refinement_seal.rs`, exclusivamente.

## Instrução

Modelar o manifesto validado, categorias de oráculo, produtores segregados, recibos e
decisão pura de selabilidade. `UNKNOWN` bloqueia onde o protocolo exige decisão.

## Restrições

- L1 não conhece TOML, JSON, Git, SHA ou filesystem;
- produtores obrigatórios devem ser distintos;
- score e ordenação são determinísticos e sem ponto flutuante ambíguo.

## Critérios

Manifestos completos selam; ausência de categoria, produtor repetido ou veredito
incompatível produz erro tipado antes de qualquer escrita.
