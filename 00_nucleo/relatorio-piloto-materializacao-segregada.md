# Relatório do piloto — materialização segregada

Data: 2026-08-24
Branch: `codex/segregated-materialization`

## Resultado

O piloto implementou `seal-refinement`, um portão que lê um manifesto congelado,
executa oráculos de refinamento sobre objetos Git imutáveis e publica um selo JSON
determinístico somente quando todo o pacote passa.

O fluxo segregado detectou defeitos em três fontes diferentes:

1. **Implementação:** a primeira execução passou 8/12 testes; o caminho absoluto do
   manifesto havia sido interpretado como caminho interno do repositório.
2. **Verificação:** após a correção, um fixture que alegava preservar um enum tinha
   acrescentado uma vírgula e alterava os bytes semânticos sob a regra exata.
3. **Contrato:** em 10/12, ficou evidente que “hash do manifesto” e “ordem TOML não
   altera o selo” eram incompatíveis sem definir uma representação canônica.

Cada defeito voltou ao produtor responsável sem revelar o material privado dos outros
papéis. O contrato foi revisado e congelado antes de cada correção de produção.

## Gate final

- 14/14 testes black-box de materialização segregada;
- 585/585 testes unitários;
- 83/83 fixtures gerais;
- 10/10 testes de refinamento anteriores;
- linter sem erro, warning de proveniência ou prompt órfão introduzido pelo piloto;
- `git diff --check` limpo.

A revisão adversarial acrescentou dois casos que o gate inicial não enxergava:

- ausência de qualquer uma das categorias `positive`, `negative` ou `unknown`;
- lavagem de negativo por uma violação-isca coexistindo com inconclusivos.

Ambos foram inicialmente RED e terminaram verdes após atualização explícita do L0.

## O que torna o processo usual

O uso cotidiano pode ser reduzido a quatro artefatos e um comando:

1. prompt e contrato congelados;
2. manifesto com baseline, produtores nominais e pares de commits;
3. testes escritos por agente sem acesso à implementação;
4. revisão adversarial sem acesso à implementação nem aos testes;
5. `crystalline-lint seal-refinement <repo> --manifest <arquivo> --output <selo>`.

O selo é um recibo reproduzível da execução. Ele não prova, sozinho, que os agentes
foram realmente isolados; os nomes de produtores são declarações nominais. O executor
que orquestra os agentes ainda precisa impor a separação de contexto.

## Limites observados

A versão 1 não prova vínculo causal entre prompt, baseline e cada mutação. Ela também
não distingue uma razão de `UNKNOWN` intencional de um fixture defeituoso. Por isso o
selo afirma apenas que os pares declarados foram executados de forma reproduzível sob
o contrato, sem alegar independência certificada ou correção completa da mudança.

O próximo incremento recomendado é tornar a segregação uma receita executável do
Tekt: criar sessões isoladas, controlar quais artefatos cada papel pode ler, registrar
os recibos e chamar este portão automaticamente. Só depois disso convém promover o
ADR do estado `PROPOSTO` para uma regra estável da arquitetura.
