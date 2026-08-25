# Assessment 0028/C — decisão e saneamento nominal de `refine-revisions`

**Papel:** C — decisão e saneamento L0 segregados  
**Data:** 2026-08-25  
**Decisão:** `CONFIRMED`  
**Classificação desta etapa:** `PASS-CONFIRMED` — patch nominal pronto para executor separado  
**Produção e testes lidos:** não  
**Arquivos-alvo alterados por C:** nenhum

## Identidade dos pareceres congelados

| Parecer | SHA-256 exigido | SHA-256 recalculado | Resultado |
|---|---|---|---|
| A — cronologia e autoridade | `ff575cffba2dd8885ef0db60c467d9927d79b8ead0384b0dffc30f3c4bbb41b4` | `ff575cffba2dd8885ef0db60c467d9927d79b8ead0384b0dffc30f3c4bbb41b4` | `PASS` |
| B — superfície pública | `83f86a97ad856d686eb715f8eafa67b9c6db59848ea7a4d21c5f23b73c4f33b9` | `83f86a97ad856d686eb715f8eafa67b9c6db59848ea7a4d21c5f23b73c4f33b9` | `PASS` |

Também foram confrontados os pins do Assessment 0028 para o prompt de refinamento,
ADR-0019, README e USAGE; os quatro hashes recalculados coincidem com os valores
congelados. Nenhuma implementação, teste, fixture ou help gerado participou da decisão.

## Decisão binária

`refine-revisions` está **CONFIRMED** como capacidade normativa vigente, exclusivamente
dentro do envelope Git local, imutável e limitado da adenda B2 aceita em 2026-08-24.

Fundamento causal:

1. a adenda B2 registra aprovação humana explícita, datada e posterior ao Gate de
   2026-08-23 que limitava a materialização à Etapa A;
2. a adenda declara a materialização autorizada e define precisamente autoridade,
   ameaças, efeitos, budgets, camadas e portabilidade;
3. o prompt vigente registra Etapa B2 e `refine-revisions` com a mesma aprovação;
4. não existe revogação no corpus autorizado;
5. o parecer A confirma essa cronologia sem recorrer à superfície ou à produção;
6. o parecer B identifica silêncio e ambiguidade públicos, corrigíveis sem ampliar B2.

O cabeçalho e o Gate antigos do ADR não revogam B2: registram o escopo anterior que a
adenda aceita ampliou. A redação final deve preservar esse histórico e separar
inequivocamente o Gate histórico do contrato vigente.

## Envelope confirmado — limite de autoridade

A confirmação autoriza documentar somente o seguinte:

- sintaxe nominal:
  `crystalline-lint refine-revisions <repository-root> --before-ref <sha-ou-ref>
  --after-ref <sha-ou-ref> --contract refinement.toml`;
- requisito externo: Git local 2.43 **ou compatibilidade demonstrada** com
  `--batch-command` e `--end-of-options`; nenhuma nova dependência de biblioteca Git;
- cada ref é resolvida uma única vez para commit OID; somente os OIDs imutáveis são
  usados depois na enumeração, extração, identidade e testemunhas;
- execução local de Git com argumentos separados por `Command`, nunca shell;
- objetos locais em somente leitura, sem rede ou fetch, checkout, worktree, stash,
  build, hooks, filtros, textconv, LFS, protocolos externos ou travessia de symlink e
  submódulo;
- working tree, índice, HEAD, branch, refs e stash não participam da comparação e não
  podem ser alterados;
- somente blobs regulares são aceitos;
- arquivo realmente ausente segue `on_missing`; objeto esperado ausente, symlink,
  submódulo, framing inválido e erro de leitura são erro de entrada ou evidência
  inconclusiva, nunca ausência conhecida;
- budgets iniciais: 512 paths observáveis, 4 MiB por blob, 32 MiB por revisão e 10
  segundos por operação Git; excesso produz `Unknown(BudgetExhausted)` ou erro antes
  de publicar resultado, sem truncamento silencioso;
