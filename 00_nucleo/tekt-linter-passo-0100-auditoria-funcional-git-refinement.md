# Passo operacional 0100 — auditoria funcional Git de `refine-revisions`

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; `BLOCKED` — F05 não fechado
> **Branch prevista:** `codex/audit-git-refinement-functional`
> **Pré-condição:** P0099 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0099 / F04
> **Lote do backlog:** F05

## Objetivo

Confrontar funcionalmente a fronteira Git de `refine-revisions` com o envelope B2 agora
confirmado. O lote cobre resolução imutável de refs, protocolo de processos Git, leitura
de blobs, contenção de efeitos, budgets, timeout/encerramento e projeção de falhas como
erro ou evidência inconclusiva — nunca como ausência conhecida ou sucesso.

O gate principal deve controlar um executável Git hostil e observar argumentos,
ambiente, stdin/stdout/stderr, framing, tempo e encerramento. Repositório Git real pode
servir como regressão e prova de integração, mas não como único oráculo de segurança.

## Fronteira e exclusões

Entrada autorizada: raiz explícita de repositório, duas refs não confiáveis, contrato de
extração já válido e respostas byte-level do processo Git. Saída observável: conteúdo
imutável por path/OID ou erro/inconclusão tipados que chegam ao extrator compartilhado.

Ficam fora:

- códigos numéricos, precedência, `quiet`, formato e política global de exit — F09/F10;
- composição completa dos comandos de refinamento — F08;
- schema e writer de snapshot — F01–F03;
- semântica do comparador L1 já fechada;
- mudança de backend para `gix`/`git2`;
- rede, fetch, checkout, build, LFS, submódulos, temporários diagnósticos e suporte a
  ambientes além do envelope publicado;
- prova universal de todas as versões/configurações/implementações Git.

O lote pode usar a CLI apenas como consumidor de regressão. Nenhuma expectativa do gate
hostil pode ser derivada de códigos de exit da CLI.

## Risco

Risco crítico, confiança alta: L3 executa processo externo sobre refs, paths, objetos,
configuração e framing hostis; timeout e kill possuem concorrência; o resultado alimenta
extração e comparação; falso sucesso pode certificar transformação não observada.

