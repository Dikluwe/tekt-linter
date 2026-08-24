# Passo operacional 0077 — saneamento de escopo e evidência de V1

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** escrito, não executado
> **Branch:** `codex/segregated-materialization`
> **Base:** assessment 0009, commits `ce4a829` e `8e5b54a`

## Objetivo

Fechar os dois REDs de V1: representar seu escopo L1–L4 no contrato puro e preservar a
identidade do prompt quando o header existe, mas a referência confinada não existe.

## Decisões congeladas

1. `HasPromptFilesystem` passa a expor `fn layer(&self) -> &Layer`. V1 não infere camada
   pelo path e não recebe um segundo oráculo divergente.
2. V1 aplica-se somente a L1, L2, L3 e L4. L0, Lab e Unknown retornam vazio antes de
   avaliar header, existência ou diretório estrito.
3. Em camada aplicável, os estados são disjuntos:
   - `header == None`: uma V1 com a mensagem histórica de linhagem ausente;
   - `header == Some && !prompt_file_exists`: uma V1 cuja mensagem distingue referência
     inexistente e inclui literalmente `header.prompt_path`;
   - `header == Some && prompt_file_exists`: vazio.
4. A severidade continua `Fatal` quando o path pertence por componentes a um diretório
   estrito e `Error` nos demais casos aplicáveis. A causa não altera essa política.
5. Cardinalidade máxima, rule id `V1`, path, linha 1 e coluna 0 permanecem inalterados.
   Não normalizar o `prompt_path`; Unicode, caixa e representação são evidência.
6. `ParsedFile` e todos os dublês legítimos implementam o novo método. V15 e sua trait
   não serão alteradas.
7. Os prompts causais de V1 e da trait devem absorver o contrato final. Todos os hashes
   serão atualizados pelo fluxo oficial `--fix-hashes`.

## Segregação e execução

- A implementa este passo e os prompts causais sem ler assessment 0009, gate ou lab.
- B endurece e ativa o gate sem ler produção modificada.
- C revisa após o primeiro gate verde sem ler testes de B.
- O orquestrador executa suíte completa, assessments, auto-lint e `git diff --check`.

## Critérios de fechamento

- assessment 0009: 6/6, zero ignorados;
- tabela 7 camadas × 3 estados V1 verde;
- mensagens de header ausente e prompt inexistente distintas;
- path causal ausente preservado literalmente;
- V15 e demais regras sem regressão;
- adversário declara **NÃO REABRIR** ou apresenta RED reproduzível.

## Parada

Registrar relatório final. Não fazer merge, instalação ou release. Qualquer mudança na
política de diretórios estritos ou na extração L3 exige contrato separado.
