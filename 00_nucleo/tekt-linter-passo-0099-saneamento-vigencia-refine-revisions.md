# Passo operacional 0099 — saneamento da vigência Git de `refine-revisions`

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; `READY WITH RESIDUAL AUDIT` / `PASS-CONFIRMED`
> **Branch prevista:** `codex/reconcile-refine-revisions-authority`
> **Pré-condição:** P0098 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0098, lote F04

## Objetivo

Resolver de forma binária, coerente e rastreável a vigência de Git/`refine-revisions`.
Hoje o prompt registra B2 como vigente, a adenda do ADR-0019 declara aprovação humana,
mas o cabeçalho, escopo e gate anteriores do mesmo ADR ainda proíbem Git; a documentação
pública descreve `refine` como sem Git e não distingue claramente `refine-revisions`.

P0099 deve escolher exatamente um resultado:

- `CONFIRMED`: `refine-revisions` é capacidade vigente e publicamente documentada dentro
  do envelope Git local, imutável, sem shell, rede ou mutação; ou
- `REVOKED`: a capacidade não é vigente; prompt, ADR e documentação deixam isso explícito
  e F05 passa a confrontar a remoção da promessa/rota pública.

Silêncio, “experimental”, coexistência das duas leituras ou decisão baseada na produção
resultam em `BLOCKED`.

## Natureza e fronteira

Este é um saneamento L0/documental. Pode alterar exclusivamente arquivos de
`00_nucleo` e documentação pública necessária para remover a contradição. Não pode
alterar Rust, testes executáveis, fixtures, configuração, dependências ou comportamento.

A existência atual de comando, adapter ou teste é evidência de implementação, nunca
autoridade para confirmar vigência. Da mesma forma, remover texto normativo não autoriza
remover produção neste passo.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| contrato de refinamento | `00_nucleo/prompts/refinement-validator.md` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` |
| ADR de refinamento | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` |
| README público | `README.md` | `3ff67521214cff672b54941e1d4392b2ab933c51ed69ecc9cf5e55e8989716d6` |
| guia público | `USAGE.md` | `245bc38db11a29467e7e72514f488fcb69fd471d401fa7eb1b6823355fa8d4f1` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment P0098 | `00_nucleo/assessments/0027-horizonte-finito-auditoria.md` | `dbfd5755641962132cafc967951ba1ae8bc8197370152af426a01eab7e1389f7` |
| backlog F01–F13 | `00_nucleo/assessments/0027-c-backlog-finito.md` | `c829befc0df2addb431406d3592c88499a2c47d70d0178ed3d25bef7369b1314` |
| fechamento P0098 | `00_nucleo/relatorio-p0098-reconciliacao-horizonte-auditoria.md` | `9e8d3acd399088c18bbfabf232cb307863fe28f03a3472e5f8f59dfc05c76a6d` |

Na execução, os hashes devem ser recalculados após o merge P0098. Divergência exige
classificação `RED`, explicação do delta e resselamento antes de qualquer alteração.

## Contradição congelada

O Assessment 0028 deve confrontar nominalmente:

1. `refinement-validator.md`: cabeçalho vigente, Etapa B2 aprovada e histórico de
   aprovação em 2026-08-24;
2. ADR-0019: cabeçalho limita o escopo a A/B1 e proíbe Git;
3. ADR-0019/Gate: afirma que leitura Git continua não autorizada;
4. ADR-0019/Adenda B2: declara-se aceita e materializável;
5. README: `refine` não lê Git nem executa comandos, afirmação correta para esse comando
   mas ambígua como descrição da capacidade de refinamento completa;
6. README/USAGE: ausência ou insuficiência da interface pública de `refine-revisions`;
7. F04: exige decisão coerente para desbloquear F05, F09 e F08.

Não se pode resolver a contradição apagando o histórico de decisões. Texto substituído
deve permanecer identificável como escopo anterior ou histórico superado.

## Critérios para a decisão

O parecer deve usar, em ordem:

1. aprovação humana explícita e datada;
2. seção normativa mais recente que declare substituir ou ampliar escopo anterior;
3. coerência com os limites arquiteturais e de autoridade aprovados;
4. documentação pública necessária para que o usuário conheça efeitos e requisitos;
5. ausência de evidência suficiente resulta em `SPEC-GAP`, nunca em escolha por
   conveniência.

A cronologia sozinha não basta se os textos não demonstram aprovação inequívoca. A
produção existente não participa desta ordem.

## Matriz mínima se `CONFIRMED`

Prompt, ADR, README e USAGE devem convergir sobre:

- nome e sintaxe do subcomando;
- requisito de Git local e versão/compatibilidade publicada;
- resolução única de refs para OIDs e uso posterior somente desses OIDs;
- repositório e objetos locais em somente leitura;
- proibição de shell, rede/fetch, checkout, worktree, stash, build, hooks, filtros, LFS e
  travessia de symlink/submódulo;