- bytes por path lógico alimentam o mesmo extrator B1 e o mesmo comparador de `refine`;
  a equivalência com `snapshot + refine` para o mesmo conteúdo permanece requisito;
- `refine` não executa Git; `refine-revisions` pode executar apenas Git local dentro
  deste envelope;
- L1 permanece pura e sem tipos Git; L2 contém argumentos/apresentação e política de
  exit; L3 contém filesystem, adapter Git e contenção do subprocesso; L4 resolve OIDs
  e compõe extração e comparação.

Não ficam autorizados temporários de diagnóstico, escrita, rede, fetch, outras formas
de revisão, outro backend, mudança de budgets, nova dependência, mutação do repositório
ou política global de exits. Qualquer desses itens exige nova decisão L0.

## Patch nominal aprovado para o executor

O executor deve aplicar somente as mudanças documentais abaixo. Pode ajustar quebras
de linha e referências internas, mas não pode acrescentar semântica além deste plano.

### 1. `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md`

1. No cabeçalho, substituir o escopo vigente “Etapas A e B1 autorizadas; Git ... não
   autorizado” por “Etapas A, B1 e B2 autorizadas; wrapper e SMT não autorizados”.
2. Renomear `## Gate` para `## Gate histórico — Etapas A e B1` e introduzir uma nota
   imediatamente abaixo esclarecendo que o texto preserva o limite aprovado em
   2026-08-23/Etapa B1, posteriormente ampliado **somente** pela adenda B2 aceita em
   2026-08-24.
3. Preservar integralmente as proibições históricas do Gate; não apagá-las nem
   apresentá-las como limite vigente de B2.
4. Renomear `## Adenda proposta B2 — fonte imutável de revisões Git` para
   `## Adenda B2 aceita — fonte imutável de revisões Git`.
5. Na subseção “Condição para aprovação”, substituir a condição futura por registro de
   fechamento: B2 foi aprovada em 2026-08-24 e está vigente apenas nos limites da
   adenda; fixtures e confronto funcional pertencem a F05, sem converter sua ausência
   em revogação normativa.
6. Manter backend, ameaça, efeitos, budgets, camadas e portabilidade da adenda sem
   expansão. Onde “proposto” descreve valor ainda não comprovado funcionalmente, não o
   promover a garantia de implementação; publicar o requisito normativo como “Git 2.43
   ou compatibilidade demonstrada”.

### 2. `00_nucleo/prompts/refinement-validator.md`

Preservar sem alteração. O prompt já registra B2 como vigente, sua sintaxe, envelope e
histórico de aprovação. O executor confirmou que qualquer alteração, ainda que apenas
explicativa, mudaria o hash L0 e exigiria atualizar cinco headers de produção, operação
proibida por P0099. As clarificações de superfície pertencem ao ADR, README e USAGE.

### 3. `README.md`

1. No bloco de uso rápido, após o exemplo de `refine`, adicionar a sintaxe nominal de
   `refine-revisions` com `<repository-root>`, `--before-ref`, `--after-ref` e
   `--contract`.
2. Na seção “Validação de refinamento”, substituir “O modo não lê Git...” por uma
   delimitação do subcomando: `refine` compara snapshots fornecidos, não lê Git, não
   executa comandos e não usa SMT.
3. Após o exemplo de snapshot, adicionar uma subseção curta “Comparação de revisões Git
   locais” contendo:
   - a sintaxe nominal;
   - Git local 2.43 ou compatibilidade demonstrada;
   - resolução única de refs para OIDs;
   - leitura somente de blobs regulares e objetos locais;
   - ausência de shell, rede/fetch e mutação de checkout, working tree, índice, HEAD,
     branch, refs ou stash;
   - não execução/travessia de hooks, filtros, textconv, LFS, symlinks e submódulos;
   - budgets de 512 paths, 4 MiB/blob, 32 MiB/revisão e 10 s/operação;
   - distinção entre `on_missing` para arquivo ausente e erro/`Unknown` para objeto
     ilegível, tipo proibido, framing inválido ou budget excedido;
   - equivalência normativa com `snapshot + refine` para o mesmo conteúdo.
