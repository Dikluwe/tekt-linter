# ADR-0022 — Núcleos Tekt para contratos L0 compartilháveis

**Estado:** aceito
**Data:** 2026-08-25
**Decisor:** mantenedor humano
**Contexto:** P0104–P0105

## Contexto

P0104 tornou `@prompt` biunívoco: cada código produtivo possui um prompt proprietário e
cada prompt proprietário possui um código. O inventário encontrou invariantes legítimos
compartilhados entre vários códigos. Duplicar esses invariantes fabricaria ownership;
permitir prompt plural destruiria a bijeção.

## Decisão

Criar o **Núcleo Tekt**, artefato L0 declarativo `.tekt`:

- localização canônica `00_nucleo/prompts/_nuclei/**/*.tekt`;
- TOML 1.0 estrito, `tekt = 1`, `kind = "nucleus"`;
- contém claims atômicas e pode depender de outros núcleos;
- forma DAG e pode ser consumido por zero ou mais prompts;
- prompt referencia núcleo por path lógico e SHA-256 completo;
- código nunca referencia núcleo diretamente;
- núcleo não possui `Hash do Código` nem owner de produção;
- V26 valida formato, inventário, pins, DAG e órfãos;
- V1/V5/V7/V15 conservam suas responsabilidades.

Para prompt sem núcleo, V5 permanece bit a bit idêntica. Para prompt com núcleo, seu hash
efetivo inclui bytes normativos do prompt e os digests efetivos reais das dependências. O
digest efetivo de núcleo inclui seus bytes e dependências transitivas. Ciclos e leitura
incompleta não produzem digest e falham fechados.

## Limite semântico

Claims são linguagem natural estruturada com modalidade `must`, `must-not` ou `may`. O
linter prova estrutura, proveniência e propagação de mudança; não prova a verdade da
claim. `.tekt` v1 não possui expressões, macros, condicionais ou execução.

## Consequências

Prompts proprietários ficam menores sem perder contratos comuns. Mudança compartilhada
invalida todos os consumers de forma determinística. Surge um grafo L0 próprio, com custo
de parser, walker, hashing transitivo, rollback e diagnóstico.

## Alternativas rejeitadas

- prompt compartilhado: viola P0104;
- copiar Markdown: cria divergência semântica;
- usar ADR como dependência executável: mistura decisão histórica e contrato consumível;
- DSL executável: complexidade e superfície de segurança sem necessidade atual;
- colocar núcleos fora de `prompts/`: separação taxonômica seria maior, mas reduziria a
  descoberta junto aos L0 proprietários; `_nuclei` mantém namespace inequívoco.