- budgets de paths, blob, revisão e tempo;
- objetos ausentes/framing inválido como erro ou `Unknown`, nunca ausência conhecida;
- working tree, índice, HEAD, branch e stash imutáveis;
- equivalência normativa com `snapshot + refine` no mesmo conteúdo;
- distinção explícita: `refine` não executa Git; `refine-revisions` executa apenas Git
  local no envelope aprovado;
- exits e requisitos que ainda dependem do saneamento F09 claramente marcados, sem
  inventar a matriz global neste lote.

O ADR deve atualizar cabeçalho, escopo e Gate sem apagar a decisão histórica A/B1.

## Matriz mínima se `REVOKED`

Prompt, ADR, README e USAGE devem convergir sobre:

- B2 não vigente e motivo verificável da revogação;
- `refine` e `snapshot + refine` como únicas superfícies vigentes relacionadas;
- nenhuma autorização para executar Git, rede, shell ou mutar repositório;
- adenda B2 preservada como proposta/aprovação histórica revogada, não como contrato
  ativo;
- destino obrigatório de qualquer rota ou documentação conflitante em F05;
- F09/F08 recebem somente a lista de comandos que permanecer vigente.

P0099 não remove código ou teste existente; apenas torna a decisão normativa inequívoca.

## Protocolo segregado

### A — cronologia e autoridade

A lê somente o Assessment 0028, prompt, ADR e registros históricos hash-pinned. Não lê
produção, README/USAGE nem recomenda redação. Produz uma linha do tempo de decisões,
separa proposta/aprovação/revogação e classifica a evidência como suficiente para
`CONFIRMED`, suficiente para `REVOKED` ou `SPEC-GAP`.

### B — superfície pública independente

B lê somente o Assessment 0028, README, USAGE e contrato de CLI documental autorizado.
Não lê produção, parecer A nem usa help gerado pelo binário como autoridade. Enumera o
que um usuário pode inferir hoje, ambiguidades de efeitos/requisitos e delta mínimo para
cada ramo possível.

### C — decisão e saneamento L0

C começa somente após A/B congelados. Recebe seus hashes e os insumos autorizados,
escolhe `CONFIRMED` ou `REVOKED` apenas se a evidência permitir e propõe um patch nominal.
Um executor separado aplica somente o patch documental aprovado. Qualquer expansão do
envelope Git além da adenda B2 é `SPEC-GAP` e interrompe o passo.

### D — adversário final

D confronta hashes, cronologia, redação final, documentação pública e arquitetura. Busca
autorização implícita, promessa escondida, escopo anterior apresentado como vigente,
efeito Git omitido, dependência/versão inventada, política de exit pertencente a F09 e
uso da produção como autoridade.

## Arquitetura Tekt

Se confirmada, a decisão deve preservar:

- L1: fatos, contrato e comparação puros, sem tipos Git;
- L2: argumentos, apresentação e política de exit, cuja matriz global permanece F09;
- L3: adapter Git local, bytes por path e contenção de subprocesso;
- L4: resolução/composição dos dois lados por OID e coordenação do comando.

P0099 documenta essas responsabilidades, mas não materializa nem move código entre
camadas.

## Classificações e fechamento

- `RED`: hash divergente, textos ainda contraditórios ou afirmação pública falsa dentro
  do ramo escolhido;
- `SPEC-GAP`: evidência não autoriza confirmar/revogar ou exige ampliar o envelope;
- `GATE-DEFECT`: teste, help gerado ou produção usado como fonte normativa;
- `PASS-CONFIRMED`: vigência confirmada e todas as superfícies reconciliadas;
- `PASS-REVOKED`: vigência revogada e todas as superfícies reconciliadas.

Fechamento global somente `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. baseline pós-merge P0098 e todos os insumos usados hash-pinned;
2. pareceres A/B em arquivos separados e identidades congeladas;
3. decisão binária e justificativa causal;
4. busca por `refine-revisions`, B2, Git não autorizado e proibições antigas em todo
   `00_nucleo`, README e USAGE;
5. prompt, ADR, README e USAGE sem afirmações normativas contraditórias;
6. histórico preservado e escopo vigente inequívoco;
7. nenhum arquivo fora de `00_nucleo`, README e USAGE alterado;
8. nenhuma produção, teste, fixture, configuração ou dependência alterada;
9. `cargo test --workspace --quiet` como regressão, não como prova normativa;
10. auto-lint V5/V6/V7/V12, reparador V5 dry-run e `git diff --check`;
11. adversário D e worktree limpo no fechamento.

## Saídas esperadas

- `00_nucleo/assessments/0028-vigencia-refine-revisions.md`;
- pareceres segregados A/B hash-pinned;
- decisão `CONFIRMED` ou `REVOKED`;
- prompt, ADR, README e USAGE reconciliados conforme o ramo escolhido;
- `00_nucleo/relatorio-p0099-saneamento-vigencia-refine-revisions.md`;
- efeito explícito sobre F05, F09 e F08;
- veredito final.

P0099 não autoriza alteração funcional, gate executável, remoção de rota, dependência,
instalação, merge, push ou release. Sem integração prévia de P0098 em `master`, sua
execução deve parar antes de criar branch concorrente.
