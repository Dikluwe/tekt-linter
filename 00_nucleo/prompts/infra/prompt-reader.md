# Prompt: FsPromptReader
Hash do Código: a11191c0

## Owner

`03_infra/prompt_reader.rs`, exclusivamente.

## Instrução

Implementar `PromptReader` lendo prompts regulares confinados à raiz L0 e calculando
SHA-256 sem a linha de metadado `Hash do Código`.

## Restrições

- path absoluto, traversal e symlink não podem escapar da raiz;
- erros externos retornam `None`/`false`, conforme a porta;
- a normalização do hash deve ser idêntica à usada pelo resselo.

## Critérios

Fixture regular produz oito hex; arquivo ausente/hostil falha sem panic; alterar apenas
o metadado excluído não muda o hash lido.