Uma falha de contenção, mutação, framing ou orçamento é `RED` mesmo quando testes com Git
real permanecem verdes.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| contrato de refinamento | `00_nucleo/prompts/refinement-validator.md` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` |
| ADR B2 confirmado | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `088e5806c948d60c2f5b1ea2c04c4b181672c037c31f53c0b125ddf594a497d6` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment F04 | `00_nucleo/assessments/0028-vigencia-refine-revisions.md` | `46b1bcec486c8e909fe1bc66a36e9e0a9b7c91d992ead6f8fd20f53fd73b4ba2` |
| decisão F04 | `00_nucleo/assessments/0028-c-decisao-saneamento.md` | `1fa048e01935717806ef48ae6ea74cda62cc2f26b6807ee2d806f8176adf9f06` |
| fechamento P0099 | `00_nucleo/relatorio-p0099-saneamento-vigencia-refine-revisions.md` | `15d0d757414358f94f073b7084c1b0d057c834986343148c0dad56fb8f854588` |
| Assessment Git histórico | `00_nucleo/assessments/0001-git-refinement.md` | `5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17` |
| fechamento histórico P0072 | `00_nucleo/relatorio-p0072-saneamento-deterministico-segregado.md` | `d43d1dd6e9d356b0f3dcd652a57c02cf7af0d24ecf24fb8c6d419d7b2a393fb7` |
| gate histórico Git real | `tests/git_refinement_assessment.rs` | `9609ebdb84d21fb79cddd744392d9fb8692513c809bf651c52eefa1c8b75c434` |
| inventário estrutural P0098 | `00_nucleo/assessments/0027-b1-delta-estrutural.md` | `fac7a67068e6f63a969f3725710026afed3f828275859e8f49cafb6a1ec914e2` |
| inventário normativo P0098 | `00_nucleo/assessments/0027-b2-autoridade-promessas.md` | `3f4fee9273c72ca0202e9f2ab95e551f7f53e55c7862493ba34660a148f94e3e` |

Todos os hashes devem ser recalculados após a integração P0099. Gate histórico é
evidência de regressão, não oráculo independente nem substituto do gate hostil.

## Preflight normativo obrigatório

O Assessment 0029 e o adversário A devem decidir, antes de qualquer gate:

1. comandos Git autorizados, ordem, argumentos e delimitadores;
2. ambiente que deve ser removido/forçado e tratamento de `PATH`/executável Git;
3. sintaxe aceita de ref e separação contra opção/pathspec;
4. resolução única para commit OID e suporte opaco a SHA-1/SHA-256;
5. modos aceitos: blob regular; arquivo ausente versus objeto ausente; symlink/gitlink;
6. framing de `ls-tree -z` e `cat-file --batch-command --buffer`, incluindo tamanho,
   tipo, NUL, newline, duplicata, resposta truncada e bytes não UTF-8;
7. budgets de 512 paths, 4 MiB/blob, 32 MiB/revisão e contabilidade antes da publicação;
8. timeout de 10 segundos por operação, kill, reap e tratamento de descendentes;
9. neutralização de prompts, lazy fetch, replace objects, locks, configs, hooks,
   protocolos, filtros, textconv, LFS, alternates e object stores externos;
10. API/porta mínima para um gate injetar processo hostil sem usar a produção como
    expectativa;
11. equivalência B2 com o mesmo extrator B1 e preservação de identidade/OID;
12. taxonomia exata de erro de entrada versus `Unknown`, sem decidir exit global.

Ausência de decisão é `SPEC-GAP`. Se a API atual não permitir gate determinístico, o
Assessment pode autorizar uma seam de injeção L1/L3 mínima, mas ela deve ser congelada
antes de B1/B2 e não pode conter semântica Git em L1.

## Alegações candidatas

### Identidade e imutabilidade

1. Cada ref é resolvida uma única vez para commit OID; chamadas posteriores usam somente
   o OID, nunca reavaliam a ref mutável.
2. OID é tratado como string opaca validada pelo Git, sem assumir comprimento SHA-1.
3. Working tree, índice, HEAD, branch, refs, stash e bytes preexistentes permanecem
   inalterados em sucesso, erro, timeout e processo hostil.
4. `.git` indireto, alternates, object store externo ou ambiente Git herdado não ampliam
   silenciosamente a origem autorizada.

### Processo e efeitos

5. Comandos usam argv separado, nunca shell; refs/paths hostis não viram opções,
   pathspec, configuração ou comando.
6. Ambiente desabilita prompt, lazy fetch, replace objects, locks opcionais,
   configuração global/sistema, hooks e protocolos externos.
7. Nenhum fluxo chama fetch, checkout, worktree, stash, build, filtro, textconv, LFS ou
   submódulo.
8. Timeout encerra e reap o processo; pipes e threads não ficam bloqueados e resultado
   parcial nunca é publicado.

### Framing, tipos e budgets

9. Somente blobs regulares entram no extrator; symlink, gitlink e tipo inesperado são
   inconclusivos/erro, nunca ausência conhecida.
10. Framing inválido, tamanho divergente, resposta truncada, objeto inesperado e bytes
    hostis falham fechados.
11. Os três budgets e a contagem de paths são aplicados sem truncamento ou publicação
    parcial; excesso vira `BudgetExhausted`/erro autorizado.
12. Arquivo realmente ausente no tree respeita `on_missing`; objeto esperado ausente ou
    ilegível não usa `on_missing`.
13. Mesmos OIDs, contrato e blobs produzem fatos equivalentes a B1/
    `snapshot + refine`, sem comparador ou normalizador paralelo.

## Protocolo segregado

### A — adversário L0 e de testabilidade

A lê somente Assessment 0029 e os insumos hash-pinned, sem produção. Classifica os doze
itens de preflight e as treze alegações como `PASS`, `SPEC-GAP` ou `RED` normativo.
Também decide se o L0 publica seam injetável suficiente. Saneamento só pode alterar
contratos/ADRs e exige resselamento antes dos gates.

### B1 — gate cego de protocolo hostil

B1 cria exclusivamente `tests/git_refinement_protocol_assessment.rs` e helpers/fixtures
sob `tests/fixtures/git_refinement_protocol/`. Sem ler produção, materializa executável
Git controlado que registra argv/ambiente e responde com framing válido ou adversarial.
Cobre resolução única, separação argv, ambiente, tipos, framing, budgets e distinção
ausente/inconclusivo. Não chama shell e não usa rede.

Se o gate precisar de seam injetável autorizada, deve consumir somente o contrato
publicado no preflight; não pode criar expectativa a partir do adapter concreto.

### B2 — gate cego de timeout e contenção

B2 cria exclusivamente `tests/git_refinement_timeout_assessment.rs` e fixtures próprias,
sem compartilhar helpers com B1. Controla processo que bloqueia, fecha pipes, produz
saída parcial, termina com status hostil ou tenta deixar descendente. Observa timeout,
kill/reap, ausência de deadlock e ausência de publicação parcial. Deve ter watchdog
externo de teste menor que o limite global da suíte e limpar somente seus próprios PIDs.

### C — confronto e correção

Somente após B1/B2 congelados e RED inicial registrado, C lê
`03_infra/git_refinement.rs` e contratos estritamente necessários. Corrige somente a
fronteira L3 e eventual port L1 de processo/conteúdo previamente autorizada.

São proibidos: mudar comparador, extractor/schema, CLI/exit, pipeline global, backend,
budgets normativos ou afrouxar gates para reproduzir implementação.

### D — adversário final

D confronta hashes, independência, ausência de oráculo circular, RED→GREEN, estado do
repositório antes/depois, argv/ambiente, framing, budgets, timeout, descendentes,
equivalência B1/B2, arquitetura Tekt e consumidores. Deve classificar qualquer teste que
dependa somente de Git real como `GATE-DEFECT` para a prova de contenção.

## Matriz mínima do gate hostil

| Classe | Casos mínimos |
|---|---|
| refs | nome hostil, opção aparente, mudança entre chamadas, ref inexistente |
| OIDs | SHA-1, SHA-256/opaco quando suportado, tipo não commit, resposta truncada |
| paths | NUL/framing, bytes não UTF-8, opção/pathspec mágico, duplicata, ausente |
| objetos | blob regular, symlink, gitlink, tipo inesperado, missing, tamanho divergente |
| ambiente | `GIT_DIR`, work tree/common dir, alternates, config global/system, prompt |
| efeitos | shell/rede/fetch/hook/filter/LFS/build/checkout nunca invocados |
| budgets | 512/513 paths, 4 MiB ±1, 32 MiB ±1, sem truncamento |
| processo | sucesso, status não zero, stderr hostil, timeout, pipe fechado, descendente |
| determinismo | mesma entrada/ordem diferente, mesmos fatos e diagnóstico estável |

Casos impossíveis de produzir com a seam autorizada devem ser `SPEC-GAP`, não simulados
por assert sobre helper do próprio gate.

## Consumidores e regressões

Executar separadamente, sem promovê-los a oráculo:

- gate histórico `git_refinement_assessment`;
- CLI `refinement_cli` apenas como regressão;
- loader/extractor e comparador de refinamento já fechados;
- suíte completa do workspace;
- busca por alterações de working tree/índice/HEAD/refs/stash antes/depois;
- V5/V6/V7/V12 e reparador V5 dry-run.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada ou produz efeito proibido;
- `SPEC-GAP`: L0 não decide framing, erro, ambiente ou testabilidade necessária;
- `GATE-DEFECT`: gate usa produção/Git real como oráculo, simula expectativa circular,
  vaza processo ou mede exit pertencente a F09;
- `PASS`: alegação confrontada no envelope autorizado.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. baseline pós-merge P0099 e todos os insumos efetivos hash-pinned;
2. Assessment 0029 e preflight A congelados antes dos gates;
3. B1/B2 independentes, com fixtures/helpers separados;
4. hashes dos gates congelados e RED inicial preservado;
5. nenhum acesso à rede e nenhum efeito fora de diretórios temporários próprios;
6. watchdogs e limpeza de processos comprovados;
7. estado Git byte-identicamente preservado em sucesso e falha;
8. gates B1/B2 e regressões dirigidas verdes;
9. `cargo test --workspace --quiet`;
10. V5/V6/V7/V12, reparador V5 dry-run e `git diff --check`;
11. adversário D e worktree limpo.

## Saídas esperadas

- `00_nucleo/assessments/0029-auditoria-funcional-git-refinement.md`;
- parecer A e eventual saneamento L0 resselado;
- `tests/git_refinement_protocol_assessment.rs`;
- `tests/git_refinement_timeout_assessment.rs`;
- fixtures hostis independentes;
- correção mínima somente após RED;
- `00_nucleo/relatorio-p0100-auditoria-funcional-git-refinement.md`;
- efeito explícito: F05 `CLOSED` ou `BLOCKED` e dependência F08 atualizada;
- veredito final.

P0100 não autoriza alteração de exits/precedência, pipeline amplo, schema/writer de
snapshot, backend Git, dependências, instalação, merge, push ou release. Sem integração
prévia de P0099 em `master`, a execução deve parar antes de criar branch concorrente.
