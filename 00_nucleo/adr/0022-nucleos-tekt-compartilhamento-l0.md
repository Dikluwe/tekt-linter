# ADR-0022 — Núcleos Tekt para contratos L0 compartilháveis

**Estado:** aceito — Rev. 1
**Data:** 2026-08-25
**Decisor:** mantenedor humano
**Contexto:** P0104–P0105; representação física revisada por P0107

## Contexto

P0104 tornou `@prompt` biunívoco: cada código produtivo possui um prompt proprietário e
cada prompt proprietário possui um código. O inventário encontrou invariantes legítimos
compartilhados entre vários códigos. Duplicar esses invariantes fabricaria ownership;
permitir prompt plural destruiria a bijeção.

## Decisão

Criar o **Núcleo Tekt**, artefato L0 declarativo TOML:

- localização canônica `00_nucleo/prompts/_nuclei/**/*.toml`;
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
claim. Núcleo TOML v1 não possui expressões, macros, condicionais ou execução.

## Revisão 1 — representação TOML

A primeira implementação usava sintaxe TOML 1.0 com extensão proprietária `.tekt`. Como
não existe linguagem própria, P0107 torna a extensão física coerente com a serialização:
`.toml`. A identidade do artefato é dada conjuntamente pelo namespace `_nuclei`, schema
fechado, `tekt = 1` e `kind = "nucleus"`; nenhum TOML fora desse namespace é Núcleo Tekt.

`.tekt` não é alias legado: sua presença é erro V26 explícito. `.tekt.toml` também é
inválido. A revisão não altera claims, modalidades, DAG, limites, hashing transitivo ou a
proibição de código referenciar núcleo diretamente.

## Consequências

Prompts proprietários ficam menores sem perder contratos comuns. Mudança compartilhada
invalida todos os consumers de forma determinística. Surge um grafo L0 próprio, com custo
de parser, walker, hashing transitivo, rollback e diagnóstico. A extensão conhecida permite
tooling TOML comum sem sugerir uma DSL nova.

## Alternativas rejeitadas

- prompt compartilhado: viola P0104;
- copiar Markdown: cria divergência semântica;
- usar ADR como dependência executável: mistura decisão histórica e contrato consumível;
- DSL executável: complexidade e superfície de segurança sem necessidade atual;
- colocar núcleos fora de `prompts/`: separação taxonômica seria maior, mas reduziria a
  descoberta junto aos L0 proprietários; `_nuclei` mantém namespace inequívoco.
