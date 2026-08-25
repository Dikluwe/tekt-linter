# Prompt: artefato compartilhável Núcleo Tekt
Hash do Código: PENDENTE_P0105

## Contexto

`@prompt` é ownership 1:1. Invariantes realmente comuns precisam de relação distinta que
não gere código. ADR-0022 define `.tekt` como Núcleo Tekt compartilhável por prompts.

## Instrução

Implementar formato TOML v1 estrito, inventário confinado, grafo DAG determinístico,
referências hash-pinned em prompts, V26 e hash efetivo transitivo. L1 recebe somente IR;
L3 lê/parseia/hasheia; L4 agrega; L2 planeja apresentação e reparo.

## Restrições

- preservar hashes atuais de prompts sem núcleo;
- rejeitar campos desconhecidos, ciclos, missing e paths fora do namespace;
- não permitir `@prompt` apontando para `.tekt`;
- não executar conteúdo `.tekt`;
- não listar consumers dentro do núcleo;
- nunca escolher ordem por filesystem ou `HashMap`;
- não escrever antes de preflight do grafo integral.

## Critérios de verificação

- formato mínimo e limites cobertos por gate cego;
- DAG, ciclos, diamante, órfãos e permutações cobertos in-memory;
- vetores de hash completos e compatibilidade sem dependência;
- binário real distingue V5/V7/V15/V26;
- reparo dry/real compartilha plano e rollback;
- suíte integral e auto-lint não escondem P0104.

## Resultado esperado

Entidades/contratos L1, parser e walker L3, agregação L4, apresentação L2, V26 registrada,
fixtures e relatório P0105. Conversão dos compartilhamentos históricos fica para P0106.
