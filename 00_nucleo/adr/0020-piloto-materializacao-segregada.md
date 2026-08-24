# ADR-0020 — Piloto de materialização segregada

**Status:** ACEITO — autorizado pelo humano em 2026-08-24
**Data:** 2026-08-24
**Origem:** Tekt ADR-0003 (proposta)

## Contexto

O refinamento entre revisões valida uma transformação concreta, mas quem escreve o
contrato ainda pode adaptá-lo à solução. Tekt propõe estender o Protocolo A/B: contrato
e oráculos surgem antes do implementador, o contrato demonstra poder discriminatório e
um verificador mecânico emite um selo.

## Decisão

Executar no próprio linter um piloto mínimo, independente de fornecedor de agentes.
Adicionar `seal-refinement`, que lê um manifesto TOML, valida hashes congelados e roda
oráculos Git imutáveis pelo extrator/comparador existentes.

O manifesto v1 contém identidade do prompt, baseline, contrato, produtores declarados
e oráculos `positive`, `negative` e `unknown`. O selo só é publicado atomicamente se:

- hashes de prompt e contrato conferem;
- baseline resolve ao OID congelado;
- produtores de contrato, implementação e verificação são distintos;
- todo positivo retorna `PRESERVED`;
- todo negativo retorna `VIOLATED` — `UNKNOWN` não conta como rejeição;
- todo oráculo de opacidade retorna `UNKNOWN`;
- `mutation_score = 1.0`;
- há pelo menos um oráculo de cada categoria;
- negativos aceitos não contêm resultados inconclusivos além da violação;
- nenhuma entrada ou repositório analisado é alterado.

O selo registra hash semântico canônico do manifesto, hashes exatos de prompt/contrato,
OIDs resolvidos, recibos dos oráculos, contagens, score e versão do protocolo. Strings
de produtor são recibos nominais, não prova de isolamento. O piloto
registra essa limitação e depende também do isolamento real do executor.

## Camadas

- L1: entidades puras de manifesto compilado, expectativa, recibo e decisão de selo.
- L2: argumentos e apresentação.
- L3: TOML/JSON, SHA-256, Git já aprovado e escrita atômica.
- L4: composição e execução ordenada.

## Gate

Prompt causal e fixtures RED precedem implementação. Agente de contrato/testes não lê
o patch implementador; implementador recebe prompt e contrato já congelados. O piloto
fica em branch dedicado e não altera o binário instalado.

### Emenda — contrato executável do verificador

O papel verificador recebe valores normativos completos ou uma lista explícita de
artefatos L0 autorizados, cada um fixado por caminho e SHA-256 exato. Proibir produção
não implica proibir sua especificação causal. Cardinalidade sem enumeração e referência
que o papel não pode ler são `SPEC-GAP` bloqueante. Testes não usam constantes exportadas
pelo alvo como expectativa, para não compartilhar o mesmo oráculo com a implementação.

## Adiado

Sandbox atestável, assinatura criptográfica de agentes, serviço remoto de identidade,
orquestração automática, política de conflito e certificado pós-implementação.