4. Informar que a matriz completa de exits de `refine-revisions` será reconciliada em
   F09; não atribuir números ou precedência neste passo.

### 4. `USAGE.md`

1. Na seção “Comandos CLI”, adicionar uma nota de que a sinopse raiz cobre o lint e
   apontar nominalmente os subcomandos `snapshot`, `refine` e `refine-revisions`, sem
   tentar redesenhar a gramática CLI global reservada a F09.
2. Em “Comparar snapshots por refinamento”, declarar explicitamente que `refine` opera
   sobre arquivos de snapshot fornecidos e não executa Git.
3. Adicionar, imediatamente depois do fluxo `snapshot`/`refine`, uma subseção
   “Comparar revisões Git locais” com a mesma sintaxe e a mesma matriz de requisitos,
   efeitos, tipos admitidos, budgets, falhas e equivalência definida para o README.
4. Distinguir operacionalmente:
   - arquivo ausente no tree: segue `on_missing`;
   - ref inexistente: erro de entrada;
   - objeto esperado ausente/ilegível, symlink, submódulo, framing inválido ou budget
     excedido: erro de entrada ou `Unknown`, nunca ausência conhecida;
   - nenhum resultado pode ser publicado a partir de conteúdo truncado.
5. Registrar os resultados semânticos possíveis sem fixar código numérico ou
   precedência para `refine-revisions`; referenciar F09 como dono dessa reconciliação.
6. Não alterar a tabela geral de exits do lint nem os exits já publicados de `refine`.

## Efeito sobre o backlog finito

- **F04/P0099:** a autoridade está decidida como `CONFIRMED`; só pode fechar após o
  executor aplicar o patch, D confrontar a redação e as validações finais passarem.
- **F05:** desbloqueado normativamente para confrontar a implementação de
  `refine-revisions` com o envelope B2; P0099 não antecipa esse confronto.
- **F09:** recebe `refine-revisions` como comando vigente e continua dono da matriz
  global de precedência e exits.
- **F08:** recebe a superfície vigente reconciliada, mas continua bloqueado pelas suas
  próprias pré-condições; P0099 não audita o pipeline.

## Resíduos e condição de interrupção

Até a aplicação e o parecer D, a documentação pública continua em `RED` por omissão e
o ADR continua em `RED` por coexistência não qualificada do Gate antigo com a adenda
aceita. Se o executor precisar decidir código de exit, aceitar efeito não enumerado,
alterar backend/budget ou prometer conformidade da implementação, deve classificar
`SPEC-GAP` e interromper em vez de improvisar.

Este parecer não autoriza alteração funcional, teste executável, fixture, configuração,
dependência, merge, push ou release.

## RED do executor e resselamento

O patch nominal inicial previa três clarificações no prompt. O executor as aplicou
provisoriamente e o gate V5 acusou drift causal em cinco arquivos de produção. Atualizar
esses headers violaria a fronteira documental de P0099; deixar o drift violaria a
validação mínima. Como A já demonstrou que o prompt vigente confirma B2 sem lacuna de
autoridade, o delta não era necessário à decisão.

As mudanças provisórias no prompt foram integralmente descartadas. Esta versão
resselada autoriza alterações somente no ADR-0019, README e USAGE. A decisão continua
`CONFIRMED`, sem expansão semântica e sem migração de hashes de produção.

## RED D1

D1 encontrou duas contradições fora do patch nominal: `linter-core.md` ainda apresenta
Git como fora de escopo após descrever somente B1, e `00_nucleo/README.md` repetia a
proibição imediatamente antes de registrar B2 como aprovada. Também faltavam nas duas
superfícies públicas as proibições de build, protocolos externos e temporários de
diagnóstico.

O índice `00_nucleo/README.md`, README e USAGE foram saneados sem ampliar B2. A mudança
necessária em `linter-core.md`, porém, altera um prompt causal compartilhado e exige
migração mecânica de headers em produção para não deixar V5 em drift. P0099 proíbe essa
migração. O passo permanece bloqueado até autorização explícita para ampliar o escopo
somente aos headers/hash derivados, sem mudança comportamental.
