# Passo operacional 0098 — reconciliação do horizonte da auditoria

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/reconcile-audit-exit-criteria`
> **Pré-condição:** P0097 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0097

## Objetivo

Atualizar o inventário residual após P0097 e transformar a campanha de auditoria em um
backlog finito, ordenado e verificável. O passo deve distinguir:

1. trabalho obrigatório para considerar o linter suficientemente auditado;
2. saneamento L0 necessário antes de qualquer gate funcional;
3. auditoria residual aceita, acionada apenas por mudança, requisito ou nova evidência;
4. itens já fechados que não podem ser reabertos sem causa concreta.

P0098 é exclusivamente documental e de leitura. Não altera produção, testes executáveis,
prompts normativos, ADRs, configuração, fixtures ou comportamento. Seu resultado escolhe
no máximo um próximo lote, mas não autoriza executá-lo.

## Pergunta de auditoria

Depois do fechamento P0097, quais seams ainda bloqueiam a condição de saída da campanha,
quais são apenas resíduos aceitos e qual sequência mínima de lotes permite encerrar a
auditoria sem alegar cobertura inexistente nem criar horizonte infinito?

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| inventário P0096 | `00_nucleo/assessments/0025-inventario-risco-residual.md` | `4d9a7fa75def17dfcd5f5e552210b825d8b64ea98e64f8e9fdd430eb0fc74e2a` |
| reconciliação P0096 | `00_nucleo/assessments/0025-c-reconciliacao-risco.md` | `f713ec185c8c4e878da8c5cc609846271a6a1af3bd3a58e69b45e6667b1c7ede` |
| fechamento P0096 | `00_nucleo/relatorio-p0096-inventario-risco-residual.md` | `b653185723e46790ac32098cc8781787c8220247114d39301453be9c42750037` |
| Assessment P0097 | `00_nucleo/assessments/0026-extracao-source-constant-rust.md` | `26d94721dc5a0e6787f407859c27bf15e26e35b47b34611dd788e0cb3d4f30da` |
| fechamento P0097 | `00_nucleo/relatorio-p0097-auditoria-extracao-source-constant-rust.md` | `fc3d130fb3794e47fbe7ed387c7d0160305faf098952fe0366c033d90b181057` |

Na execução, todos os hashes devem ser recalculados depois da integração P0097. Uma
divergência é `RED` documental e exige resselamento; ela não autoriza copiar o conteúdo
vigente como nova verdade sem reconciliar a mudança.

## Universo inicial

P0098 deve começar pelas seams congeladas em P0096 e aplicar somente deltas sustentados
por P0097 ou por evidência posterior explicitamente identificada:

- S1 — extractor/escritor de snapshot;
- S2 — refinamento Git/subprocesso;
- S3 — manifesto, recibo e selo;
- S4 — pipeline principal;
- S5 — parsers concretos, desdobrado por linguagem e característica;
- S6 — preflight/precedência CLI ampliada.

P0097 pode fechar apenas a projeção numérica Rust de S5. Ele não fecha automaticamente
todos os campos de `SourceConstant`, citações, V22, V16 nem outros parsers.

## Arquitetura Tekt

Cada item residual deve declarar proprietário e travessias:

- L1: entidades, contratos e decisões puras de regra;
- L2: apresentação e serialização de diagnósticos;
- L3: filesystem, Git, subprocessos, parsers e persistência;
- L4: composição, precedência, comandos e política de execução.

O backlog não pode usar nomes de arquivo como unidade principal nem propor um lote que
misture decisão L1, efeito L3 e coordenação L4 sem pontos de parada. Se uma seam atravessa
camadas, ela deve ser decomposta por contrato observável antes de ser selecionada.

## Taxonomia de destino

Cada seam ou sub-seam deve receber exatamente um destino:

- `MANDATORY`: comportamento prometido, efeito externo ou decisão global ainda sem
  contrato/gate/consumidor suficientes; bloqueia a condição de saída;
- `L0-BLOCKED`: provavelmente obrigatório, mas autoridade ausente ou contraditória;
  requer saneamento normativo antes de gate ou produção;
- `ACCEPTED-RESIDUAL`: risco conhecido e nomeado, fora das promessas atuais; não bloqueia
  encerramento e só reabre por gatilho explícito;
- `CLOSED`: possui autoridade, gate independente, consumidor confrontado e fechamento;
- `REOPENED`: era fechado, mas existe causa concreta e identificada de reabertura.

Incerteza não pode virar `ACCEPTED-RESIDUAL`. Ausência de requisito demonstrável também
não pode virar trabalho obrigatório por precaução genérica.

## Condição de saída da campanha

A campanha pode ser declarada encerrada quando, simultaneamente:

1. não houver seam `MANDATORY` sem lote delimitado e ordenado;
2. não houver `L0-BLOCKED` afetando comportamento publicamente prometido;
3. toda entrada hostil ou efeito externo obrigatório tiver falha definida e gate
   independente proporcional ao risco;
4. regras suportadas tiverem ao menos uma cadeia confrontada fonte/entrada → IR → decisão
   → diagnóstico ou saída observável;
5. travessias L1/L2/L3/L4 estiverem explícitas e nenhuma camada usar outra como oráculo;
6. todos os resíduos restantes estiverem nomeados, justificados e associados a um gatilho
   objetivo de reabertura;
7. suíte, consumidores, auto-lint, hashes e fechamento adversarial estiverem verdes.

Encerrar a campanha não significa provar todos os programas, gramáticas ou ambientes
possíveis. Significa atingir a fronteira publicada e passar a auditar incrementalmente
mudanças futuras.

## Gatilhos de reabertura

Um item `CLOSED` ou `ACCEPTED-RESIDUAL` só pode voltar ao backlog por pelo menos um destes
eventos, com evidência:

- mudança posterior no produtor, contrato, consumidor ou dependência relevante;
- novo consumidor direto ou nova linguagem oficialmente suportada;
- bug reproduzível, incidente, vulnerabilidade ou comportamento incompatível;
- requisito de produto que transforme o residual em promessa;
- hash L0 divergente ou contradição normativa descoberta;
- gate demonstravelmente circular, contaminado ou insuficiente.

“Ainda não lemos tudo” e “pode existir algum caso” não são gatilhos válidos.

## Protocolo segregado

### A — cobertura histórica pós-P0097

A lê apenas o Assessment 0027 e os insumos hash-pinned. Reproduz o estado final de cada
seam e registra exatamente qual parte de S5 foi fechada por P0097. Não lê produção, não
pontua risco e não escolhe P0099.

### B1 — delta estrutural somente leitura

B1 lê produção, testes e histórico desde o baseline P0096, mas não lê o parecer A nem o
inventário normativo B2. Verifica produtores, consumidores, efeitos e travessias atuais,
procurando mudanças ou seams omitidas. Teste existente não vira gate independente por
estar verde.

### B2 — autoridade e promessas

B2 lê prompts, ADRs, CLI documentada e fechamentos, sem ler produção nem B1. Separa
comportamentos prometidos de explorações opcionais, identifica contradições e classifica
pré-condições normativas como suficientes ou `SPEC-GAP`.

### C — backlog finito e ranking

C recebe apenas A/B1/B2 congelados. Reconcilia discordâncias sem média silenciosa, aplica
a taxonomia de destino, decompõe cada `MANDATORY`/`L0-BLOCKED` em lotes delimitados e
produz:

- número finito de lotes restantes;
- dependências e ordem parcial entre eles;
- critério de aceite de cada lote;
- risco e confiança;
- no máximo um candidato para P0099.

Se o número não puder ser determinado, C deve nomear a informação ausente e retornar
`BLOCKED`; não pode usar “auditoria contínua” como resultado.

### D — adversário final

D confronta hashes, cobertura, produção, consumidores, arquitetura e backlog. Busca
especialmente: promessa obrigatória escondida como residual, residual opcional promovido
sem causa, seam crítica omitida, item fechado reaberto genericamente, dupla contagem e
lote que atravesse camadas sem seam observável.

## Classificações

- `RED`: hash divergente, cobertura falsa, omissão comprovada ou destino incompatível com
  a evidência;
- `SPEC-GAP`: autoridade não decide se a seam é promessa ou qual comportamento exigir;
- `GATE-DEFECT`: teste histórico/acoplado tratado como prova independente;
- `PASS`: classificação e condição de saída reproduzíveis;
- `ACCEPTED-RESIDUAL`: exclusão deliberada com risco e gatilho de reabertura registrados.

Fechamento somente `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. baseline pós-merge P0097 e todos os insumos usados hash-pinned;
2. reconciliação explícita das seis seams S1–S6;
3. desdobramento de S5 sem promover o P0097 a fechamento do parser Rust inteiro;
4. busca reversa produtor→consumidor e consumidor→produtor;
5. mapa seam→L0→camada→efeito→consumidor→gate→destino;
6. contagem finita de lotes `MANDATORY` e `L0-BLOCKED`;
7. critério de aceite e dependências de cada lote;
8. nenhum arquivo fora de `00_nucleo` alterado;
9. `cargo test --workspace --quiet` como regressão, não como prova de cobertura;
10. auto-lint V5/V6/V7/V12, reparador V5 dry-run e `git diff --check`;
11. adversário D e worktree limpo no fechamento.

## Saídas esperadas

- `00_nucleo/assessments/0027-horizonte-finito-auditoria.md`;
- artefatos segregados A/B1/B2, incorporados ou anexados com SHA-256;
- matriz reconciliada das seams S1–S6 e seus desdobramentos;
- backlog finito, dependências, critérios de aceite e gatilhos de reabertura;
- recomendação única para P0099 ou declaração de que a condição de saída foi atingida;
- `00_nucleo/relatorio-p0098-reconciliacao-horizonte-auditoria.md`;
- veredito final.

P0098 não autoriza alteração funcional, criação de gates executáveis, saneamento de L0,
merge, push, instalação ou release. Sem integração prévia de P0097 em `master`, sua
execução deve parar antes de criar branch concorrente.
